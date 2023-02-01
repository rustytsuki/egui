
use super::*;
use std::sync::Arc;

pub struct SkiaWinitApp {
    repaint_proxy: Arc<egui::mutex::Mutex<EventLoopProxy<UserEvent>>>,
    app_name: String,
    native_options: NativeOptions,
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
            app_creator: Some(app_creator),
            is_focused: true,
            frame_nr: 0,
        }
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
