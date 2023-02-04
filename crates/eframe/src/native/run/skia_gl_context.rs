use skia_safe::{Surface, gpu::{gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin}, ColorType, Color4f};
use winit::window::Window;

use crate::epi;

pub struct SkiaGlutinWindowContext {
    pub surface: Surface,   // surface must be ahead of gl_context, for the order of destroying
    gl_context: glutin::context::PossiblyCurrentContext,
    gl_display: glutin::display::Display,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl SkiaGlutinWindowContext {
    // refactor this function to use `glutin-winit` crate eventually.
    // preferably add android support at the same time.
    #[allow(unsafe_code)]
    pub unsafe fn new(
        winit_window: &winit::window::Window,
        native_options: &epi::NativeOptions,
    ) -> Option<Self> {
        // copied from run::glow_integration::GlutinWindowContext
        use glutin::prelude::*;
        use raw_window_handle::*;
        let hardware_acceleration = match native_options.hardware_acceleration {
            crate::HardwareAcceleration::Required => Some(true),
            crate::HardwareAcceleration::Preferred => None,
            crate::HardwareAcceleration::Off => Some(false),
        };

        let raw_display_handle = winit_window.raw_display_handle();
        let raw_window_handle = winit_window.raw_window_handle();

        // EGL is crossplatform and the official khronos way
        // but sometimes platforms/drivers may not have it, so we use back up options where possible.
        // TODO: check whether we can expose these options as "features", so that users can select the relevant backend they want.

        // try egl and fallback to windows wgl. Windows is the only platform that *requires* window handle to create display.
        #[cfg(target_os = "windows")]
        let preference =
            glutin::display::DisplayApiPreference::EglThenWgl(Some(raw_window_handle));
        // try egl and fallback to x11 glx
        #[cfg(target_os = "linux")]
        let preference = glutin::display::DisplayApiPreference::EglThenGlx(Box::new(
            winit::platform::unix::register_xlib_error_hook,
        ));
        #[cfg(target_os = "macos")]
        let preference = glutin::display::DisplayApiPreference::Cgl;
        #[cfg(target_os = "android")]
        let preference = glutin::display::DisplayApiPreference::Egl;

        let gl_display = glutin::display::Display::new(raw_display_handle, preference)
            .expect("failed to create glutin display");
        let swap_interval = if native_options.vsync {
            glutin::surface::SwapInterval::Wait(std::num::NonZeroU32::new(1).unwrap())
        } else {
            glutin::surface::SwapInterval::DontWait
        };

        let config_template = glutin::config::ConfigTemplateBuilder::new()
            .prefer_hardware_accelerated(hardware_acceleration)
            .with_depth_size(native_options.depth_buffer);
        // we don't know if multi sampling option is set. so, check if its more than 0.
        let config_template = if native_options.multisampling > 0 {
            config_template.with_multisampling(
                native_options
                    .multisampling
                    .try_into()
                    .expect("failed to fit multisamples into u8"),
            )
        } else {
            config_template
        };
        let config_template = config_template
            .with_stencil_size(native_options.stencil_buffer)
            .with_transparency(native_options.transparent)
            .compatible_with_native_window(raw_window_handle)
            .build();
        // finds all valid configurations supported by this display that match the config_template
        // this is where we will try to get a "fallback" config if we are okay with ignoring some native
        // options required by user like multi sampling, srgb, transparency etc..
        // TODO: need to figure out a good fallback config template
        let config = gl_display
            .find_configs(config_template)
            .expect("failed to find even a single matching configuration")
            .next()
            .expect("failed to find a matching configuration for creating opengl context");

        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(Some(raw_window_handle));
        // for surface creation.
        let (width, height): (u32, u32) = winit_window.inner_size().into();
        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    raw_window_handle,
                    std::num::NonZeroU32::new(width).unwrap(),
                    std::num::NonZeroU32::new(height).unwrap(),
                );
        // start creating the gl objects
        let gl_context = gl_display
            .create_context(&config, &context_attributes)
            .expect("failed to create opengl context");

        let gl_surface = gl_display
            .create_window_surface(&config, &surface_attributes)
            .expect("failed to create glutin window surface");
        let gl_context = gl_context
            .make_current(&gl_surface)
            .expect("failed to make gl context current");
        gl_surface
            .set_swap_interval(&gl_context, swap_interval)
            .expect("failed to set vsync swap interval");

        // create skia gl surface
        gl_rs::load_with(|s| {
            let s = std::ffi::CString::new(s).expect("failed to construct C string from string for gl proc address");
            gl_display.get_proc_address(&s)
        });

        let surface = create_surface(winit_window).unwrap();

        Some(SkiaGlutinWindowContext {
            surface,
            gl_context,
            gl_display,
            gl_surface,
        })
    }
    pub fn resize(&mut self, physical_size: winit::dpi::PhysicalSize<u32>) {
        use glutin::surface::GlSurface;
        self.gl_surface.resize(
            &self.gl_context,
            physical_size
                .width
                .try_into()
                .expect("physical size must not be zero"),
            physical_size
                .height
                .try_into()
                .expect("physical size must not be zero"),
        );
    }

    pub fn swap_buffers(&mut self) -> glutin::error::Result<()> {
        self.surface.flush();

        use glutin::surface::GlSurface;
        self.gl_surface.swap_buffers(&self.gl_context)
    }
}

fn create_surface(window: &Window) -> Option<Surface> {
    if let Some(mut gr_context) = skia_safe::gpu::DirectContext::new_gl(None, None) {
        let fb_info = {
            let mut fboid: gl_rs::types::GLint = 0;
            unsafe { gl_rs::GetIntegerv(gl_rs::FRAMEBUFFER_BINDING, &mut fboid) };
            
            let mut max_texture_side = 0;
            unsafe { gl_rs::GetIntegerv(gl_rs::MAX_TEXTURE_SIZE, &mut max_texture_side); }

            FramebufferInfo {
                fboid: fboid.try_into().unwrap(),
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
            }
        };

        let stencil_bits = 8;
        let (width, height): (i32, i32) = window.inner_size().into();
        let backend_render_target = BackendRenderTarget::new_gl((width, height), None, stencil_bits, fb_info);
        if let Some(surface) = Surface::from_backend_render_target(
            &mut gr_context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        ) {
            return Some(surface);
        }
    }

    None
}