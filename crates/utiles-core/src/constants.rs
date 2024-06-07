//! Constants module

/// MAX ZOOM (30)
pub const MAX_ZOOM: u8 = 30;

/// MAX ZOOM JS NUMBER (28)
pub const MAX_ZOOM_JS: u8 = 28;

/// Earth radius in meters
pub const EARTH_RADIUS: f64 = 6_378_137.0;

/// Earth circumference in meters
pub const EARTH_CIRCUMFERENCE: f64 = 2.0 * std::f64::consts::PI * EARTH_RADIUS;

/// Episilon for floating point comparison for web mercator
pub const EPSILON: f64 = 1e-14;

/// Episilon for floating point comparison for latlng
pub const LL_EPSILON: f64 = 1e-11;

/// Magic-number/application-id of mbtiles
pub const MBTILES_MAGIC_NUMBER: u32 = 0x4d50_4258;
