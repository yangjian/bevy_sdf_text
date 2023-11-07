use bevy::asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext};
use thiserror::Error;

use crate::SdfFont;

#[derive(Default)]
pub struct SdfFontLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SdfFontLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load: {0}")]
    Io(#[from] std::io::Error),

    /// An [IO](std::io) Error
    #[error("Could load: {0}")]
    Parse(#[from] anyhow::Error),
}

impl AssetLoader for SdfFontLoader {
    type Asset = SdfFont;
    type Settings = ();
    type Error = SdfFontLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let name = load_context
                .path()
                .file_name()
                .and_then(|o| o.to_str())
                .unwrap_or_default();

            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let font = SdfFont::new(name.to_owned(), bytes)?;
            Ok(font)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ttf", "otf"]
    }
}
