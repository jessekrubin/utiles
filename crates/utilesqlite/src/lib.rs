// pub use crate::mbtiles::{Mbtiles, MetadataRow, all_metadata};
// pub mod mbtiles;
// mod mbtiles;
// mod metadata_row;
// mod metadata2tilejson;
pub mod mbtiles;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
