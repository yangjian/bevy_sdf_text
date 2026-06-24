use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bevy::log;
use image::{DynamicImage, EncodableLayout, ImageFormat, Rgba32FImage};
use msdfgen::{
    Bitmap, Bound, FillRule, FontExt, Framing, Gray, MsdfGeneratorConfig, Pixel, Range, Rgba,
    Vector2, MID_VALUE,
};
use owned_ttf_parser::{Face, GlyphId};
use ttf_parser_msdf as msdf_ttf;

use crate::FontMetrics;

use super::{GlyphMetrics, SdfAtlas, SdfAtlasGlyph, SdfAtlasParams, SdfFont};

impl SdfFont {
    pub fn create(
        name: &str,
        data: Vec<u8>,
        altas_params: SdfAtlasParams,
        chars: Vec<char>,
    ) -> Result<Self> {
        let face = Face::parse(&data, 0).context("parse font face")?;
        let metrics = FontMetrics::new(&face).context("get font metrics")?;
        let atlas = SdfAtlas::create(&data, altas_params, chars).context("create SDF atlas")?;

        Ok(Self {
            name: name.to_owned(),
            data,
            metrics,
            atlas,
        })
    }
}

impl SdfAtlas {
    pub fn create(
        font_data: &[u8],
        altas_params: SdfAtlasParams,
        chars: Vec<char>,
    ) -> Result<Self> {
        let face = Face::parse(font_data, 0)?;
        let msdf_face =
            msdf_ttf::Face::parse(font_data, 0).map_err(|_| anyhow!("parse font face for msdf"))?;

        let mut gen_config = MsdfGeneratorConfig::default();
        gen_config.set_overlap_support(true);

        let px_range = altas_params.px_range as f64;
        let fill_rule = FillRule::default();
        let [dist_scale, dist_offset] = altas_params.distance_scale_offset;

        let (char_width, char_height) = (altas_params.char_size, altas_params.char_size);
        let char_gap = altas_params.char_gap;
        let char_glyphs: Vec<(char, GlyphId)> = chars
            .iter()
            .filter_map(|c| face.glyph_index(*c).map(|g| (*c, g)))
            .collect();
        let grid_width = 16_u32;
        let grid_height = (char_glyphs.len() as u32 + 15) / 16;

        let (img_width, img_height) = (
            (char_width + char_gap) * grid_width,
            (char_height + char_gap) * grid_height,
        );

        let mut out_image = Rgba32FImage::new(img_width, img_height);

        let mut glyphs = HashMap::new();
        let mut max_distance: [f32; 2] = [0.0, 0.0];

        for (i, (char, glyph_id)) in char_glyphs.into_iter().enumerate() {
            if char.is_whitespace() || char.is_ascii_control() {
                continue;
            }

            let grid_x = (i as u32) % grid_width;
            let grid_y = (i as u32) / grid_width;

            let mut shape = match msdf_face.glyph_shape(msdf_ttf::GlyphId(glyph_id.0)) {
                Some(o) => o,
                None => {
                    log::warn!("no glyph shape for char {char:?}");
                    continue;
                }
            };

            let metrics = GlyphMetrics::for_glyph(&face, glyph_id)
                .ok_or_else(|| anyhow!("no glyph metrics for char {char:?}"))?;

            let bound = Bound {
                left: metrics.x_min as f64,
                bottom: metrics.y_min as f64,
                right: metrics.x_max as f64,
                top: metrics.y_max as f64,
            };
            let framing: Framing<f64> = bound
                .autoframe(char_width, char_height, Range::px(px_range), None)
                .ok_or_else(|| anyhow!("no framing for char {char:?}"))?;

            let mut bitmap = Bitmap::new(char_width, char_width);
            shape.edge_coloring_simple(3.0, 4099870759);
            shape.generate_mtsdf(&mut bitmap, framing, gen_config);
            shape.correct_sign(&mut bitmap, framing, fill_rule);
            shape.correct_msdf_error(&mut bitmap, framing, gen_config);

            let (mut uv_p00, mut uv_p11) = project_bound(&framing, bound);

            // add texture coord UV offset
            uv_p00[0] += (char_gap / 2 + grid_x * (char_width + char_gap)) as f64;
            uv_p00[1] += (char_gap / 2 + grid_y * (char_height + char_gap)) as f64;
            uv_p11[0] += (char_gap / 2 + grid_x * (char_width + char_gap)) as f64;
            uv_p11[1] += (char_gap / 2 + grid_y * (char_height + char_gap)) as f64;

            let margin = altas_params.px_range as f64 / 2.0;

            let px_width = uv_p11[0] - uv_p00[0];
            let px_height = uv_p11[1] - uv_p00[1];

            let margin_ratio = [(margin / px_width) as f32, (margin / px_height) as f32];

            // normalize texture coord UV
            let u0 = (uv_p00[0] - margin) / img_width as f64;
            let v0 = (uv_p00[1] - margin) / img_height as f64;
            let u1 = (uv_p11[0] + margin) / img_width as f64;
            let v1 = (uv_p11[1] + margin) / img_height as f64;

            for y in 0..char_height {
                for x in 0..char_width {
                    let out_x = char_gap / 2 + grid_x * (char_width + char_gap) + x;
                    let out_y =
                        char_gap / 2 + grid_y * (char_height + char_gap) + char_height - 1 - y;
                    let out_pixel = out_image.get_pixel_mut(out_x, out_y);
                    let src_pixel = bitmap.pixel(x, y);
                    out_pixel[0] = (src_pixel.r / dist_scale + dist_offset).clamp(0.0, 1.0);
                    out_pixel[1] = (src_pixel.g / dist_scale + dist_offset).clamp(0.0, 1.0);
                    out_pixel[2] = (src_pixel.b / dist_scale + dist_offset).clamp(0.0, 1.0);
                    out_pixel[3] = (src_pixel.a / dist_scale + dist_offset).clamp(0.0, 1.0);

                    for c in src_pixel.components() {
                        if *c > max_distance[0] {
                            max_distance[0] = *c;
                        } else if *c < max_distance[1] {
                            max_distance[1] = *c;
                        }
                    }
                }
            }

            glyphs.insert(
                char,
                SdfAtlasGlyph {
                    char,
                    metrics,
                    margin_ratio,
                    tex_coord_p00: [u0 as f32, v0 as f32],
                    tex_coord_p11: [u1 as f32, v1 as f32],
                },
            );
        }

        log::info!("max distance: {max_distance:?} vs scale: {dist_scale}");

        let out_image = DynamicImage::ImageRgba32F(out_image).into_rgba16();
        let mut png_data: Vec<u8> = Vec::new();
        out_image
            .write_to(&mut Cursor::new(&mut png_data), ImageFormat::Png)
            .context("encode PNG")?;

        Ok(Self {
            params: altas_params,
            chars,
            grid_size: [grid_width, grid_height],
            glyphs,
            image_size: [img_width, img_height],
            png_data,
        })
    }

    pub fn mdsfgen_bitmap(&self) -> Result<Bitmap<Rgba<f32>>> {
        let image = self.to_f32_image()?;
        let (width, height) = image.dimensions();
        let mut bitmap: Bitmap<Rgba<f32>> = Bitmap::new(width, height);
        bitmap.raw_pixels_mut().copy_from_slice(image.as_bytes());
        Ok(bitmap)
    }

    pub fn export_bitmap_png<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path)?;
        let bitmap = self.mdsfgen_bitmap()?;
        bitmap.write_png(file)?;
        Ok(())
    }

    pub fn export_preview_png<P: AsRef<Path>>(&self, scale: (u32, u32), path: P) -> Result<()> {
        let bitmap = self.mdsfgen_bitmap()?;

        let (img_width, img_height) = (bitmap.width(), bitmap.height());
        let mut preview: Bitmap<Gray<f32>> = Bitmap::<Gray<f32>>::new(
            img_width * scale.0 / scale.1,
            img_height * scale.0 / scale.1,
        );

        bitmap.render(&mut preview, self.params.px_range as f64, MID_VALUE);

        let mut output: File = File::create(path.as_ref())?;
        preview.write_png(&mut output)?;

        Ok(())
    }
}

fn project_bound(framing: &Framing<f64>, bound: Bound<f64>) -> ([f64; 2], [f64; 2]) {
    let scale = framing.projection.scale;
    let translate = framing.projection.translate;
    let v0 = Vector2 {
        x: bound.left,
        y: bound.bottom,
    };
    let v1 = Vector2 {
        x: bound.right,
        y: bound.top,
    };
    let p00 = scale * (v0 + translate);
    let p11 = scale * (v1 + translate);
    ([p00.x, p00.y], [p11.x, p11.y])
}

#[cfg(test)]
mod tests {
    use crate::{printable_ascii_chars, SdfAtlasParams, SdfFont};

    #[test]
    fn test_sdf_atlas() {
        let _ = std::fs::create_dir_all("tmp");

        for name in [
            "Barlow-Regular",
            "Barlow-Bold",
            "Roboto-Regular",
            "Roboto-Bold",
        ] {
            let font_path = format!("assets/fonts/{}.ttf", name);
            let font_data = std::fs::read(&font_path).unwrap();
            let atlas_params: SdfAtlasParams = Default::default();
            let chars = printable_ascii_chars();
            let font = SdfFont::create(name, font_data, atlas_params, chars).unwrap();

            let bin_data = font.to_vec().unwrap();
            let sdfb_path = format!("tmp/{}.sdfb", name);
            std::fs::write(&sdfb_path, &bin_data).unwrap();
            println!("font {}: export sdf binary {}", name, sdfb_path);

            let loaded = SdfFont::from_slice(&bin_data).unwrap();
            let atlas = &loaded.atlas;

            let atlas_img_path = format!("tmp/{}-atlas.png", name);
            atlas.export_bitmap_png(&atlas_img_path).unwrap();
            println!("font {}: export bitmat image {}", name, atlas_img_path);

            let preview_img_path = format!("tmp/{}-preview.png", name);
            atlas.export_preview_png((8, 1), &preview_img_path).unwrap();
            println!("font {}: export preview image {}", name, preview_img_path);
        }
    }
}
