use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::tasks::ConditionalSendFuture;
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

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let font = SdfFont::from_slice(&bytes)?;
            Ok(font)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["sdfb"]
    }
}
