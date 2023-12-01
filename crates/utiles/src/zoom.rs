pub enum ZoomOrZooms {
    Zoom(u8),
    Zooms(Vec<u8>),
}

impl From<u8> for ZoomOrZooms {
    fn from(zoom: u8) -> Self {
        ZoomOrZooms::Zoom(zoom)
    }
}

impl From<Vec<u8>> for ZoomOrZooms {
    fn from(zooms: Vec<u8>) -> Self {
        ZoomOrZooms::Zooms(zooms)
    }
}

type Zooms = Vec<u8>;
type ZoomsSet = u32;

pub fn zset2zvec(zset: ZoomsSet) -> Vec<u8> {
    (0..32)
        .rev()
        .filter(|&i| (zset & (1 << i)) != 0)
        .map(|i| (31 - i) as u8) // Convert bit index to zoom level and cast to u8
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zset2zvec_none() {
        let zset: u32 = 0b00000000_00000000_00000000_00000000; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> = vec![];
        assert_eq!(zset2zvec(zset), zvec);
    }
    #[test]
    fn zset2zvec_0_1_2() {
        let zset: u32 = 0b11100000_00000000_00000000_00000000; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> = vec![
            0, 1, 2,
        ];

        assert_eq!(zset2zvec(zset), zvec);
    }
    #[test]
    fn zset2zvec_all() {
        let zset: u32 = 0b11111111_11111111_11111111_11111111; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> =  vec![
            0, 1, 2, 3, 4, 5, 6, 7,
            8, 9,10,11,12,13,14,15,
            16,17,18,19,20,21,22,23,
            24,25,26,27,28,29,30,31,
        ];
        assert_eq!(zset2zvec(zset), zvec);
    }
}

