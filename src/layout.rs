use ab_glyph::{Font, FontArc, PxScale, PxScaleFont, ScaleFont};
use bevy::prelude::{Rect, Vec2};
use glyph_brush_layout::{
    BuiltInLineBreaker, FontId, GlyphPositioner, HorizontalAlign, Layout, SectionGeometry,
    SectionGlyph, SectionText, VerticalAlign,
};

use crate::GlyphMetrics;

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
    pub v_alignment: Option<TextAlignment>,
    pub bounds: Option<(f32, f32)>,
}

impl LayoutInput {
    fn font_at_section(&self, index: usize) -> &FontArc {
        let font_index = self.sections[index].style.font_index;
        &self.fonts[font_index]
    }

    fn as_layout(&self) -> Layout<BuiltInLineBreaker> {
        let mut layout = Layout::default_wrap().v_align(VerticalAlign::Bottom);
        if let Some(o) = self.h_alignment {
            layout = layout.h_align(o.into());
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
    pub ascent: f32,
    pub descent: f32,
    pub bbox: Rect,
    pub glyphs: Vec<PositionedGlyph>,
}

#[derive(Clone, Debug)]
pub struct PositionedGlyph {
    pub char: char,
    pub position: Vec2,
    pub bbox: Rect,
}

impl PositionedGlyph {
    pub fn bbox_with_margin(&self, ratio: [f32; 2]) -> Rect {
        let center = self.bbox.center();
        let width = self.bbox.width() * (1.0 + ratio[0]);
        let height = self.bbox.height() * (1.0 + ratio[1]);
        Rect {
            min: Vec2 {
                x: center.x - width * 0.5,
                y: center.y - height * 0.5,
            },
            max: Vec2 {
                x: center.x + width * 0.5,
                y: center.y + height * 0.5,
            },
        }
    }
}

#[derive(Clone, Default, Debug)]
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
struct InnerOutput {
    v_alignment: Option<TextAlignment>,
    lines: Vec<InnerLine>,
    sections: Vec<InnerSectionGlyphs>,
}

impl InnerOutput {
    fn new(input: &LayoutInput) -> Self {
        let glyphs = {
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
            layout.calculate_glyphs(&input.fonts, &geometry, &sections)
        };

        let (lines, sections) = {
            let mut lines: Vec<InnerLine> = Vec::new();
            let mut sections: Vec<InnerSectionGlyphs> = Vec::new();

            for glyph in glyphs.into_iter() {
                let section_index = glyph.section_index;
                let baseline = glyph.glyph.position.y;

                let position = glyph.glyph.position;
                if baseline == lines.last().map_or(f32::INFINITY, |o| o.baseline) {
                    let line = lines.last_mut().unwrap();
                    if line.sections.last().unwrap() != &section_index {
                        line.sections.push(section_index);
                    }
                } else {
                    lines.push(InnerLine {
                        baseline: position.y,
                        sections: vec![section_index],
                    });
                }

                if section_index == sections.last().map_or(usize::MAX, |o| o.index) {
                    sections.last_mut().unwrap().glyphs.push(glyph);
                } else {
                    let text = input.sections[section_index].value.clone();
                    let font = input.font_at_section(section_index).clone();
                    sections.push(InnerSectionGlyphs {
                        index: section_index,
                        text,
                        font,
                        glyphs: vec![glyph],
                    });
                }
            }

            (lines, sections)
        };

        Self {
            v_alignment: input.v_alignment,
            lines,
            sections,
        }
    }

    fn line_min_max_y(&self, index: usize) -> Option<(f32, f32)> {
        let line = self.lines.get(index)?;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for i in line.sections.iter() {
            let section = self.sections.get(*i).unwrap();
            let (ascent, descent) = section.ascent_descent();
            min_y = min_y.min(line.baseline + descent);
            max_y = max_y.max(line.baseline + ascent);
        }
        Some((min_y, max_y))
    }

    fn into_final_output(mut self) -> LayoutOutput {
        let (bbox_min_y, bbox_height) = {
            let t0 = self.line_min_max_y(0).unwrap_or_default();
            let t1 = self
                .line_min_max_y(self.lines.len() - 1)
                .unwrap_or_default();
            (t0.0, t1.1 - t0.0)
        };

        let alignment_offset = match self.v_alignment {
            Some(TextAlignment::Bottom) => 0.0,
            Some(TextAlignment::Top) => bbox_height,
            _ => bbox_height * 0.5,
        };

        for (index, line) in self.lines.iter().enumerate() {
            let (min_y, max_y) = self.line_min_max_y(index).unwrap();
            let line_middle = (min_y + max_y) * 0.5;
            let y = bbox_height
                - (line_middle - bbox_min_y)
                - (line_middle - line.baseline)
                - alignment_offset;

            for section_index in line.sections.iter() {
                let section = self.sections.get_mut(*section_index).unwrap();
                for item in section.glyphs.iter_mut() {
                    if item.glyph.position.y == line.baseline {
                        item.glyph.position.y = y;
                    }
                }
            }
        }

        let mut sections = Vec::new();
        let mut bbox = Rect {
            min: Vec2::splat(f32::MAX),
            max: Vec2::splat(f32::MIN),
        };

        for item in self.sections.iter() {
            sections.push(item.layout());
            bbox = bbox.union(item.compute_bbox());
        }
        LayoutOutput { bbox, sections }
    }
}

#[derive(Clone, Debug)]
struct InnerLine {
    pub baseline: f32,
    pub sections: Vec<usize>,
}

#[derive(Clone, Debug)]
struct InnerSectionGlyphs {
    pub index: usize,
    pub text: String,
    pub font: FontArc,
    pub glyphs: Vec<SectionGlyph>,
}

impl InnerSectionGlyphs {
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

    fn ascent_descent(&self) -> (f32, f32) {
        let scaled_font = self.scaled_font();
        (scaled_font.ascent(), scaled_font.descent())
    }

    fn compute_bbox(&self) -> Rect {
        let scaled_font: PxScaleFont<FontArc> = self.scaled_font();
        let (ascent, descent) = (scaled_font.ascent(), scaled_font.descent());

        let mut bbox = Rect {
            min: Vec2::splat(f32::MAX),
            max: Vec2::splat(f32::MIN),
        };

        for item in self.glyphs.iter() {
            let glyph = &item.glyph;
            let bearing_x = scaled_font.h_side_bearing(glyph.id);
            let x_min = glyph.position.x + bearing_x; // TODO: handle kerning
            let x_max = glyph.position.x + scaled_font.h_advance(glyph.id) - bearing_x;

            // NOTE: Y Axis is from bottom to top
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

        let (ascent, descent) = (scaled_font.ascent(), scaled_font.descent());
        let bbox = self.compute_bbox();

        PositionedSection {
            index: self.index,
            ascent,
            descent,
            bbox,
            glyphs,
        }
    }
}

pub fn run_layout(input: &LayoutInput) -> LayoutOutput {
    InnerOutput::new(&input).into_final_output()
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
        let output = run_layout(&input);
        let bbox = output.bbox;
        println!("output bbox: {bbox:?}");
        assert_eq!(bbox.min.y.abs(), bbox.max.y);

        for section in output.sections.iter() {
            let (ascent, descent) = (section.ascent, section.descent);
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
            LayoutTextSection::new("Hello, World! \n".into(), LayoutTextStyle::new(0, 20.0)),
            LayoutTextSection::new("Good morning, VAR!".into(), LayoutTextStyle::new(1, 30.0)),
        ];

        Ok(LayoutInput {
            fonts,
            sections,
            bounds: Some((80.0, f32::INFINITY)),
            h_alignment: Some(TextAlignment::Center),
            ..Default::default()
        })
    }
}
