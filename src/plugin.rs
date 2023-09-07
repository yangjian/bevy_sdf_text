use bevy::prelude::*;

use crate::{setup_font_atlas, update_text_anchor, update_text_background, update_text_mesh};
use crate::{SdfFont, SdfFontAtlasReady, SdfFontAtlasRes, SdfFontLoader};
use crate::{SdfRectMaterial, SdfText, SdfTextMaterial, SdfTextSection, SdfTextStyle};

#[derive(Default)]
pub struct SdfTextPlugin;

impl Plugin for SdfTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<SdfFont>()
            .register_type::<SdfText>()
            .register_type::<SdfTextSection>()
            .register_type::<Vec<SdfTextSection>>()
            .register_type::<SdfTextStyle>()
            .init_asset_loader::<SdfFontLoader>()
            .init_resource::<SdfFontAtlasRes>()
            .init_resource::<Events<SdfFontAtlasReady>>()
            .add_plugins(MaterialPlugin::<SdfTextMaterial>::default())
            .add_plugins(MaterialPlugin::<SdfRectMaterial>::default())
            .add_systems(PreUpdate, setup_font_atlas)
            .add_systems(Update, update_text_mesh)
            .add_systems(PostUpdate, (update_text_anchor, update_text_background));
    }
}
