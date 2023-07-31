use std::{fs::File, path::Path};

use anyhow::{anyhow, Result};
use msdfgen::{Bitmap, FillRule, FontExt, Gray, MsdfGeneratorConfig, Range, Rgba, MID_VALUE};
use ttf_parser::{Face, GlyphId};

#[derive(Clone, Debug)]
pub struct MtsdfAtlasConfig {
    pub font_path: String,
    pub char_size: (u32, u32),
    pub char_gap: u32,
    pub px_range: f32,
}

impl MtsdfAtlasConfig {
    pub fn font(path: String) -> Self {
        Self {
            font_path: path,
            char_size: (72, 72),
            char_gap: 8,
            px_range: 4.0,
        }
    }
}

pub struct MtsdfAtlas {
    pub config: MtsdfAtlasConfig,
    pub chars: Vec<char>,
    pub grid_size: (u32, u32),
    pub bitmap: Bitmap<Rgba<f32>>,
}

impl MtsdfAtlas {
    pub fn create(config: &MtsdfAtlasConfig, chars: &[char]) -> Result<Self> {
        let data = std::fs::read(&config.font_path)?;
        let font = Face::parse(&data, 0)?;

        let gen_config = MsdfGeneratorConfig::default();
        let px_range = config.px_range as f64;

        let (char_width, char_height) = config.char_size;
        let char_gap = config.char_gap;
        let char_glyphs: Vec<(char, GlyphId)> = chars
            .iter()
            .filter_map(|c| font.glyph_index(*c).map(|g| (*c, g)))
            .collect();
        let grid_width = 16_u32;
        let grid_height = (char_glyphs.len() as u32 + 15) / 16;

        let (img_width, img_height) = (
            (char_width + char_gap) * grid_width,
            (char_height + char_gap) * grid_height,
        );
        let mut atlas_bitmap = Bitmap::<Rgba<f32>>::new(img_width, img_height);

        let out_chars: Vec<char> = Vec::new();

        for (i, (char, glyph_id)) in char_glyphs.into_iter().enumerate() {
            let bbox = font.glyph_bounding_box(glyph_id).unwrap();

            let mut shape = font
                .glyph_shape(glyph_id)
                .ok_or_else(|| anyhow!("no glyph shape for char {char:?}"))?;

            let bound = shape.get_bound();
            let framing = bound
                .autoframe(char_width, char_height, Range::px(px_range), None)
                .ok_or_else(|| anyhow!("no autoframe output for char {char:?}"))?;
            let fill_rule = FillRule::default();

            let mut bitmap = Bitmap::new(char_width, char_width);

            shape.edge_coloring_simple(3.0, 0);
            shape.generate_mtsdf(&mut bitmap, framing, gen_config);
            shape.correct_sign(&mut bitmap, framing, fill_rule);
            shape.correct_msdf_error(&mut bitmap, framing, gen_config);

            let error = shape.estimate_error(&mut bitmap, framing, 5, Default::default());
            println!("char {char:?}: bbox {bbox:?}, bound {bound:?}, estimated error: {error}");

            let grid_x = (i as u32) % grid_width;
            let grid_y = (i as u32) / grid_width;

            for y in 0..char_height {
                for x in 0..char_width {
                    let p = atlas_bitmap.pixel_mut(
                        char_gap / 2 + grid_x * (char_width + char_gap) + x,
                        char_gap / 2 + grid_y * (char_height + char_gap) + char_height - 1 - y,
                    );
                    *p = *bitmap.pixel(x, y);
                }
            }
        }

        Ok(MtsdfAtlas {
            config: config.clone(),
            chars: out_chars,
            grid_size: (grid_width, grid_height),
            bitmap: atlas_bitmap,
        })
    }

    pub fn export_bitmap_png<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let output = File::create(path.as_ref())?;
        self.bitmap.write_png(output)?;
        Ok(())
    }

    pub fn export_preview_png<P: AsRef<Path>>(&self, scale: (u32, u32), path: P) -> Result<()> {
        let img_width = self.bitmap.width();
        let img_height = self.bitmap.height();

        let mut preview: Bitmap<Gray<f32>> = Bitmap::<Gray<f32>>::new(
            img_width * scale.0 / scale.1,
            img_height * scale.0 / scale.1,
        );
        self.bitmap
            .render(&mut preview, self.config.px_range as f64, MID_VALUE);

        let mut output = File::create(path.as_ref())?;
        preview.write_png(&mut output)?;

        Ok(())
    }
}

pub fn printable_ascii_chars() -> Vec<char> {
    (33..=126_u32).filter_map(char::from_u32).collect()
}

#[cfg(test)]
mod tests {
    use crate::{printable_ascii_chars, MtsdfAtlas, MtsdfAtlasConfig};

    #[test]
    fn test_initialize_msdf() {
        let _ = std::fs::create_dir("tmp");

        let chars: Vec<char> = printable_ascii_chars();
        for name in ["Roboto-Regular", "FiraSans-Regular"] {
            let font_path = format!("assets/fonts/{}.ttf", name);
            let config = MtsdfAtlasConfig::font(font_path);
            let atlas: MtsdfAtlas = MtsdfAtlas::create(&config, &chars).unwrap();

            let atlas_img_path = format!("tmp/{}-atlas.png", name);
            atlas.export_bitmap_png(&atlas_img_path).unwrap();
            println!("font {}: export bitmat image {}", name, atlas_img_path);

            let preview_img_path = format!("tmp/{}-preview.png", name);
            atlas.export_preview_png((8, 1), &preview_img_path).unwrap();
            println!("font {}: export preview image {}", name, preview_img_path);
        }
    }
}
