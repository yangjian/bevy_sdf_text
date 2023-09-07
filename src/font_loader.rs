use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};

use crate::SdfFont;

#[derive(Default)]
pub struct SdfFontLoader;

impl AssetLoader for SdfFontLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let name = load_context
                .path()
                .file_name()
                .and_then(|o| o.to_str())
                .unwrap_or_default();
            let sdf_font = SdfFont::new(name.to_owned(), bytes.into())?;
            load_context.set_default_asset(LoadedAsset::new(sdf_font));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ttf", "otf"]
    }
}
