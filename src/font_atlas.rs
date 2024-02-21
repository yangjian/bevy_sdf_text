use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use ab_glyph::FontArc;
use anyhow::{anyhow, Result};
use bevy::asset::AssetId;
use bevy::log;
use bevy::prelude::{AlphaMode, Assets, Handle, Resource};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::texture::{Image, ImageSampler};
use msdfgen::{
    Bitmap, Bound, FillRule, FontExt, Framing, Gray, MsdfGeneratorConfig, Projection, Range, Rgba,
    Vector2, MID_VALUE,
};
use owned_ttf_parser::{AsFaceRef, Face, GlyphId, OwnedFace};

use crate::{GlyphMetrics, SdfFont, SdfTextMaterial, SdfTextMaterialUniform};

#[derive(Default, Debug, Resource)]
pub struct SdfFontAtlasRes {
    atlases: HashMap<AssetId<SdfFont>, SdfFontAtlas>,
}

impl SdfFontAtlasRes {
    pub fn font_atlas(&self, handle: AssetId<SdfFont>) -> Option<&SdfFontAtlas> {
        self.atlases.get(&handle)
    }

    pub fn insert(
        &mut self,
        font: AssetId<SdfFont>,
        face: &OwnedFace,
        params: SdfAtlasParams,
        textures: &mut Assets<Image>,
        materials: &mut Assets<SdfTextMaterial>,
    ) -> Result<()> {
        let font_data = face.as_slice().to_owned();
        let ab_font: FontArc = FontArc::try_from_vec(font_data)?;
        let chars = printable_ascii_chars();
        let atlas = SdfAtlas::new(face.as_face_ref(), params, chars)?;

        let mut image = Image::new(
            Extent3d {
                width: atlas.bitmap.width(),
                height: atlas.bitmap.height(),
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            atlas.bitmap.raw_pixels().to_vec(),
            TextureFormat::Rgba32Float,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.sampler = ImageSampler::linear();

        let texture = textures.add(image);
        let material = materials.add(SdfTextMaterial {
            uniform: SdfTextMaterialUniform {
                px_range: atlas.px_range(),
            },
            atlas_texture: texture.clone(),
            alpha_mode: AlphaMode::Blend,
        });

        self.atlases.insert(
            font,
            SdfFontAtlas {
                ab_font,
                params,
                grid_size: atlas.grid_size,
                glyphs: atlas.glyphs,
                texture,
                material,
            },
        );

        Ok(())
    }
}

#[derive(Debug)]
pub struct SdfFontAtlas {
    pub ab_font: FontArc,
    pub params: SdfAtlasParams,
    pub grid_size: (u32, u32),
    pub glyphs: HashMap<char, SdfAtlasGlyph>,
    pub texture: Handle<Image>,
    pub material: Handle<SdfTextMaterial>,
}

#[derive(Clone, Copy, Debug)]
pub struct SdfAtlasParams {
    pub char_size: u32,
    pub char_gap: u32,
    pub px_range: u32,
}

impl Default for SdfAtlasParams {
    fn default() -> Self {
        Self {
            char_size: 64,
            char_gap: 8,
            px_range: 4,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SdfAtlasGlyph {
    pub char: char,
    pub metrics: GlyphMetrics,
    pub projection: Projection<f64>,
    pub margin_ratio: [f32; 2],
    pub tex_coord_p00: [f32; 2],
    pub tex_coord_p11: [f32; 2],
}

#[derive(Clone)]
pub struct SdfAtlas {
    pub params: SdfAtlasParams,
    pub chars: Vec<char>,
    pub grid_size: (u32, u32),
    pub glyphs: HashMap<char, SdfAtlasGlyph>,
    pub bitmap: Bitmap<Rgba<f32>>,
}

impl SdfAtlas {
    pub fn new(face: &Face, params: SdfAtlasParams, chars: Vec<char>) -> Result<Self> {
        let mut gen_config = MsdfGeneratorConfig::default();
        gen_config.set_overlap_support(true);

        let px_range = params.px_range as f64;
        let fill_rule = FillRule::default();

        let (char_width, char_height) = (params.char_size, params.char_size);
        let char_gap = params.char_gap;
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
        let mut atlas_bitmap = Bitmap::<Rgba<f32>>::new(img_width, img_height);

        let mut glyphs = HashMap::new();

        for (i, (char, glyph_id)) in char_glyphs.into_iter().enumerate() {
            if char.is_whitespace() || char.is_ascii_control() {
                continue;
            }

            let grid_x = (i as u32) % grid_width;
            let grid_y = (i as u32) / grid_width;

            let mut shape = match face.glyph_shape(glyph_id) {
                Some(o) => o,
                None => {
                    log::warn!("no glyph shape for char {char:?}");
                    continue;
                }
            };

            let metrics = GlyphMetrics::for_glyph(face, glyph_id)
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

            let margin = params.px_range as f64 / 2.0;

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
                    let p = atlas_bitmap.pixel_mut(
                        char_gap / 2 + grid_x * (char_width + char_gap) + x,
                        char_gap / 2 + grid_y * (char_height + char_gap) + char_height - 1 - y,
                    );
                    *p = *bitmap.pixel(x, y);
                }
            }

            glyphs.insert(
                char,
                SdfAtlasGlyph {
                    char,
                    metrics,
                    projection: framing.projection,
                    margin_ratio,
                    tex_coord_p00: [u0 as f32, v0 as f32],
                    tex_coord_p11: [u1 as f32, v1 as f32],
                },
            );
        }

        Ok(SdfAtlas {
            params,
            chars,
            grid_size: (grid_width, grid_height),
            glyphs,
            bitmap: atlas_bitmap,
        })
    }

    pub fn px_range(&self) -> f32 {
        self.params.px_range as f32
    }

    pub fn get_glyph(&self, char: char) -> Option<&SdfAtlasGlyph> {
        self.glyphs.get(&char)
    }

    pub fn export_bitmap_png<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let output: File = File::create(path.as_ref())?;
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
            .render(&mut preview, self.params.px_range as f64, MID_VALUE);

        let mut output: File = File::create(path.as_ref())?;
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
    use crate::{printable_ascii_chars, SdfAtlas, SdfFont};

    #[test]
    fn test_sdf_atlas() {
        let _ = std::fs::create_dir("tmp");

        for name in ["Roboto-Regular", "Barlow-Regular"] {
            let font_path = format!("assets/fonts/{}.ttf", name);
            let font_data = std::fs::read(&font_path).unwrap();
            let font = SdfFont::new(name.into(), font_data).unwrap();
            let chars = printable_ascii_chars();
            let atlas = SdfAtlas::new(font.face_ref(), Default::default(), chars).unwrap();

            let atlas_img_path = format!("tmp/{}-atlas.png", name);
            atlas.export_bitmap_png(&atlas_img_path).unwrap();
            println!("font {}: export bitmat image {}", name, atlas_img_path);

            let preview_img_path = format!("tmp/{}-preview.png", name);
            atlas.export_preview_png((8, 1), &preview_img_path).unwrap();
            println!("font {}: export preview image {}", name, preview_img_path);
        }
    }
}
