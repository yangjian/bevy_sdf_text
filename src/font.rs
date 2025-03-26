use std::path::Path;

use anyhow::{Context, Result};
use bevy::prelude::{Asset, Rect, Vec2};
use bevy::reflect::TypePath;
use owned_ttf_parser::{Face, GlyphId};
use serde::{Deserialize, Serialize};

use crate::SdfAtlas;

#[derive(Asset, TypePath, Debug, Serialize, Deserialize)]
pub struct SdfFont {
    pub name: String,
    pub data: Vec<u8>,
    pub metrics: FontMetrics,
    pub atlas: SdfAtlas,
}

impl SdfFont {
    pub fn from_slice(data: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(data).context("deserialize SDF font")
    }

    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read(path).context("read file")?;
        Self::from_slice(&data)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self).context("serialize SDF font")
    }

    pub fn export_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = self.to_vec()?;
        std::fs::write(path, &data).context("write data")?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
        let bbox = font.glyph_bounding_box(glyph_id)?;
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
