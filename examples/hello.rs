use ab_glyph::FontArc;
use anyhow::Result;
use bevy::core_pipeline::fxaa::Fxaa;
use bevy::prelude::shape::UVSphere;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat};
use bevy::render::texture::ImageSampler;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use bevy_sdf_text::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            MaterialPlugin::<SdfTextMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut texture_images: ResMut<Assets<Image>>,
    mut sdf_materials: ResMut<Assets<SdfTextMaterial>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
) {
    let name = "MyRobotoSlab-Regular";
    let font_path = format!("assets/fonts/{name}.ttf");
    let font = SdfFont::new(name.into(), &font_path, Default::default()).unwrap();
    let chars = printable_ascii_chars();
    let atlas = SdfAtlas::create(font.face(), font.atlas_params, &chars).unwrap();

    let input = prepare_input(&font).unwrap();
    let output = run_layout(&input);

    let text_meshes: Vec<Mesh> = output
        .sections
        .iter()
        .map(|o| build_sdf_text_mesh(o, &atlas))
        .collect();

    let mut atlas_texture = Image::new(
        Extent3d {
            width: atlas.bitmap.width(),
            height: atlas.bitmap.height(),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas.bitmap.raw_pixels().to_vec(),
        TextureFormat::Rgba32Float,
    );
    atlas_texture.sampler_descriptor = ImageSampler::linear();

    let atlas_texture_handle = texture_images.add(atlas_texture);
    let material_handle = sdf_materials.add(SdfTextMaterial {
        uniform: SdfTextMaterialUniform {
            px_range: atlas.px_range(),
            fg_color: Color::WHITE,
            bg_color: Color::NONE,
        },
        atlas_texture: atlas_texture_handle,
        alpha_mode: AlphaMode::Blend,
    });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(PlaneWithUV {
            size: [16.0, 6.0],
            p00_uv: [0.0, 1.0],
            p11_uv: [1.0, 0.0],
        })),
        material: material_handle.clone(),
        transform: Transform::from_xyz(0.0, -3.2, 0.0),
        ..default()
    });

    let dot_mesh = meshes.add(
        UVSphere {
            radius: 0.05,
            sectors: 10,
            stacks: 10,
        }
        .into(),
    );
    let dot_material = std_materials.add(StandardMaterial {
        emissive: Color::WHITE,
        ..Default::default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: dot_mesh.clone(),
        material: dot_material.clone(),
        ..default()
    });

    for section in output.sections.iter() {
        for glyph in section.glyphs.iter() {
            let transform =
                Transform::from_translation(Vec3::new(glyph.position.x, glyph.position.y, 0.0));
            commands.spawn(MaterialMeshBundle {
                mesh: dot_mesh.clone(),
                material: dot_material.clone(),
                transform,
                ..default()
            });
        }
    }

    for mesh in text_meshes {
        commands.spawn(MaterialMeshBundle {
            mesh: meshes.add(mesh),
            material: material_handle.clone(),
            ..default()
        });
    }

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert((PanOrbitCamera::default(), Fxaa::default()));
}

fn prepare_input(font: &SdfFont) -> Result<LayoutInput> {
    let font_data = font.owned_face.as_slice();
    let fonts = vec![FontArc::try_from_vec(font_data.into())?];

    let sections = vec![
        LayoutTextSection::new(
            "Notification: Hello, World!".into(),
            LayoutTextStyle::new(0, 2.0),
        ),
        LayoutTextSection::new("Good Morning, VAR!".into(), LayoutTextStyle::new(0, 2.0)),
    ];

    Ok(LayoutInput {
        fonts,
        sections,
        bounds: Some((16.0, f32::INFINITY)),
        h_alignment: Some(bevy_sdf_text::TextAlignment::Center),
        v_alignment: Some(bevy_sdf_text::TextAlignment::Bottom),
        ..Default::default()
    })
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
