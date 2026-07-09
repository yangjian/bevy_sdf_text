use bevy::asset::load_internal_asset;
use bevy::prelude::*;

use crate::{SdfFont, SdfFontLoader};

use super::font_atlas::*;
use super::material::*;
use super::systems::*;
use super::text::*;

#[derive(Default)]
pub struct SdfTextPlugin;

impl Plugin for SdfTextPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SDF_TEXT_SHADER_HANDLE,
            "shaders/sdf_text_material.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SDF_RECT_SHADER_HANDLE,
            "shaders/sdf_rect_material.wgsl",
            Shader::from_wgsl
        );

        app.init_asset::<SdfFont>()
            .register_type::<SdfText>()
            .register_type::<SdfTextSection>()
            .register_type::<Vec<SdfTextSection>>()
            .register_type::<SdfTextStyle>()
            .register_type::<SdfTextBackgroundNodeInfo>()
            .init_asset_loader::<SdfFontLoader>()
            .init_resource::<SdfFontAtlasRes>()
            .init_resource::<Messages<SdfFontAtlasReady>>()
            .add_plugins(MaterialPlugin::<SdfTextMaterial>::default())
            .add_plugins(MaterialPlugin::<SdfRectMaterial>::default())
            .add_systems(First, setup_font_atlas)
            .add_systems(
                PreUpdate,
                (before_sdf_update_text_mesh, update_text_mesh).chain(),
            )
            .add_systems(Update, (update_text_anchor, update_text_background));
    }
}
