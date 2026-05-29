use bevy::prelude::*;
use glyph_brush_layout::HorizontalAlign;
use serde::{Deserialize, Serialize};

use crate::SdfFont;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Default)]
pub struct SdfText {
    pub sections: Vec<SdfTextSection>,
    /// The text's internal alignment. Should not affect its position within a container.
    pub alignment: SdfTextAlignment,
    pub bounds: Option<(f32, f32)>,
}

impl Default for SdfText {
    fn default() -> Self {
        Self {
            sections: Default::default(),
            alignment: SdfTextAlignment::Left,
            bounds: Default::default(),
        }
    }
}

impl SdfText {
    pub fn from_section(value: impl Into<String>, style: SdfTextStyle) -> Self {
        Self {
            sections: vec![SdfTextSection::new(value, style)],
            ..default()
        }
    }
}

#[derive(Clone, Default, Debug, Reflect)]
pub struct SdfTextSection {
    pub value: String,
    pub style: SdfTextStyle,
}

impl SdfTextSection {
    /// Create a new [`SdfTextSection`].
    pub fn new(value: impl Into<String>, style: SdfTextStyle) -> Self {
        Self {
            value: value.into(),
            style,
        }
    }

    /// Create an empty [`SdfTextSection`] from a style. Useful when the value will be set dynamically.
    pub const fn from_style(style: SdfTextStyle) -> Self {
        Self {
            value: String::new(),
            style,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct SdfTextStyle {
    pub font: Handle<SdfFont>,

    /// The vertical height of rasterized glyphs in world unit.<br/>
    /// The SDF atlas is generated once independent to font size
    pub font_size: f32,

    pub color: Color,
}

impl Default for SdfTextStyle {
    fn default() -> Self {
        Self {
            font: Default::default(),
            font_size: 12.0,
            color: Color::WHITE,
        }
    }
}

/// Describes horizontal alignment preference for positioning & bounds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum SdfTextAlignment {
    /// Leftmost character is immediately to the right of the render position.<br/>
    /// Bounds start from the render position and advance rightwards.
    #[default]
    Left,
    /// Leftmost & rightmost characters are equidistant to the render position.<br/>
    /// Bounds start from the render position and advance equally left & right.
    Center,
    /// Rightmost character is immediately to the left of the render position.<br/>
    /// Bounds start from the render position and advance leftwards.
    Right,
}

impl From<SdfTextAlignment> for HorizontalAlign {
    fn from(val: SdfTextAlignment) -> Self {
        match val {
            SdfTextAlignment::Left => HorizontalAlign::Left,
            SdfTextAlignment::Center => HorizontalAlign::Center,
            SdfTextAlignment::Right => HorizontalAlign::Right,
        }
    }
}

/// How a text is positioned relative to its [`Transform`](bevy_transform::components::Transform).
/// It defaults to `SdfTextAnchor::BottomLeft`.
#[derive(Component, Debug, Clone, Default, Reflect)]
pub enum SdfTextAnchor {
    #[default]
    BottomLeft,
    BottomCenter,
    BottomRight,
    CenterLeft,
    Center,
    CenterRight,
    TopLeft,
    TopCenter,
    TopRight,
    Custom(Vec2),
}

impl SdfTextAnchor {
    pub fn as_offset(&self, size: Vec2) -> Vec2 {
        match self {
            Self::BottomLeft => Vec2::new(0.0, 0.0) * size,
            Self::BottomCenter => Vec2::new(-0.5, 0.0) * size,
            Self::BottomRight => Vec2::new(-1.0, 0.0) * size,
            Self::CenterLeft => Vec2::new(0.0, -0.5) * size,
            Self::Center => Vec2::new(-0.5, -0.5) * size,
            Self::CenterRight => Vec2::new(-1.0, -0.5) * size,
            Self::TopLeft => Vec2::new(0.0, -1.0) * size,
            Self::TopCenter => Vec2::new(-0.5, -1.0) * size,
            Self::TopRight => Vec2::new(-1.0, -1.0) * size,
            Self::Custom(o) => *o,
        }
    }
}
