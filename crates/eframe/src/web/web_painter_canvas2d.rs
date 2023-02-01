use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

use egui::Rgba;

use crate::{WebGlContextOption, WebOptions};

use super::web_painter::WebPainter;

pub(crate) struct WebPainterCanvas2D {
    canvas: HtmlCanvasElement,
    canvas_id: String,
}

impl WebPainterCanvas2D {
    pub async fn new(canvas_id: &str, options: &WebOptions) -> Result<Self, String> {
        let canvas = super::canvas_element_or_die(canvas_id);

        Ok(Self {
            canvas,
            canvas_id: canvas_id.to_owned(),
        })
    }
}

impl WebPainter for WebPainterCanvas2D {
    fn canvas_id(&self) -> &str{
        todo!()
    }

    /// Maximum size of a texture in one direction.
    fn max_texture_side(&self) -> usize {
        todo!()
    }

    /// Update all internal textures and paint gui.
    fn paint_and_update_textures(
        &mut self,
        clear_color: Rgba,
        clipped_primitives: &[egui::ClippedPrimitive],
        pixels_per_point: f32,
        textures_delta: &egui::TexturesDelta,
    ) -> Result<(), JsValue> {
        todo!()
    }

    /// Destroy all resources.
    fn destroy(&mut self) {
        todo!()
    }
}
