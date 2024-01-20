/// A geotransform is an affine transformation from the image coordinate space
/// (row, column), also known as (pixel, line) to the georeferenced coordinate
/// space (projected or geographic coordinates).
///
/// A geotransform consists in a set of 6 coefficients:
///
/// GT(0) x-coordinate of the upper-left corner of the upper-left pixel.
/// GT(1) w-e pixel resolution / pixel width.
/// GT(2) row rotation (typically zero).
/// GT(3) y-coordinate of the upper-left corner of the upper-left pixel.
/// GT(4) column rotation (typically zero).
/// GT(5) n-s pixel resolution / pixel height (negative value for a north-up image).
pub struct GeoTransform {
    /// x-coordinate of the upper-left corner of the upper-left pixel.
    pub gt0: f64,
    /// w-e pixel resolution / pixel width.
    pub gt1: f64,
    /// row rotation (typically zero).
    pub gt2: f64,
    /// y-coordinate of the upper-left corner of the upper-left pixel.
    pub gt3: f64,
    /// column rotation (typically zero).
    pub gt4: f64,
    /// n-s pixel resolution / pixel height (negative value for a north-up image).
    pub gt5: f64,
}

impl GeoTransform {
    #[must_use]
    pub fn new(gt0: f64, gt1: f64, gt2: f64, gt3: f64, gt4: f64, gt5: f64) -> Self {
        GeoTransform {
            gt0,
            gt1,
            gt2,
            gt3,
            gt4,
            gt5,
        }
    }

    #[must_use]
    pub fn optzoom(&self) -> u8 {
        let equator = 2.0 * std::f64::consts::PI * 6_378_137.0; // 2 * pi * radius of earth in meters
        let resolution = self.gt1 * (equator / 360.0);
        let zoom_level = (equator / 256.0) / resolution; // Assuming pixel_size is 256 as in the previous example
        (zoom_level.log2().min(20.0).floor() + 0.5) as u8
    }
}

impl From<(f64, f64, f64, f64, f64, f64)> for GeoTransform {
    fn from(gt: (f64, f64, f64, f64, f64, f64)) -> Self {
        GeoTransform::new(gt.0, gt.1, gt.2, gt.3, gt.4, gt.5)
    }
}

#[must_use]
pub fn geotransform2optzoom(geotransform: (f64, f64, f64, f64, f64, f64)) -> u8 {
    let gt = GeoTransform::from(geotransform);
    gt.optzoom()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geotransform2optzoom() {
        let optz = geotransform2optzoom((
            -77.000138, 0.000278, 0.0, 26.0001389, 0.0, -0.000278,
        ));
        assert_eq!(optz, 12);
    }
}
