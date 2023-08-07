use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum SdfTextError {
    #[error("font not found")]
    NoSuchFont,

    #[error("glyph not found for char {0}")]
    GlyphNotFound(char),
}
