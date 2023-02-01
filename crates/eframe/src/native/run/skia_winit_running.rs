use super::{*, skia_painter::SkiaPainter};

pub struct SkiaWinitRunning {
    pub painter: SkiaPainter,
    pub integration: epi_integration::EpiIntegration,
    pub app: Box<dyn epi::App>,
    pub skia_window: SkiaWindowContext,
}

pub struct SkiaWindowContext {
    window: winit::window::Window,
}

impl SkiaWindowContext {
    pub fn new(winit_window: winit::window::Window, native_options: &epi::NativeOptions,) -> Self {
        todo!()
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }
    fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        todo!()
    }
}