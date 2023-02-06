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
        // refer to: https://stackoverflow.com/questions/4774172/image-manipulation-and-texture-mapping-using-html5-canvas
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
                        let i0 = mesh.indices[i];
                        i += 1;
                        let i1 = mesh.indices[i];
                        i += 1;
                        let i2 = mesh.indices[i];
                        i += 1;

                        let p0 = mesh.vertices[i0 as usize];
                        let p1 = mesh.vertices[i1 as usize];
                        let p2 = mesh.vertices[i2 as usize];
                        
                        let color0 = format!(
                            "#{:02X}{:02X}{:02X}",
                            p0.color.r(),
                            p0.color.g(),
                            p0.color.b()
                        );
                        let color1 = format!(
                            "#{:02X}{:02X}{:02X}",
                            p1.color.r(),
                            p1.color.g(),
                            p1.color.b()
                        );
                        let color2 = format!(
                            "#{:02X}{:02X}{:02X}",
                            p2.color.r(),
                            p2.color.g(),
                            p2.color.b()
                        );

                        let x0 = p0.pos.x as f64;
                        let x1 = p1.pos.x as f64;
                        let x2 = p2.pos.x as f64;

                        let y0 = p0.pos.y as f64;
                        let y1 = p1.pos.y as f64;
                        let y2 = p2.pos.y as f64;

                        let u0 = p0.uv.x as f64;
                        let u1 = p1.uv.x as f64;
                        let u2 = p2.uv.x as f64;

                        let v0 = p0.uv.y as f64;
                        let v1 = p1.uv.y as f64;
                        let v2 = p2.uv.y as f64;

                        // web_sys::console::log_1(&format!("color0: {}", color0).into());
                        // web_sys::console::log_1(&format!("color1: {}", color1).into());
                        // web_sys::console::log_1(&format!("color2: {}", color2).into());
                        canvas_ctx.save();
                        canvas_ctx.scale(pixels_per_point as f64, pixels_per_point as f64).unwrap();
                        canvas_ctx.set_stroke_style(&color0.clone().into());
                        canvas_ctx.set_fill_style(&color0.into());
                        canvas_ctx.set_line_width(0.);
                        canvas_ctx.begin_path();
                        canvas_ctx.move_to(x0, y0);
                        canvas_ctx.line_to(x1, y1);
                        canvas_ctx.line_to(x2, y2);
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
