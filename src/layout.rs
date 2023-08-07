use ab_glyph::{Font, FontArc, PxScale, PxScaleFont, ScaleFont};
use anyhow::Result;
use bevy::prelude::{Rect, Vec2};
use glyph_brush_layout::{
    BuiltInLineBreaker, FontId, GlyphPositioner, HorizontalAlign, Layout, SectionGeometry,
    SectionGlyph, SectionText, VerticalAlign,
};

use super::GlyphMetrics;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Top,
    Bottom,
}

impl From<TextAlignment> for HorizontalAlign {
    fn from(value: TextAlignment) -> Self {
        match value {
            TextAlignment::Right => HorizontalAlign::Right,
            TextAlignment::Center => HorizontalAlign::Center,
            _ => HorizontalAlign::Left,
        }
    }
}

impl From<TextAlignment> for VerticalAlign {
    fn from(value: TextAlignment) -> Self {
        match value {
            TextAlignment::Top => VerticalAlign::Top,
            TextAlignment::Center => VerticalAlign::Center,
            _ => VerticalAlign::Bottom,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutTextStyle {
    pub font_index: usize,
    pub font_size: f32,
}

impl LayoutTextStyle {
    pub fn new(font_index: usize, font_size: f32) -> Self {
        Self {
            font_index,
            font_size,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LayoutTextSection {
    pub value: String,
    pub style: LayoutTextStyle,
}

impl LayoutTextSection {
    pub fn new(value: String, style: LayoutTextStyle) -> Self {
        Self { value, style }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LayoutInput {
    pub fonts: Vec<FontArc>,
    pub sections: Vec<LayoutTextSection>,

    pub h_alignment: Option<TextAlignment>,
    pub v_alignment: Option<TextAlignment>, // NOT supported yet
    pub bounds: Option<(f32, f32)>,
}

impl LayoutInput {
    fn font_at_section(&self, index: usize) -> &FontArc {
        let font_index = self.sections[index].style.font_index;
        &self.fonts[font_index]
    }

    fn as_layout(&self) -> Layout<BuiltInLineBreaker> {
        let mut layout = Layout::default_wrap();
        if let Some(o) = self.h_alignment {
            layout = layout.h_align(o.into());
        }
        if let Some(o) = self.v_alignment {
            layout = layout.v_align(o.into());
        }
        layout
    }

    fn as_geometry(&self) -> SectionGeometry {
        SectionGeometry {
            screen_position: (0.0, 0.0),
            bounds: self.bounds.unwrap_or((f32::INFINITY, f32::INFINITY)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PositionedSection {
    pub index: usize,
    pub ascent_descent: (f32, f32),
    pub bbox: Rect,
    pub glyphs: Vec<PositionedGlyph>,
}

#[derive(Clone, Debug)]
pub struct PositionedGlyph {
    pub char: char,
    pub position: Vec2,
    pub bbox: Rect,
}

#[derive(Clone, Debug)]
pub struct LayoutOutput {
    pub bbox: Rect,
    pub sections: Vec<PositionedSection>,
}

impl LayoutOutput {
    pub fn size(&self) -> Vec2 {
        let width = self.bbox.width();
        let height = self.bbox.height();
        Vec2::new(width, height)
    }
}

#[derive(Clone, Debug)]
struct RawSectionGlyphs {
    pub index: usize,
    pub text: String,
    pub font: FontArc,
    pub glyphs: Vec<SectionGlyph>,
}

impl RawSectionGlyphs {
    fn groups(input: &LayoutInput, glyphs: Vec<SectionGlyph>) -> Vec<Self> {
        let mut sections: Vec<Self> = Vec::new();
        for glyph in glyphs {
            let index = glyph.section_index;
            if index == sections.last().map_or(usize::MAX, |o| o.index) {
                sections.last_mut().unwrap().glyphs.push(glyph);
            } else {
                let text = input.sections[index].value.clone();
                let font = input.font_at_section(index).clone();
                sections.push(Self {
                    index,
                    text,
                    font,
                    glyphs: vec![glyph],
                });
            }
        }
        sections
    }

    fn font_scale(&self) -> PxScale {
        self.glyphs
            .first()
            .map_or_else(|| PxScale { x: 0.0, y: 0.0 }, |o| o.glyph.scale)
    }

    fn scaled_font(&self) -> PxScaleFont<FontArc> {
        PxScaleFont {
            font: self.font.clone(),
            scale: self.font_scale(),
        }
    }

    fn char_at_byte_index(&self, byte_index: usize) -> Option<char> {
        self.text
            .char_indices()
            .find_map(|o| (o.0 == byte_index).then_some(o.1))
    }

    fn compute_bbox(&self) -> Rect {
        let scaled_font = self.scaled_font();
        let ascent = scaled_font.ascent();
        let descent = scaled_font.descent();

        let mut bbox = Rect {
            min: Vec2::splat(f32::MAX),
            max: Vec2::splat(f32::MIN),
        };

        for item in self.glyphs.iter() {
            let glyph = &item.glyph;
            let x_min = glyph.position.x;
            let x_max = x_min + scaled_font.h_advance(glyph.id); // TODO: handle kerning

            // NOTE: Y Axis is assumbed to be from bottom to top
            let rect = Rect {
                min: Vec2::new(x_min, glyph.position.y + descent),
                max: Vec2::new(x_max, glyph.position.y + ascent),
            };
            bbox = bbox.union(rect);
        }
        bbox
    }

    fn layout(&self) -> PositionedSection {
        let scaled_font = self.scaled_font();
        let scale = Vec2::new(scaled_font.h_scale_factor(), scaled_font.v_scale_factor());
        let ascent_descent = (scaled_font.ascent(), scaled_font.descent());

        let section_bbox = self.compute_bbox();

        let mut glyphs = Vec::new();
        for glyph in self.glyphs.iter() {
            let char = match self.char_at_byte_index(glyph.byte_index) {
                Some(o) => o,
                None => continue,
            };
            let glyph_bbox = match get_glyph_metrics(&self.font, char) {
                Some(o) => o,
                None => continue,
            };

            let (pos_x, pos_y) = (glyph.glyph.position.x, glyph.glyph.position.y);
            let position = Vec2::new(pos_x, pos_y);
            let bbox = glyph_bbox.transformed_bbox(position, scale);

            glyphs.push(PositionedGlyph {
                char,
                position,
                bbox,
            })
        }

        PositionedSection {
            index: self.index,
            ascent_descent,
            bbox: section_bbox,
            glyphs,
        }
    }
}

pub fn run_layout(input: &LayoutInput) -> Result<LayoutOutput> {
    let mut sections: Vec<SectionText> = Vec::with_capacity(input.sections.len());

    for section in input.sections.iter() {
        let font_index = section.style.font_index;
        let scale = PxScale::from(section.style.font_size);
        sections.push(SectionText {
            text: &section.value,
            font_id: FontId(font_index),
            scale,
        });
    }

    let layout = input.as_layout();
    let geometry = input.as_geometry();
    let mut glyphs = layout.calculate_glyphs(&input.fonts, &geometry, &sections);
    fix_section_glyph_position_y(&mut glyphs);

    let groups = RawSectionGlyphs::groups(input, glyphs);

    let mut bbox = Rect {
        min: Vec2::splat(f32::MAX),
        max: Vec2::splat(f32::MIN),
    };

    let mut sections = Vec::new();
    for group in groups {
        let section = group.layout();
        bbox = bbox.union(section.bbox);
        sections.push(section);
    }
    Ok(LayoutOutput { bbox, sections })
}

fn fix_section_glyph_position_y(glyphs: &mut [SectionGlyph]) {
    let offset = match glyphs.first() {
        Some(o) => o.glyph.position.y,
        None => return,
    };

    for item in glyphs {
        item.glyph.position.y -= offset;
    }
}

fn get_glyph_metrics(font: &FontArc, char: char) -> Option<GlyphMetrics> {
    let id = font.glyph_id(char);
    let outline = font.outline(id)?;
    let advance = font.h_advance_unscaled(id);
    let bearing = font.h_side_bearing_unscaled(id);
    let bbox = outline.bounds;
    Some(GlyphMetrics {
        advance,
        bearing,
        x_min: bbox.min.x,
        x_max: bbox.max.x,
        y_min: bbox.min.y,
        y_max: bbox.max.y,
    })
}

#[cfg(test)]
mod tests {
    use ab_glyph::FontArc;
    use anyhow::Result;

    use super::{run_layout, LayoutInput, LayoutTextSection, LayoutTextStyle, TextAlignment};

    #[test]
    fn test_text_layout() {
        let input = prepare_input().unwrap();
        let output = run_layout(&input).unwrap();
        println!("output bbox: {:?}", output.bbox);
        for section in output.sections.iter() {
            let (ascent, descent) = section.ascent_descent;
            println!(
                "section {}: ascent {ascent}, descent {descent}, bbox {:?}",
                section.index, section.bbox
            );
            for glyph in section.glyphs.iter() {
                println!(
                    "  char {:?}: position {:?}, bbox {:?}",
                    glyph.char, glyph.position, glyph.bbox
                );
            }
        }
    }

    fn prepare_input() -> Result<LayoutInput> {
        let mut fonts = Vec::new();
        for name in ["Roboto-Regular", "FiraSans-Regular"] {
            let path = format!("assets/fonts/{name}.ttf");
            let data: Vec<u8> = std::fs::read(&path)?;
            fonts.push(FontArc::try_from_vec(data)?);
        }

        let sections = vec![
            LayoutTextSection::new("Hello, World! ".into(), LayoutTextStyle::new(0, 20.0)),
            LayoutTextSection::new("Good morning, VAR!".into(), LayoutTextStyle::new(1, 30.0)),
        ];

        Ok(LayoutInput {
            fonts,
            sections,
            bounds: Some((120.0, f32::INFINITY)),
            h_alignment: Some(TextAlignment::Center),
            ..Default::default()
        })
    }
}
