use anyhow::{anyhow, Result};
use bevy::prelude::{Rect, Vec2};
use bevy::reflect::{TypePath, TypeUuid};
use owned_ttf_parser::{AsFaceRef, Face, GlyphId, OwnedFace};

#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "82b51e5b-f557-46cb-a7aa-1d35aff6f403"]
pub struct SdfFont {
    pub name: String,
    pub face: OwnedFace,
    pub metrics: FontMetrics,
}

impl SdfFont {
    pub fn new(name: String, data: Vec<u8>) -> Result<Self> {
        let face: OwnedFace = OwnedFace::from_vec(data, 0)?;
        let metrics = FontMetrics::new(face.as_face_ref())
            .ok_or_else(|| anyhow!("unable to get font metrics"))?;

        Ok(Self {
            name,
            face,
            // ab_font,
            metrics,
        })
    }

    pub fn face_ref(&self) -> &Face {
        self.face.as_face_ref()
    }

    pub fn data(&self) -> Vec<u8> {
        self.face.as_slice().into()
    }
}

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

#[cfg(test)]
mod tests {
    use owned_ttf_parser::PreParsedSubtables;
    use ttf_parser::gpos::{PairAdjustment, PositioningSubtable, SingleAdjustment};

    use crate::SdfFont;

    #[test]
    fn test_font() {
        for name in [
            "Roboto-Regular",
            "OpenSans-Regular",
            "FiraSans-Regular",
            "MyRobotoSlab-Regular",
        ] {
            let font_path = format!("assets/fonts/{}.ttf", name);
            let font_data = std::fs::read(&font_path).unwrap();
            let font = SdfFont::new(name.into(), font_data).unwrap();
            println!("font {}: {:?}", font.name, font.metrics);

            let tables = font.face_ref().tables();
            println!(
                "gpos: {}, kern {}",
                tables.gpos.is_some(),
                tables.kern.is_some(),
            );

            let tables = font.face_ref().tables();
            let gpos = tables.gpos.as_ref().unwrap();
            for lookup in gpos.lookups {
                for item in lookup.subtables.into_iter::<PositioningSubtable>() {
                    match item {
                        PositioningSubtable::Single(_) => println!("single table"),
                        PositioningSubtable::Pair(_) => println!("pair table"),
                        PositioningSubtable::Cursive(_) => println!("cursive table"),
                        PositioningSubtable::Context(_) => println!("context table"),
                        PositioningSubtable::ChainContext(_) => println!("chain context table"),
                        _ => println!("unknown table"),
                    };

                    if let PositioningSubtable::Single(s) = item {
                        match s {
                            SingleAdjustment::Format1 { coverage: _, value } => {
                                println!("single v1: {:?}", value)
                            }
                            SingleAdjustment::Format2 {
                                coverage: _,
                                values,
                            } => {
                                println!("single v2: {:?}", values)
                            }
                        }
                    } else if let PositioningSubtable::Pair(p) = item {
                        match p {
                            PairAdjustment::Format1 {
                                coverage: _,
                                sets: _,
                            } => {
                                println!("pair v1")
                            }
                            PairAdjustment::Format2 {
                                coverage: _,
                                classes: _,
                                matrix: _,
                            } => println!("pair v2"),
                        }
                    }
                }
            }

            let faster_face: PreParsedSubtables<'_, _> = PreParsedSubtables::from(font.face);
            let g0 = faster_face.glyph_index('V').unwrap();
            let g1 = faster_face.glyph_index('A').unwrap();
            let kern = faster_face.glyphs_hor_kerning(g0, g1);
            println!("VA kern: {kern:?}");
        }
    }
}
