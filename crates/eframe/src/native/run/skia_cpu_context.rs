use skia_bindings::SkPixmap;
use skia_safe::{Surface, ImageInfo, ColorType, AlphaType, wrapper::ValueWrapper, Pixel, Handle};
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
        let surface = Self::create_surface(winit_window.inner_size());
        Self {
            surface,
            graphics_context,
        }
    }

    pub fn create_surface(physical_size: winit::dpi::PhysicalSize<u32>) -> Surface {
        let corlor_type = if cfg!(target_os = "macos") { ColorType::BGRA8888 } else { ColorType::RGBA8888 };
        let image_info = ImageInfo::new((physical_size.width as i32, physical_size.height as i32), corlor_type, AlphaType::Premul, None);
        Surface::new_raster(&image_info, None, None).unwrap()
    }

    pub fn resize(&mut self, physical_size: winit::dpi::PhysicalSize<u32>) {
        self.surface = Self::create_surface(physical_size);
    }

    pub fn swap_buffers(&mut self) {
        let pixmap = self.surface.peek_pixels().unwrap();
        let pixels: &[u32] = Self::pixels_to_u32(&pixmap).unwrap();
        
        let width = pixmap.width();
        let height = pixmap.height();

        self.graphics_context.set_buffer(pixels, width as u16, height as u16);
    }

    pub fn pixels_to_u32<P: Pixel>(pixmap: &Handle<SkPixmap>) -> Option<&[P]> {
        let addr = unsafe { pixmap.addr() };

        let info = pixmap.info();
        let pixel_size = std::mem::size_of::<P>();

        if info.bytes_per_pixel() == pixel_size {
            let len = pixmap.compute_byte_size() / pixel_size;
            return Some(unsafe { std::slice::from_raw_parts(addr as *const P, len) });
        }

        None
    }
}
