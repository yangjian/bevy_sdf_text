use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};

use super::SdfFont;

#[derive(Default)]
pub struct SdfFontLoader;

impl AssetLoader for SdfFontLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let sdf_font = SdfFont::load(Default::default(), bytes.into())?;
            load_context.set_default_asset(LoadedAsset::new(sdf_font));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ttf", "otf"]
    }
}
