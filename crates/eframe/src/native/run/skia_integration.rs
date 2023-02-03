
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

        let painter = SkiaPainter::new(skia_window.surface());

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
        self.is_focused
    }

    fn integration(&self) -> Option<&EpiIntegration> {
        self.running.as_ref().map(|r| &r.integration)
    }

    fn window(&self) -> Option<&winit::window::Window> {
        self.running.as_ref().map(|r| r.skia_window.window())
    }

    fn save_and_destroy(&mut self) {
        if let Some(mut running) = self.running.take() {
            running
                .integration
                .save(running.app.as_mut(), running.skia_window.window());
            running.app.on_exit();
            running.painter.destroy();
        }
    }

    fn paint(&mut self) -> EventResult {
        if let Some(running) = &mut self.running {
            #[cfg(feature = "puffin")]
            puffin::GlobalProfiler::lock().new_frame();
            crate::profile_scope!("frame");

            let SkiaWinitRunning {
                painter,
                integration,
                app,
                skia_window,
            } = running;

            let window = skia_window.window();

            let screen_size_in_pixels: [u32; 2] = window.inner_size().into();

            skia_window.clear(
                screen_size_in_pixels,
                app.clear_color(&integration.egui_ctx.style().visuals),
            );

            let egui::FullOutput {
                platform_output,
                repaint_after,
                textures_delta,
                shapes,
            } = integration.update(app.as_mut(), window);

            integration.handle_platform_output(window, platform_output);

            let clipped_primitives = {
                crate::profile_scope!("tessellate");
                integration.egui_ctx.tessellate(shapes)
            };

            painter.paint_and_update_textures(
                screen_size_in_pixels,
                integration.egui_ctx.pixels_per_point(),
                clipped_primitives,
                &textures_delta,
            );

            integration.post_rendering(app.as_mut(), window);

            {
                crate::profile_scope!("swap_buffers");
                skia_window.swap_buffers();
            }

            integration.post_present(window);

            #[cfg(feature = "__screenshot")]
            // give it time to settle:
            if self.frame_nr == 2 {
                if let Ok(path) = std::env::var("EFRAME_SCREENSHOT_TO") {
                    assert!(
                        path.ends_with(".png"),
                        "Expected EFRAME_SCREENSHOT_TO to end with '.png', got {path:?}"
                    );
                    let [w, h] = screen_size_in_pixels;
                    let pixels = painter.read_screen_rgba(screen_size_in_pixels);
                    let image = image::RgbaImage::from_vec(w, h, pixels).unwrap();
                    let image = image::imageops::flip_vertical(&image);
                    image.save(&path).unwrap_or_else(|err| {
                        panic!("Failed to save screenshot to {path:?}: {err}");
                    });
                    eprintln!("Screenshot saved to {path:?}.");
                    std::process::exit(0);
                }
            }

            let control_flow = if integration.should_close() {
                EventResult::Exit
            } else if repaint_after.is_zero() {
                EventResult::RepaintNext
            } else if let Some(repaint_after_instant) =
                std::time::Instant::now().checked_add(repaint_after)
            {
                // if repaint_after is something huge and can't be added to Instant,
                // we will use `ControlFlow::Wait` instead.
                // technically, this might lead to some weird corner cases where the user *WANTS*
                // winit to use `WaitUntil(MAX_INSTANT)` explicitly. they can roll their own
                // egui backend impl i guess.
                EventResult::RepaintAt(repaint_after_instant)
            } else {
                EventResult::Wait
            };

            integration.maybe_autosave(app.as_mut(), window);

            if !self.is_focused {
                // On Mac, a minimized Window uses up all CPU: https://github.com/emilk/egui/issues/325
                // We can't know if we are minimized: https://github.com/rust-windowing/winit/issues/208
                // But we know if we are focused (in foreground). When minimized, we are not focused.
                // However, a user may want an egui with an animation in the background,
                // so we still need to repaint quite fast.
                crate::profile_scope!("bg_sleep");
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            self.frame_nr += 1;

            control_flow
        } else {
            EventResult::Wait
        }
    }

    fn on_event(
        &mut self,
        event_loop: &EventLoopWindowTarget<UserEvent>,
        event: &winit::event::Event<'_, UserEvent>,
    ) -> EventResult {
        match event {
            winit::event::Event::Resumed => {
                if self.running.is_none() {
                    self.init_run_state(event_loop);
                }
                EventResult::RepaintNow
            }
            winit::event::Event::Suspended => {
                #[cfg(target_os = "android")]
                {
                    tracing::error!("Suspended app can't destroy Window surface state with current Egui Glow backend (undefined behaviour)");
                    // Instead of destroying everything which we _know_ we can't re-create
                    // we instead currently just try our luck with not destroying anything.
                    //
                    // When the application resumes then it will get a new `SurfaceView` but
                    // we have no practical way currently of creating a new EGL surface
                    // via the Glutin API while keeping the GL context and the rest of
                    // our app state. This will likely result in a black screen or
                    // frozen screen.
                    //
                    //self.running = None;
                }
                EventResult::Wait
            }

            winit::event::Event::WindowEvent { event, .. } => {
                if let Some(running) = &mut self.running {
                    // On Windows, if a window is resized by the user, it should repaint synchronously, inside the
                    // event handler.
                    //
                    // If this is not done, the compositor will assume that the window does not want to redraw,
                    // and continue ahead.
                    //
                    // In eframe's case, that causes the window to rapidly flicker, as it struggles to deliver
                    // new frames to the compositor in time.
                    //
                    // The flickering is technically glutin or glow's fault, but we should be responding properly
                    // to resizes anyway, as doing so avoids dropping frames.
                    //
                    // See: https://github.com/emilk/egui/issues/903
                    let mut repaint_asap = false;

                    match &event {
                        winit::event::WindowEvent::Focused(new_focused) => {
                            self.is_focused = *new_focused;
                        }
                        winit::event::WindowEvent::Resized(physical_size) => {
                            repaint_asap = true;

                            // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                            // See: https://github.com/rust-windowing/winit/issues/208
                            // This solves an issue where the app would panic when minimizing on Windows.
                            if physical_size.width > 0 && physical_size.height > 0 {
                                running.skia_window.resize(*physical_size);
                            }
                        }
                        winit::event::WindowEvent::ScaleFactorChanged {
                            new_inner_size,
                            ..
                        } => {
                            repaint_asap = true;
                            running.skia_window.resize(**new_inner_size);
                        }
                        winit::event::WindowEvent::CloseRequested
                            if running.integration.should_close() =>
                        {
                            return EventResult::Exit
                        }
                        _ => {}
                    }

                    let event_response =
                        running.integration.on_event(running.app.as_mut(), event);

                    if running.integration.should_close() {
                        EventResult::Exit
                    } else if event_response.repaint {
                        if repaint_asap {
                            EventResult::RepaintNow
                        } else {
                            EventResult::RepaintNext
                        }
                    } else {
                        EventResult::Wait
                    }
                } else {
                    EventResult::Wait
                }
            }
            #[cfg(feature = "accesskit")]
            winit::event::Event::UserEvent(UserEvent::AccessKitActionRequest(
                accesskit_winit::ActionRequestEvent { request, .. },
            )) => {
                if let Some(running) = &mut self.running {
                    running
                        .integration
                        .on_accesskit_action_request(request.clone());
                    // As a form of user input, accessibility actions should
                    // lead to a repaint.
                    EventResult::RepaintNext
                } else {
                    EventResult::Wait
                }
            }
            _ => EventResult::Wait,
        }
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
