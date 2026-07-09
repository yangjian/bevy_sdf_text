use bevy::anti_alias::fxaa::Fxaa;
use bevy::color::palettes::css;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use bevy_sdf_text::*;

#[derive(Resource)]
struct ImmediateTextFont(Handle<SdfFont>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(SdfTextPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_immediate_text)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(ImmediateTextFont(assets.load("fonts/Barlow-Regular.sdfb")));

    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.0, 24.0).looking_at(Vec3::ZERO, Vec3::Y),
            Tonemapping::None,
            Fxaa::default(),
        ))
        .insert(PanOrbitCamera::default());
}

fn draw_immediate_text(mut painter: SdfTextPainter, font: Res<ImmediateTextFont>, time: Res<Time>) {
    let seconds = time.elapsed_secs();
    let base_style = SdfTextStyle {
        font: font.0.clone(),
        font_size: 1.2,
        color: css::AQUAMARINE.into(),
    };

    {
        let config = painter.config_mut();
        config.style = base_style.clone();
        config.anchor = SdfTextAnchor::Center;
        config.transform = Transform::from_xyz(0.0, 2.0, 0.0);
        config.background = SdfTextBackground {
            padding: Vec2::new(0.4, 0.2),
            color: css::INDIGO.into(),
            border_radius: 0.2,
            border_size: 3.0,
            border_color: css::AQUAMARINE.into(),
            ..Default::default()
        };
    }
    painter.draw("Immediate mode text");

    {
        let config = painter.config_mut();
        config.style = SdfTextStyle {
            color: css::SALMON.into(),
            ..base_style.clone()
        };
        config.transform = Transform::from_xyz(0.0, 0.0, 0.0);
        config.background = SdfTextBackground {
            padding: Vec2::new(0.35, 0.15),
            color: css::MIDNIGHT_BLUE.into(),
            border_radius: 0.15,
            border_size: 2.0,
            border_color: css::SALMON.into(),
            ..Default::default()
        };
    }
    painter.draw(&format!("time: {seconds:.2}"));

    {
        let config = painter.config_mut();
        config.style = SdfTextStyle {
            font_size: 0.8,
            color: css::SEA_GREEN.into(),
            ..base_style
        };
        config.transform = Transform::from_xyz(0.0, -2.0, 0.0);
        config.background = SdfTextBackground {
            padding: Vec2::new(0.3, 0.15),
            color: css::BLACK.with_alpha(0.75).into(),
            border_radius: 0.15,
            border_size: 2.0,
            border_color: css::SEA_GREEN.into(),
            ..Default::default()
        };
    }
    painter.draw("drawn every frame with TextPainter");
}
