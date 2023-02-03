use skia_safe::{Surface, ImageInfo, ColorType, AlphaType, wrapper::ValueWrapper};
use softbuffer::GraphicsContext;
use winit::window::Window;

use crate::epi;

pub struct SkiaCPUWindowContext {
    pub surface: Surface,
    graphics_context: GraphicsContext,
}

impl SkiaCPUWindowContext {
    pub fn new(winit_window: &Window, native_options: &epi::NativeOptions) -> Self {
        let graphics_context = unsafe { GraphicsContext::new(&winit_window, &winit_window) }.unwrap();
        let (width, height): (i32, i32) = winit_window.inner_size().into();
        let image_info = ImageInfo::new((width, height), ColorType::N32, AlphaType::Premul, None);
        let surface = Surface::new_raster(&image_info, None, None).unwrap();
        Self {
            surface,
            graphics_context,
        }
    }

    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
    }

    pub fn swap_buffers(&mut self) {
        self.surface.flush();

        let pixmap = self.surface.peek_pixels().unwrap();
        let pixels: &[u32] = pixmap.pixels().unwrap();
        
        let width = pixmap.width();
        let height = pixmap.height();

        self.graphics_context.set_buffer(&pixels, width as u16, height as u16);
    }
}
