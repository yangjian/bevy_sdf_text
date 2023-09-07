use bevy::core_pipeline::fxaa::Fxaa;
use bevy::prelude::shape::UVSphere;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use bevy_sdf_text::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin, SdfTextPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    assets: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
) {
    let firasans_font = assets.load("fonts/FiraSans-Regular.ttf");
    let roboto_font = assets.load("fonts/MyRobotoSlab-Regular.ttf");

    commands.spawn(SdfTextBundle {
        text: SdfText {
            sections: vec![
                SdfTextSection {
                    value: "Hello, World!\n".into(),
                    style: SdfTextStyle {
                        font: firasans_font,
                        font_size: 2.0,
                        color: Color::SEA_GREEN,
                    },
                },
                SdfTextSection {
                    value: "Good Morning, VAR!".into(),
                    style: SdfTextStyle {
                        font: roboto_font.clone(),
                        font_size: 2.5,
                        color: Color::AQUAMARINE,
                    },
                },
            ],
            alignment: SdfTextAlignment::Right,
            bounds: Some((16.0, f32::INFINITY)),
        },
        text_anchor: SdfTextAnchor::BottomRight,
        text_background: SdfTextBackground {
            padding: Vec2::new(0.5, 0.5),
            color: Color::BLACK,
            border_radius: 0.5,
            border_size: 6.0,
            border_color: Color::WHITE,
            ..Default::default()
        },
        ..Default::default()
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

    let anchors: Vec<SdfTextAnchor> = vec![
        SdfTextAnchor::BottomLeft,
        SdfTextAnchor::BottomCenter,
        SdfTextAnchor::BottomRight,
        SdfTextAnchor::CenterLeft,
        SdfTextAnchor::Center,
        SdfTextAnchor::CenterRight,
        SdfTextAnchor::TopLeft,
        SdfTextAnchor::TopCenter,
        SdfTextAnchor::TopRight,
    ];

    for (i, anchor) in anchors.iter().enumerate() {
        commands.spawn(SdfTextBundle {
            text: SdfText {
                sections: vec![SdfTextSection {
                    value: format!("Hello, World! {anchor:?}"),
                    style: SdfTextStyle {
                        font: roboto_font.clone(),
                        font_size: 2.0,
                        color: Color::SALMON,
                    },
                }],
                alignment: SdfTextAlignment::Left,
                ..Default::default()
            },
            text_anchor: anchor.clone(),
            text_background: SdfTextBackground {
                padding: Vec2::new(0.4, 0.2),
                color: Color::INDIGO,
                border_radius: 0.4,
                border_size: 4.0,
                border_color: Color::DARK_GREEN,
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, -2.5 * (i + 1) as f32, 0.0),
            ..default()
        });

        commands.spawn(MaterialMeshBundle {
            mesh: dot_mesh.clone(),
            material: dot_material.clone(),
            transform: Transform::from_xyz(0.0, -2.5 * (i + 1) as f32, 0.0),
            ..default()
        });
    }

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert((PanOrbitCamera::default(), Fxaa::default()));
}
