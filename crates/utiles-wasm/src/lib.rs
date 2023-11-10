use utiles::pmtiles::xyz2pmid as _xyz2pmid;
use utiles::xyz2quadkey as uxyz2qk;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[wasm_bindgen]
pub fn pmtileid(x: u32, y: u32, z: u8) -> u64 {
    _xyz2pmid(x, y, z)
}

#[wasm_bindgen]
pub fn xyz2qk(x: u32, y: u32, z: u8) -> String {
    uxyz2qk(x, y, z)
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
