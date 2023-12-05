type Zooms = Vec<u8>;
type ZoomsSetInt = u32;

pub struct ZoomRange {
    pub minz: u8,
    pub maxz: u8,
}

// default zoom range
impl Default for ZoomRange {
    fn default() -> Self {
        Self {
            minz: 0,
            maxz: 32,
        }
    }
}

impl ZoomRange {
    pub fn new(min: u8, max: u8) -> Self {
        Self {
            minz: min,
            maxz: max,
        }
    }

    pub fn from_max(max: u8) -> Self {
        Self {
            minz: 0,
            maxz: max,
        }
    }

    pub fn from_min(min: u8) -> Self {
        Self {
            minz: min,
            maxz: 32,
        }
    }
}


pub fn zset2zvec(zset: ZoomsSetInt) -> Vec<u8> {
    (0..32)
        .rev()
        .filter(|&i| (zset & (1 << i)) != 0)
        .map(|i| (31 - i) as u8) // Convert bit index to zoom level and cast to u8
        .collect()
}

pub fn zvec2zset(zvec: Zooms) -> ZoomsSetInt {
    zvec.iter().fold(0, |acc, &z| acc | (1 << (31 - z)))
}

pub enum ZoomOrZooms {
    Zoom(u8),
    Zooms(Vec<u8>),
}

pub struct ZoomsSet(ZoomsSetInt);



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


impl From<ZoomOrZooms> for Zooms {
    fn from(zoom_or_zooms: ZoomOrZooms) -> Self {
        match zoom_or_zooms {
            ZoomOrZooms::Zoom(zoom) => vec![zoom],
            ZoomOrZooms::Zooms(zooms) => zooms,
        }
    }
}

impl From<ZoomOrZooms> for ZoomsSetInt {
    fn from(zoom_or_zooms: ZoomOrZooms) -> Self {
        match zoom_or_zooms {
            ZoomOrZooms::Zoom(zoom) => 1 << (31 - zoom),
            ZoomOrZooms::Zooms(zooms) => zvec2zset(zooms),
        }
    }
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

    #[test]
    fn zvec2zset_none() {
        let zset: u32 = 0b00000000_00000000_00000000_00000000; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> = vec![];
        assert_eq!(zvec2zset(zvec), zset);
    }

    #[test]
    fn zvec2zset_0_1_2() {
        let zset: u32 = 0b11100000_00000000_00000000_00000000; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> = vec![
            0, 1, 2,
        ];

        assert_eq!(zvec2zset(zvec), zset);
    }

    #[test]
    fn zvec2zset_all() {
        let zset: u32 = 0b11111111_11111111_11111111_11111111; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> =  vec![
            0, 1, 2, 3, 4, 5, 6, 7,
            8, 9,10,11,12,13,14,15,
            16,17,18,19,20,21,22,23,
            24,25,26,27,28,29,30,31,
        ];
        assert_eq!(zvec2zset(zvec), zset);
    }

    #[test]
    fn zvec2zset_0_1_2_3_4_5_6_7() {
        let zset: u32 = 0b11111111_00000000_00000000_00000000; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> = vec![
            0, 1, 2, 3, 4, 5, 6, 7,
        ];
        let zset_from_zvec = zvec2zset(zvec);
        assert_eq!(zset_from_zvec, zset);
    }
}
