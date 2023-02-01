use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

use egui::Rgba;

use crate::WebOptions;

use super::web_painter::WebPainter;

pub(crate) struct WebPainterCanvas2D {
    canvas: HtmlCanvasElement,
    canvas_ctx: CanvasRenderingContext2d,
    canvas_id: String,
}

impl WebPainterCanvas2D {
    pub async fn new(canvas_id: &str, options: &WebOptions) -> Result<Self, String> {
        let canvas = super::canvas_element_or_die(canvas_id);
        let canvas_ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        Ok(Self {
            canvas,
            canvas_ctx,
            canvas_id: canvas_id.to_owned(),
        })
    }
}

impl WebPainter for WebPainterCanvas2D {
    fn canvas_id(&self) -> &str {
        &self.canvas_id
    }

    /// Maximum size of a texture in one direction.
    fn max_texture_side(&self) -> usize {
        65535
    }

    /// Update all internal textures and paint gui.
    fn paint_and_update_textures(
        &mut self,
        clear_color: Rgba,
        clipped_primitives: &[egui::ClippedPrimitive],
        pixels_per_point: f32,
        textures_delta: &egui::TexturesDelta,
    ) -> Result<(), JsValue> {
        let canvas_ctx = &self.canvas_ctx;

        textures_delta.set.iter().for_each(|(id, image_delta)| {
            tracing::debug!("textures_delta set");
        });

        for primitive in clipped_primitives {
            let info = format!(
                "primitive: {}, {}, {}, {}",
                primitive.clip_rect.min.x,
                primitive.clip_rect.min.y,
                primitive.clip_rect.max.x,
                primitive.clip_rect.max.y,
            );
            tracing::debug!(info);

            match &primitive.primitive {
                egui::epaint::Primitive::Mesh(mesh) => {
                    tracing::debug!("primitive: {}", mesh.indices.len());
                    let mut i = 0;
                    while i < mesh.indices.len() {
                        let p0 = mesh.indices[i];
                        i += 1;
                        let p1 = mesh.indices[i];
                        i += 1;
                        let p2 = mesh.indices[i];
                        i += 1;

                        let v0 = mesh.vertices[p0 as usize];
                        let v1 = mesh.vertices[p1 as usize];
                        let v2 = mesh.vertices[p2 as usize];
                        let color0 = format!(
                            "#{:02X}{:02X}{:02X}",
                            v0.color.r(),
                            v0.color.g(),
                            v0.color.b()
                        );
                        let color1 = format!(
                            "#{:02X}{:02X}{:02X}",
                            v1.color.r(),
                            v1.color.g(),
                            v1.color.b()
                        );
                        let color2 = format!(
                            "#{:02X}{:02X}{:02X}",
                            v2.color.r(),
                            v2.color.g(),
                            v2.color.b()
                        );
                        // web_sys::console::log_1(&format!("color0: {}", color0).into());
                        // web_sys::console::log_1(&format!("color1: {}", color1).into());
                        // web_sys::console::log_1(&format!("color2: {}", color2).into());
                        canvas_ctx.save();
                        canvas_ctx.scale(pixels_per_point as f64, pixels_per_point as f64).unwrap();
                        canvas_ctx.set_stroke_style(&color0.clone().into());
                        canvas_ctx.set_fill_style(&color0.into());
                        canvas_ctx.set_line_width(0.);
                        canvas_ctx.begin_path();
                        canvas_ctx.move_to(v0.pos.x as f64, v0.pos.y as f64);
                        canvas_ctx.line_to(v1.pos.x as f64, v1.pos.y as f64);
                        canvas_ctx.line_to(v2.pos.x as f64, v2.pos.y as f64);
                        canvas_ctx.close_path();
                        canvas_ctx.fill();
                        canvas_ctx.restore();
                    }
                }
                egui::epaint::Primitive::Callback(_) => todo!(),
            }
        }

        textures_delta.free.iter().for_each(|id| {
            tracing::debug!("textures_delta free");
        });

        Ok(())
    }

    /// Destroy all resources.
    fn destroy(&mut self) {}
}
