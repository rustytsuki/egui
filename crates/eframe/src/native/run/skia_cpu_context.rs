use skia_safe::Surface;
use winit::window::Window;

use crate::epi;

pub struct SkiaCPUWindowContext {
}

impl SkiaCPUWindowContext {
    
}

pub fn new(winit_window: &Window, native_options: &epi::NativeOptions) -> Option<(SkiaCPUWindowContext, Surface)> {
    todo!()
}