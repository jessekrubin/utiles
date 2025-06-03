#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
mod cover;
mod cover_geotypes;
mod errors;
pub use cover::geojson2tiles;
pub use cover_geotypes::geometry2tiles;
pub use errors::UtilesCoverError;

#[cfg(test)]
mod tests;

pub type Result<T> = std::result::Result<T, UtilesCoverError>;
