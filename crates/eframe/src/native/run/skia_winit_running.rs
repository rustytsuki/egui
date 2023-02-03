use skia_safe::{Surface, gpu::{gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin}, ColorType, Color4f};

use super::{*, skia_painter::SkiaPainter, glow_integration::GlutinWindowContext};

pub struct SkiaWinitRunning {
    pub painter: SkiaPainter,
    pub integration: epi_integration::EpiIntegration,
    pub app: Box<dyn epi::App>,
    pub skia_window: SkiaWindowContext,
}

pub struct SkiaWindowContext {
    gl_window: GlutinWindowContext,
    surface: Surface,
}

impl SkiaWindowContext {
    pub fn new(winit_window: winit::window::Window, native_options: &epi::NativeOptions,) -> Self {
        let gl_window = unsafe { GlutinWindowContext::new(winit_window, native_options) };
        let gl = gl_rs::load_with(|s| {
            let s = std::ffi::CString::new(s).expect("failed to construct C string from string for gl proc address");
            gl_window.get_proc_address(&s)
        });

        let mut surface = create_surface(gl_window.window()).unwrap();

        Self {
            gl_window,
            surface
        }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.gl_window.window()
    }

    pub fn surface(&self) -> Surface {
        self.surface.clone()
    }

    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        self.gl_window.resize(physical_size);
    }

    pub fn clear(&self, screen_size_in_pixels: [u32; 2], clear_color: egui::Rgba) {
        let mut surface = self.surface.clone();
        surface.canvas().clear(Color4f::new(clear_color[0], clear_color[1], clear_color[2], clear_color[3]));
    }

    pub fn swap_buffers(&self) -> glutin::error::Result<()> {
        let mut surface = self.surface.clone();
        surface.flush();
        self.gl_window.swap_buffers()
    }
}

fn create_surface(window: &winit::window::Window) -> Option<Surface> {
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