use bevy::prelude::*;

use crate::{SdfFont, SdfFontLoader};

use super::bundle::*;
use super::font_atlas::*;
use super::material::*;
use super::text::*;

#[derive(Default)]
pub struct SdfTextPlugin;

impl Plugin for SdfTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<SdfFont>()
            .register_type::<SdfText>()
            .register_type::<SdfTextSection>()
            .register_type::<Vec<SdfTextSection>>()
            .register_type::<SdfTextStyle>()
            .register_type::<SdfTextBackgroundNodeInfo>()
            .init_asset_loader::<SdfFontLoader>()
            .init_resource::<SdfFontAtlasRes>()
            .init_resource::<Events<SdfFontAtlasReady>>()
            .add_plugins(MaterialPlugin::<SdfTextMaterial>::default())
            .add_plugins(MaterialPlugin::<SdfRectMaterial>::default())
            .add_systems(First, setup_font_atlas)
            .add_systems(PreUpdate, update_text_mesh)
            .add_systems(Update, (update_text_anchor, update_text_background));
    }
}
