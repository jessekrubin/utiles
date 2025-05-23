mod cover;
mod errors;
pub use cover::geojson2tiles;
pub use errors::UtilesCoverError;

#[cfg(test)]
mod tests;

pub type Result<T> = std::result::Result<T, UtilesCoverError>;
