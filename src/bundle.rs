use std::collections::{HashMap, HashSet};

use ab_glyph::FontArc;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

use crate::layout::*;
use crate::SdfAtlas;
use crate::SdfFont;
use crate::SdfTextMaterial;

#[derive(Bundle, Clone, Debug, Default)]
pub struct SdfTextBundle {
    /// Text mesh configuration
    pub text: SdfText,

    /// Standard bevy [`Transform`] for positioning the text
    pub transform: Transform,

    /// Standard bevy [`GlobalTransform`]
    pub global_transform: GlobalTransform,

    /// The visibility properties of the text.
    pub visibility: Visibility,

    /// Internal state, no public API
    pub sdf_text_state: SdfTextState,
}

#[derive(Component, Clone, Default, Debug)]
// #[reflect(Component, Default)]
pub struct SdfText {
    pub sections: Vec<SdfTextSection>,
    pub horizontal_alignment: Option<crate::TextAlignment>,
    pub bounds: Option<(f32, f32)>,
}

impl SdfText {
    pub const fn with_horizontal_alignment(mut self, alignment: crate::TextAlignment) -> Self {
        self.horizontal_alignment = Some(alignment);
        self
    }
}

#[derive(Clone, Default, Debug)]
pub struct SdfTextSection {
    pub text: String,
    pub font: Handle<SdfFont>,
    pub font_size: f32,
}

#[derive(Component, Clone, Default, Debug)]
pub struct SdfTextState {
    pub(crate) ready_to_layout: bool,
    pub(crate) layout_output: Option<LayoutOutput>,
}

#[derive(Default, Resource)]
pub struct SdfTextPipeline {}

/// Updates the layout and size information whenever the text or style is changed.
/// This information is computed by the `TextPipeline` on insertion, then stored.
///
/// ## World Resources
///
/// [`ResMut<Assets<Image>>`](Assets<Image>) -- This system only adds new [`Image`] assets.
/// It does not modify or observe existing ones.
#[allow(clippy::too_many_arguments)]
pub fn update_text(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut textures: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<SdfTextMaterial>>,
    mut fonts: ResMut<Assets<SdfFont>>,
    mut text_pipeline: ResMut<SdfTextPipeline>,
    mut text_query: Query<
        (Entity, Ref<SdfText>, &mut SdfTextState),
        Or<(Changed<SdfText>, Changed<SdfTextState>)>,
    >,
    mut queue: Local<HashSet<Entity>>,
) {
    'outer: for (entity, text, mut state) in &mut text_query {
        if !text.is_changed() || !queue.remove(&entity) {
            continue;
        }

        if text.is_changed() {
            state.ready_to_layout = text.sections.iter().all(|o| fonts.get(&o.font).is_some());
        }

        if !state.ready_to_layout {
            queue.insert(entity);
            continue;
        }

        let mut layout_input = LayoutInput {
            h_alignment: text.horizontal_alignment,
            bounds: text.bounds,
            ..Default::default()
        };

        for section in text.sections.iter() {
            let font = match fonts.get(&section.font).map(|o| o.ab_font.clone()) {
                Some(o) => o,
                None => {
                    state.ready_to_layout = false;
                    queue.insert(entity);
                    continue 'outer;
                }
            };
            layout_input.fonts.push(font);
            layout_input.sections.push(LayoutTextSection {
                value: section.text.clone(),
                style: LayoutTextStyle {
                    font_index: layout_input.fonts.len() - 1,
                    font_size: section.font_size,
                },
            });
        }

        let layout_output = run_layout(&layout_input);
        state.layout_output = Some(layout_output);
    }
}

fn build_sdf_text_mesh(section: &PositionedSection, atlas: &SdfAtlas) -> Mesh {
    let n = section.glyphs.len();
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n * 4);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n * 4);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n * 4);
    let mut indices: Vec<u16> = Vec::with_capacity(n * 6);

    let up = Vec3::Y.to_array();

    for glyph in section.glyphs.iter() {
        let atlas_glyph = match atlas.get_glyph(glyph.char) {
            Some(o) => o,
            None => continue,
        };

        let n = positions.len() as u16;

        let bbox = glyph.bbox_with_margin(atlas_glyph.margin_ratio);
        let (x_min, x_max) = (bbox.min.x, bbox.max.x);
        let (y_min, y_max) = (bbox.min.y, bbox.max.y);
        positions.push([x_min, y_min, 0.0]);
        positions.push([x_max, y_min, 0.0]);
        positions.push([x_max, y_max, 0.0]);
        positions.push([x_min, y_max, 0.0]);

        uvs.push([atlas_glyph.tex_coord_p00[0], atlas_glyph.tex_coord_p11[1]]);
        uvs.push(atlas_glyph.tex_coord_p11);
        uvs.push([atlas_glyph.tex_coord_p11[0], atlas_glyph.tex_coord_p00[1]]);
        uvs.push(atlas_glyph.tex_coord_p00);

        for _ in 0..4 {
            normals.push(up);
        }

        indices.push(n);
        indices.push(n + 1);
        indices.push(n + 2);
        indices.push(n);
        indices.push(n + 2);
        indices.push(n + 3);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U16(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
