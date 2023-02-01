
use super::{*, skia_winit_running::{SkiaWindowContext, SkiaWinitRunning}, skia_painter::SkiaPainter};
use std::sync::Arc;

pub struct SkiaWinitApp {
    repaint_proxy: Arc<egui::mutex::Mutex<EventLoopProxy<UserEvent>>>,
    app_name: String,
    native_options: NativeOptions,
    running: Option<SkiaWinitRunning>,

    // Note that since this `AppCreator` is FnOnce we are currently unable to support
    // re-initializing the `GlowWinitRunning` state on Android if the application
    // suspends and resumes.
    app_creator: Option<epi::AppCreator>,
    is_focused: bool,

    frame_nr: u64,
}

impl SkiaWinitApp {
    pub fn new(
        event_loop: &EventLoop<UserEvent>,
        app_name: &str,
        native_options: NativeOptions,
        app_creator: epi::AppCreator,
    ) -> Self {
        Self {
            repaint_proxy: Arc::new(egui::mutex::Mutex::new(event_loop.create_proxy())),
            app_name: app_name.to_owned(),
            native_options,
            running: None,
            app_creator: Some(app_creator),
            is_focused: true,
            frame_nr: 0,
        }
    }

    fn create_skia_windowed_context(
        event_loop: &EventLoopWindowTarget<UserEvent>,
        storage: Option<&dyn epi::Storage>,
        title: &String,
        native_options: &NativeOptions,
    ) -> SkiaWindowContext {
        crate::profile_function!();

        let window_settings = epi_integration::load_window_settings(storage);

        let window_builder = epi_integration::window_builder(native_options, &window_settings)
            .with_title(title)
            .with_transparent(native_options.transparent)
            // Keep hidden until we've painted something. See https://github.com/emilk/egui/pull/2279
            // We must also keep the window hidden until AccessKit is initialized.
            .with_visible(false);
        let winit_window = window_builder
            .build(event_loop)
            .expect("failed to create winit window");

        SkiaWindowContext::new(winit_window, native_options)
    }

    fn init_run_state(&mut self, event_loop: &EventLoopWindowTarget<UserEvent>) {
        let storage = epi_integration::create_storage(&self.app_name);

        let skia_window = Self::create_skia_windowed_context(
            event_loop,
            storage.as_deref(),
            &self.app_name,
            &self.native_options,
        );

        let painter = SkiaPainter::new();

        let system_theme = self.native_options.system_theme();
        let mut integration = epi_integration::EpiIntegration::new(
            event_loop,
            painter.max_texture_side(),
            skia_window.window(),
            system_theme,
            storage
        );
        #[cfg(feature = "accesskit")]
        {
            integration.init_accesskit(skia_window.window(), self.repaint_proxy.lock().clone());
        }
        let theme = system_theme.unwrap_or(self.native_options.default_theme);
        integration.egui_ctx.set_visuals(theme.egui_visuals());

        skia_window.window().set_ime_allowed(true);
        if self.native_options.mouse_passthrough {
            skia_window.window().set_cursor_hittest(false).unwrap();
        }

        {
            let event_loop_proxy = self.repaint_proxy.clone();
            integration.egui_ctx.set_request_repaint_callback(move || {
                event_loop_proxy
                    .lock()
                    .send_event(UserEvent::RequestRepaint)
                    .ok();
            });
        }

        let app_creator = std::mem::take(&mut self.app_creator)
            .expect("Single-use AppCreator has unexpectedly already been taken");
        let mut app = app_creator(&epi::CreationContext {
            egui_ctx: integration.egui_ctx.clone(),
            integration_info: integration.frame.info(),
            storage: integration.frame.storage()
        });

        if app.warm_up_enabled() {
            integration.warm_up(app.as_mut(), skia_window.window());
        }

        self.running = Some(SkiaWinitRunning {
            painter,
            integration,
            app,
            skia_window,
        });
    }
}

impl WinitApp for SkiaWinitApp {
    fn is_focused(&self) -> bool {
        todo!()
    }

    fn integration(&self) -> Option<&EpiIntegration> {
        todo!()
    }

    fn window(&self) -> Option<&winit::window::Window> {
        todo!()
    }

    fn save_and_destroy(&mut self) {
        todo!()
    }

    fn paint(&mut self) -> EventResult {
        todo!()
    }

    fn on_event(
        &mut self,
        event_loop: &EventLoopWindowTarget<UserEvent>,
        event: &winit::event::Event<'_, UserEvent>,
    ) -> EventResult {
        todo!()
    }
}

pub fn run_skia(
    app_name: &str,
    mut native_options: epi::NativeOptions,
    app_creator: epi::AppCreator,
) {
    if native_options.run_and_return {
        with_event_loop(native_options, |event_loop, mut native_options| {
            if native_options.centered {
                center_window_pos(event_loop.available_monitors().next(), &mut native_options);
            }

            let skia_eframe = SkiaWinitApp::new(event_loop, app_name, native_options, app_creator);
            run_and_return(event_loop, skia_eframe);
        });
    } else {
        let event_loop = create_event_loop_builder(&mut native_options).build();

        if native_options.centered {
            center_window_pos(event_loop.available_monitors().next(), &mut native_options);
        }

        let skia_eframe = SkiaWinitApp::new(&event_loop, app_name, native_options, app_creator);
        run_and_exit(event_loop, skia_eframe);
    }
}
