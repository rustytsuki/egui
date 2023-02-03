use skia_safe::{Surface, gpu::{gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin}, ColorType, Color4f};
use winit::window::Window;
use super::{*, skia_painter::SkiaPainter, skia_gl_context::SkiaGlutinWindowContext, skia_cpu_context::SkiaCPUWindowContext};

pub static FORCE_CPU: bool = false;

pub struct SkiaWinitRunning {
    pub painter: SkiaPainter,
    pub integration: epi_integration::EpiIntegration,
    pub app: Box<dyn epi::App>,
    pub skia_window: SkiaWindowContext,
}

pub struct SkiaWindowContext {
    gl_context: Option<SkiaGlutinWindowContext>,
    cpu_context: Option<SkiaCPUWindowContext>,
    window: Window,
    surface: Surface,
}

impl SkiaWindowContext {
    pub fn new(winit_window: Window, native_options: &epi::NativeOptions) -> Self {
        if !FORCE_CPU {
            if let Some((gl_context, surface)) = skia_gl_context::new(&winit_window, native_options) {
                return Self {
                    gl_context: Some(gl_context),
                    cpu_context: None,
                    window: winit_window,
                    surface,
                };
            }
        }
        
        if let Some((cpu_context, surface)) = skia_cpu_context::new(&winit_window, native_options) {
            return Self {
                gl_context: None,
                cpu_context: Some(cpu_context),
                window: winit_window,
                surface,
            };
        } else {
            panic!("create window error!");
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn surface(&self) -> Surface {
        self.surface.clone()
    }

    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(gl_context) = &self.gl_context {
            gl_context.resize(physical_size);
        } else {
            todo!();
        }
    }

    pub fn clear(&self, screen_size_in_pixels: [u32; 2], clear_color: egui::Rgba) {
        let mut surface = self.surface.clone();
        surface.canvas().clear(Color4f::new(clear_color[0], clear_color[1], clear_color[2], clear_color[3]));
    }

    pub fn swap_buffers(&self) {
        {
            let mut surface = self.surface.clone();
            surface.flush();
        }

        if let Some(gl_context) = &self.gl_context {
            gl_context.swap_buffers();
        } else {
            todo!();
        }
    }
}
