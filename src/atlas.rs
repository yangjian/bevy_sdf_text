use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::{anyhow, Result};
use bevy::prelude::{Rect, Vec2};
use msdfgen::{
    Bitmap, Bound, FillRule, FontExt, Framing, Gray, MsdfGeneratorConfig, Projection, Range, Rgba,
    Vector2, MID_VALUE,
};
use ttf_parser::{Face, GlyphId};

#[derive(Clone, Copy, Debug)]
pub struct FontMetrics {
    pub glyph_count: u32,
    pub units_per_em: f32,
    pub ascender: f32,
    pub descender: f32,
    pub line_gap: f32,
    pub space_advance: f32,
}

impl FontMetrics {
    pub fn new(font: &Face) -> Option<Self> {
        let space_id = font.glyph_index(' ')?;
        let space_advance = font.glyph_hor_advance(space_id)? as f32;
        Some(Self {
            glyph_count: font.number_of_glyphs() as u32,
            units_per_em: font.units_per_em() as f32,
            ascender: font.ascender() as f32,
            descender: font.descender() as f32,
            line_gap: font.line_gap() as f32,
            space_advance,
        })
    }
}

// Glyph Metrics: https://freetype.org/freetype2/docs/glyphs/glyphs-3.html
#[derive(Clone, Copy, Debug)]
pub struct GlyphMetrics {
    pub advance: f32,
    pub bearing: f32,
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

impl GlyphMetrics {
    pub fn for_glyph(font: &Face, glyph_id: GlyphId) -> Option<Self> {
        let advance = font.glyph_hor_advance(glyph_id)?;
        let bearing = font.glyph_hor_side_bearing(glyph_id)?;
        let bbox: ttf_parser::Rect = font.glyph_bounding_box(glyph_id)?;
        Some(Self {
            advance: advance as f32,
            bearing: bearing as f32,
            x_min: bbox.x_min as f32,
            y_min: bbox.y_min as f32,
            x_max: bbox.x_max as f32,
            y_max: bbox.y_max as f32,
        })
    }

    pub fn transformed_bbox(&self, position: Vec2, scale: Vec2) -> Rect {
        Rect::new(
            position.x + self.x_min * scale.x,
            position.y + self.y_min * scale.y,
            position.x + self.x_max * scale.x,
            position.y + self.y_max * scale.y,
        )
    }
}

#[derive(Clone, Debug)]
pub struct MtsdfAtlasConfig {
    pub font_path: String,
    pub char_size: u32,
    pub char_gap: u32,
    pub px_range: u32,
}

impl MtsdfAtlasConfig {
    pub fn new(font_path: String) -> Self {
        Self {
            font_path,
            char_size: 64,
            char_gap: 8,
            px_range: 4,
        }
    }

    pub fn margin_pixels(&self) -> f32 {
        self.px_range as f32 * 0.5
    }
}

pub struct MtsdfAtlas {
    pub config: MtsdfAtlasConfig,
    pub font_metrics: FontMetrics,
    pub grid_size: (u32, u32),
    pub bitmap: Bitmap<Rgba<f32>>,
    pub glyphs: HashMap<char, MtsdfAtlasGlyph>,
}

pub struct MtsdfAtlasGlyph {
    pub char: char,
    pub metrics: GlyphMetrics,
    pub projection: Projection<f64>,
    pub tex_coord_p00: [f32; 2],
    pub tex_coord_p11: [f32; 2],
}

impl MtsdfAtlas {
    pub fn create(config: &MtsdfAtlasConfig, chars: &[char]) -> Result<Self> {
        let data = std::fs::read(&config.font_path)?;
        let font = Face::parse(&data, 0)?;
        let font_metrics =
            FontMetrics::new(&font).ok_or_else(|| anyhow!("unable to get font metrics"))?;

        let mut gen_config = MsdfGeneratorConfig::default();
        gen_config.set_overlap_support(true);

        let px_range = config.px_range as f64;
        let fill_rule = FillRule::default();

        let (char_width, char_height) = (config.char_size, config.char_size);
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

        let mut glyphs = HashMap::new();

        for (i, (char, glyph_id)) in char_glyphs.into_iter().enumerate() {
            if char.is_whitespace() || char.is_ascii_control() {
                continue;
            }

            let grid_x = (i as u32) % grid_width;
            let grid_y = (i as u32) / grid_width;

            let mut shape = match font.glyph_shape(glyph_id) {
                Some(o) => o,
                None => {
                    println!("no glyph shape for char {char:?}");
                    continue;
                }
            };

            let metrics = GlyphMetrics::for_glyph(&font, glyph_id)
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
            shape.edge_coloring_simple(3.0, 0);
            shape.generate_mtsdf(&mut bitmap, framing, gen_config);
            shape.correct_sign(&mut bitmap, framing, fill_rule);
            shape.correct_msdf_error(&mut bitmap, framing, gen_config);

            let (mut uv_p00, mut uv_p11) = project_bound(&framing, bound);

            // add texture coord UV offset
            uv_p00[0] +=
                (char_gap / 2 + grid_x * (char_width + char_gap) - config.px_range / 2) as f64;
            uv_p00[1] +=
                (char_gap / 2 + grid_y * (char_height + char_gap) - config.px_range / 2) as f64;
            uv_p11[0] +=
                (char_gap / 2 + grid_x * (char_width + char_gap) + config.px_range / 2) as f64;
            uv_p11[1] +=
                (char_gap / 2 + grid_y * (char_height + char_gap) + config.px_range / 2) as f64;

            // normalize texture coord UV
            let u0 = uv_p00[0] / img_width as f64;
            let v0 = uv_p00[1] / img_height as f64;
            let u1 = uv_p11[0] / img_width as f64;
            let v1 = uv_p11[1] / img_height as f64;

            for y in 0..char_height {
                for x in 0..char_width {
                    let p = atlas_bitmap.pixel_mut(
                        char_gap / 2 + grid_x * (char_width + char_gap) + x,
                        char_gap / 2 + grid_y * (char_height + char_gap) + char_height - 1 - y,
                    );
                    *p = *bitmap.pixel(x, y);
                }
            }

            glyphs.insert(
                char,
                MtsdfAtlasGlyph {
                    char,
                    metrics,
                    projection: framing.projection,
                    tex_coord_p00: [u0 as f32, v0 as f32],
                    tex_coord_p11: [u1 as f32, v1 as f32],
                },
            );
        }

        Ok(MtsdfAtlas {
            config: config.clone(),
            font_metrics,
            glyphs,
            grid_size: (grid_width, grid_height),
            bitmap: atlas_bitmap,
        })
    }

    pub fn px_range(&self) -> f32 {
        self.config.px_range as f32
    }

    pub fn get_glyph(&self, char: char) -> Option<&MtsdfAtlasGlyph> {
        self.glyphs.get(&char)
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
    use crate::{printable_ascii_chars, MtsdfAtlas, MtsdfAtlasConfig};

    #[test]
    fn test_initialize_msdf() {
        let _ = std::fs::create_dir("tmp");

        let chars: Vec<char> = printable_ascii_chars();
        for name in ["Roboto-Regular", "FiraSans-Regular"] {
            let font_path = format!("assets/fonts/{}.ttf", name);
            let config = MtsdfAtlasConfig::new(font_path);
            let atlas: MtsdfAtlas = MtsdfAtlas::create(&config, &chars).unwrap();
            println!("font {name}: {:?}", atlas.font_metrics);

            let glyph = atlas.get_glyph('f').unwrap();
            println!("char {}: {:?}", glyph.char, glyph.metrics);

            let atlas_img_path = format!("tmp/{}-atlas.png", name);
            atlas.export_bitmap_png(&atlas_img_path).unwrap();
            println!("font {}: export bitmat image {}", name, atlas_img_path);

            let preview_img_path = format!("tmp/{}-preview.png", name);
            atlas.export_preview_png((8, 1), &preview_img_path).unwrap();
            println!("font {}: export preview image {}", name, preview_img_path);
        }
    }
}
