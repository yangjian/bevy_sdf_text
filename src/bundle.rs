use std::collections::HashSet;

use bevy::asset::HandleId;
use bevy::log;
use bevy::prelude::*;

use crate::{build_sdf_text_mesh, layout::*};
use crate::{PlaneWithUV, SdfRectMaterial, SdfTextMaterial};
use crate::{SdfFont, SdfFontAtlas, SdfFontAtlasRes, SdfText, SdfTextAnchor};

#[derive(Bundle, Clone, Debug, Default)]
pub struct SdfTextBundle {
    /// Text mesh configuration
    pub text: SdfText,

    /// How the text is positioned relative to its transform.
    pub text_anchor: SdfTextAnchor,

    /// Text background.
    pub text_background: SdfTextBackground,

    /// Standard bevy [`Transform`] for positioning the text
    pub transform: Transform,

    /// Standard bevy [`GlobalTransform`]
    pub global_transform: GlobalTransform,

    /// The visibility properties of the text.
    pub visibility: Visibility,

    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering.
    pub computed_visibility: ComputedVisibility,
}

#[derive(Component, Clone, Default, Debug, PartialEq, Reflect)]
pub struct SdfTextBackground {
    pub padding: Vec2,
    pub color: Color,
    pub border_color: Color,
    pub border_size: f32,      // in pixel unit
    pub border_radius: f32,    // in world unit
    pub z_offset: Option<f32>, // default is -0.001
}

impl SdfTextBackground {
    pub fn is_empty(&self) -> bool {
        self.color.a() == 0.0 && !self.has_border()
    }

    pub fn has_border(&self) -> bool {
        self.border_color.a() > 0.0 && self.border_size > 0.0
    }
}

#[derive(Component, Clone, Debug)]
pub(crate) struct SdfTextState {
    layout_output: LayoutOutput,
}

#[derive(Component, Clone, Debug)]
pub(crate) struct SdfTextBackgroundNode {
    entity: Entity,
}

#[derive(Event)]
pub(crate) struct SdfFontAtlasReady(HandleId);

pub(crate) fn update_text_mesh(
    font_atlas_res: Res<SdfFontAtlasRes>,
    _atlas_events: EventReader<SdfFontAtlasReady>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut text_query: Query<(Entity, Ref<SdfText>), Without<SdfTextState>>,
    mut queue: Local<HashSet<Entity>>,
) {
    let mut handle_text = |entity: Entity, text: &SdfText| -> bool {
        let text_atlases: Vec<&SdfFontAtlas> = text
            .sections
            .iter()
            .filter_map(|o| font_atlas_res.font_atlas(o.style.font.id()))
            .collect();
        if text_atlases.len() != text.sections.len() {
            let mut entity_cmds = commands.get_entity(entity).unwrap();
            entity_cmds.clear_children().remove::<SdfTextState>();
            return false;
        }

        let mut layout_input = LayoutInput {
            alignment: text.alignment,
            bounds: text.bounds,
            ..Default::default()
        };

        for (index, section) in text.sections.iter().enumerate() {
            layout_input.fonts.push(text_atlases[index].ab_font.clone());
            layout_input.sections.push(LayoutTextSection {
                value: section.value.clone(),
                style: LayoutTextStyle {
                    font_index: index,
                    font_size: section.style.font_size,
                    color: section.style.color.as_rgba_linear(),
                },
            });
        }

        let layout_output = run_layout(&layout_input);

        let child_id = commands
            .spawn((
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                ComputedVisibility::default(),
            ))
            .with_children(|parent| {
                for (index, section) in layout_output.sections.iter().enumerate() {
                    // TODO: merge meshes with same font
                    let atlas = &text_atlases[index];
                    let mesh = build_sdf_text_mesh(section, &atlas.glyphs);
                    parent.spawn(MaterialMeshBundle {
                        mesh: meshes.add(mesh),
                        material: atlas.material.clone(),
                        ..default()
                    });
                }
            })
            .id();

        let mut entity_cmds = commands.get_entity(entity).unwrap();
        entity_cmds
            .clear_children()
            .add_child(child_id)
            .insert(SdfTextState { layout_output });
        true
    };

    for (entity, text) in &mut text_query {
        if !text.is_changed() && !queue.remove(&entity) {
            continue;
        }

        if !handle_text(entity, &text) {
            queue.insert(entity);
        }
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn update_text_anchor(
    mut commands: Commands,
    mut text_query: Query<
        (&SdfTextAnchor, &SdfTextState, &SdfTextBackground, &Children),
        Or<(Changed<SdfTextAnchor>, Changed<SdfTextState>)>,
    >,
) {
    for (anchor, state, background, children) in &mut text_query {
        let child = match children.first() {
            Some(o) => o,
            None => continue,
        };

        let mut bbox = state.layout_output.bbox;
        if !background.is_empty() {
            bbox.min.x -= background.padding.x;
            bbox.max.x += background.padding.x;
            bbox.min.y -= background.padding.y;
            bbox.max.y += background.padding.y;
        }

        let mut offset = anchor.as_offset(bbox.size());
        offset.x -= bbox.min.x;
        offset.y -= bbox.min.y;

        let mut child_cmds = commands.get_entity(*child).unwrap();
        child_cmds.insert(Transform::from_xyz(offset.x, offset.y, 0.0));
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn update_text_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SdfRectMaterial>>,
    mut text_query: Query<
        (
            &SdfTextBackground,
            &SdfTextState,
            Option<&SdfTextBackgroundNode>,
            &Children,
        ),
        Or<(Changed<SdfTextBackground>, Changed<SdfTextState>)>,
    >,
) {
    for (background, state, bg_node, children) in &mut text_query {
        let child = match children.first() {
            Some(o) => o,
            None => continue,
        };

        let mut rect = state.layout_output.bbox;
        rect.min.x -= background.padding.x;
        rect.max.x += background.padding.x;
        rect.min.y -= background.padding.y;
        rect.max.y += background.padding.y;

        let size = rect.size();

        let visibility = background
            .is_empty()
            .then_some(Visibility::Hidden)
            .unwrap_or_default();

        let mesh = meshes.add(
            PlaneWithUV {
                rect,
                color: background.color,
                p00_uv: (size * -0.5).to_array(),
                p11_uv: (size * 0.5).to_array(),
            }
            .into(),
        );
        let material = materials.add(SdfRectMaterial {
            uniform: crate::SdfRectMaterialUniform {
                size,
                border_radius: background.border_radius,
                border_pixels: background.border_size,
                border_color: background.border_color.as_rgba_f32().into(),
            },
            alpha_mode: AlphaMode::Blend,
        });

        let node = commands
            .spawn(MaterialMeshBundle {
                mesh,
                material,
                visibility,
                transform: Transform::from_xyz(0.0, 0.0, background.z_offset.unwrap_or(-0.001)),
                ..Default::default()
            })
            .id();

        if let Some(node) = bg_node {
            commands.get_entity(node.entity).unwrap().despawn();
        }

        let mut child_cmds = commands.get_entity(*child).unwrap();
        child_cmds
            .add_child(node)
            .insert(SdfTextBackgroundNode { entity: node });
    }
}

/// Auto setup font atlas on SdfFont loaded event
pub(crate) fn setup_font_atlas(
    fonts: Res<Assets<SdfFont>>,
    mut font_events: EventReader<AssetEvent<SdfFont>>,
    mut atlas_events: EventWriter<SdfFontAtlasReady>,
    mut textures: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<SdfTextMaterial>>,
    mut font_atlas_res: ResMut<SdfFontAtlasRes>,
) {
    for event in font_events.into_iter() {
        if let AssetEvent::Created { handle } = event {
            let font = fonts.get(handle).unwrap();
            match font_atlas_res.insert(
                handle.id(),
                &font.face,
                Default::default(), // TODO
                textures.as_mut(),
                materials.as_mut(),
            ) {
                Ok(_) => {
                    log::info!("inserted atlas for font {}", font.name);
                    atlas_events.send(SdfFontAtlasReady(handle.id()));
                }
                Err(err) => log::error!("can't insert altas for font {}: {err}", font.name),
            }
        }
    }
}
