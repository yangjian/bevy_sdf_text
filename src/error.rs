use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum MtsdfTextError {
    #[error("font not found")]
    NoSuchFont,

    #[error("glyph not found for char {0}")]
    GlyphNotFound(char),
}
