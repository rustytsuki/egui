use egui::ImageData;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

use egui::Rgba;

use crate::WebOptions;

use super::web_painter::WebPainter;

#[derive(Eq, PartialEq)]
enum PaintType {
    Image,
    Font,
}

struct PaintHandle {
    image: HtmlCanvasElement,
    paint_type: PaintType,
}

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
        let ctx = &self.canvas_ctx;

        textures_delta.set.iter().for_each(|(id, image_delta)| {
            let delta_image = match &image_delta.image {
                ImageData::Color(color_image) => {
                    let pixels = color_image.pixels.iter().flat_map(|p| p.to_array()).collect::<Vec<_>>();
                    web_sys::ImageData::new_with_u8_clamped_array_and_sh(wasm_bindgen::Clamped(&pixels), color_image.width() as u32, color_image.height() as u32).unwrap()
                }
                ImageData::Font(font) => {
                    let pixels = font.srgba_pixels(Some(1.0)).flat_map(|p| p.to_array()).collect::<Vec<_>>();
                    web_sys::ImageData::new_with_u8_clamped_array_and_sh(wasm_bindgen::Clamped(&pixels), font.width() as u32, font.height() as u32).unwrap()
                }
            };
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
                        ctx.save();
                        ctx.scale(pixels_per_point as f64, pixels_per_point as f64).unwrap();
                        ctx.set_stroke_style(&color0.clone().into());
                        ctx.set_fill_style(&color0.into());
                        ctx.set_line_width(0.);
                        ctx.begin_path();
                        ctx.move_to(x0, y0);
                        ctx.line_to(x1, y1);
                        ctx.line_to(x2, y2);
                        ctx.close_path();
                        ctx.clip();

                        // Compute matrix transform
                        let delta = u0*v1 + v0*u2 + u1*v2 - v1*u2 - v0*u1 - u0*v2;
                        let delta_a = x0*v1 + v0*x2 + x1*v2 - v1*x2 - v0*x1 - x0*v2;
                        let delta_b = u0*x1 + x0*u2 + u1*x2 - x1*u2 - x0*u1 - u0*x2;
                        let delta_c = u0*v1*x2 + v0*x1*u2 + x0*u1*v2 - x0*v1*u2 - v0*u1*x2 - u0*x1*v2;
                        let delta_d = y0*v1 + v0*y2 + y1*v2 - v1*y2 - v0*y1 - y0*v2;
                        let delta_e = u0*y1 + y0*u2 + u1*y2 - y1*u2 - y0*u1 - u0*y2;
                        let delta_f = u0*v1*y2 + v0*y1*u2 + y0*u1*v2 - y0*v1*u2 - v0*u1*y2 - u0*y1*v2;

                        // Draw the transformed image
                        ctx.transform(delta_a/delta, delta_d/delta,
                                    delta_b/delta, delta_e/delta,
                                    delta_c/delta, delta_f/delta).ok();

                        // fill texture
                        // ctx.drawImage(texture, 0, 0);

                        ctx.restore();
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
