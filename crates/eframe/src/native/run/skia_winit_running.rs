use skia_safe::{Surface, Color4f, ConditionallySend, Canvas};
use winit::{window::Window, dpi::PhysicalSize};
use super::{*, skia_painter::SkiaPainter, skia_gl_context::SkiaGlutinWindowContext, skia_cpu_context::SkiaCPUWindowContext};

pub static FORCE_CPU: bool = cfg!(feature = "skia_force_cpu");

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
}

impl SkiaWindowContext {
    pub fn new(winit_window: Window, native_options: &epi::NativeOptions) -> Self {
        if !FORCE_CPU {
            if let Some(gl_context) = unsafe { SkiaGlutinWindowContext::new(&winit_window, native_options) } {
                return Self {
                    gl_context: Some(gl_context),
                    cpu_context: None,
                    window: winit_window,
                };
            }
        }
        
        let cpu_context = SkiaCPUWindowContext::new(&winit_window, native_options);
        return Self {
            gl_context: None,
            cpu_context: Some(cpu_context),
            window: winit_window,
        };
    }

    pub fn is_cpu(&self) -> bool {
        self.gl_context.is_none()
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn canvas(&mut self) -> &mut Canvas {
        if let Some(gl_context) = &mut self.gl_context {
            gl_context.surface.canvas()
        } else {
            self.cpu_context.as_mut().unwrap().surface.canvas()
        }
    }

    pub fn resize(&mut self, physical_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(gl_context) = &mut self.gl_context {
            gl_context.resize(physical_size);
        } else {
            self.cpu_context.as_mut().unwrap().resize(physical_size);
        }
    }

    pub fn clear(&mut self, screen_size_in_pixels: [u32; 2], clear_color: egui::Rgba) {
        self.canvas().clear(Color4f::new(clear_color[0], clear_color[1], clear_color[2], clear_color[3]));
    }

    pub fn swap_buffers(&mut self) {
        if let Some(gl_context) = &mut self.gl_context {
            gl_context.swap_buffers();
        } else {
            self.cpu_context.as_mut().unwrap().swap_buffers();
        }
    }
}
