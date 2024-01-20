#[must_use]
pub fn xyz2pmid(x: u32, y: u32, z: u8) -> u64 {
    if z == 0 {
        return 0;
    }
    let base_id: u64 = (4u64.pow(u32::from(z)) - 1) / 3;
    let h = fast_hilbert::xy2h(x, y, z);
    base_id + h
}

#[allow(dead_code)]
#[must_use]
pub fn zxy2pmid(z: u8, x: u32, y: u32) -> u64 {
    xyz2pmid(x, y, z)
}

/// Calculate the index of the tile for the zoom level as well as the zoom level.
///
/// h stands for
#[must_use]
pub fn calculate_h_o(i: u64) -> (u64, u8) {
    if i == 0 {
        return (0, 0);
    }
    let mut acc: u64 = 0;
    let mut z: u8 = 0;
    loop {
        let num_tiles: u64 = (1 << z) * (1 << z);
        if acc + num_tiles > i {
            return (i - acc, z);
        }
        acc += num_tiles;
        z += 1;
    }
}

#[must_use]
pub fn pmid2xyz(i: u64) -> (u32, u32, u8) {
    if i == 0 {
        return (0, 0, 0);
    }
    let (i_o, z) = calculate_h_o(i);
    let (x, y) = fast_hilbert::h2xy(i_o, z);
    (x, y, z)
}

#[allow(dead_code)]
#[must_use]
pub fn pmid2zxy(i: u64) -> (u8, u32, u32) {
    let (x, y, z) = pmid2xyz(i);
    (z, x, y)
}

// Fast parent ID calculation without converting to ZXY (ported from pmtiles go)
#[must_use]
pub fn parent_id(i: u64) -> u64 {
    let mut acc: u64 = 0;
    let mut last_acc: u64 = 0;
    let mut z: u8 = 0;
    loop {
        let num_tiles: u64 = (1 << z) * (1 << z);
        if acc + num_tiles > i {
            return last_acc + (i - acc) / 4;
        }
        last_acc = acc;
        acc += num_tiles;
        z += 1;
    }
}

// Tests ported from pmtiles go with the help of good old sed and 10 mins of checking it by hand!
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zxy_to_id() {
        assert_eq!(0, zxy2pmid(0, 0, 0));
        assert_eq!(1, zxy2pmid(1, 0, 0));
        assert_eq!(2, zxy2pmid(1, 0, 1));
        assert_eq!(3, zxy2pmid(1, 1, 1));
        assert_eq!(4, zxy2pmid(1, 1, 0));
        assert_eq!(5, zxy2pmid(2, 0, 0));
    }

    #[test]
    fn test_id_to_zxy() {
        let (z, x, y) = pmid2zxy(0);
        assert_eq!(0, z);
        assert_eq!(0, x);
        assert_eq!(0, y);
        let (z, x, y) = pmid2zxy(1);
        assert_eq!(1, z);
        assert_eq!(0, x);
        assert_eq!(0, y);
        let (z, x, y) = pmid2zxy(19_078_479);
        assert_eq!(12, z);
        assert_eq!(3423, x);
        assert_eq!(1763, y);
    }

    #[test]
    fn test_many_tile_ids() {
        for z in 0..10 {
            for x in 0..(1 << z) {
                for y in 0..(1 << z) {
                    let id = zxy2pmid(z, x, y);
                    let (rz, rx, ry) = pmid2zxy(id);
                    assert_eq!(z, rz);
                    assert_eq!(x, rx);
                    assert_eq!(y, ry);
                }
            }
        }
    }

    #[test]
    fn test_extremes() {
        for tz in 0..32 {
            let dim = (1 << tz) - 1;
            let (z, x, y) = pmid2zxy(zxy2pmid(tz, 0, 0));
            assert_eq!(tz, z);
            assert_eq!(0, x);
            assert_eq!(0, y);
            let (z, x, y) = pmid2zxy(zxy2pmid(z, dim, 0));
            assert_eq!(tz, z);
            assert_eq!(dim, x);
            assert_eq!(0, y);
            let (z, x, y) = pmid2zxy(zxy2pmid(z, 0, dim));
            assert_eq!(tz, z);
            assert_eq!(0, x);
            assert_eq!(dim, y);
            let (z, x, y) = pmid2zxy(zxy2pmid(z, dim, dim));
            assert_eq!(tz, z);
            assert_eq!(dim, x);
            assert_eq!(dim, y);
        }
    }

    #[test]
    fn test_parent() {
        assert_eq!(zxy2pmid(0, 0, 0), parent_id(zxy2pmid(1, 0, 0)));

        assert_eq!(zxy2pmid(1, 0, 0), parent_id(zxy2pmid(2, 0, 0)));
        assert_eq!(zxy2pmid(1, 0, 0), parent_id(zxy2pmid(2, 0, 1)));
        assert_eq!(zxy2pmid(1, 0, 0), parent_id(zxy2pmid(2, 1, 0)));
        assert_eq!(zxy2pmid(1, 0, 0), parent_id(zxy2pmid(2, 1, 1)));

        assert_eq!(zxy2pmid(1, 0, 1), parent_id(zxy2pmid(2, 0, 2)));
        assert_eq!(zxy2pmid(1, 0, 1), parent_id(zxy2pmid(2, 0, 3)));
        assert_eq!(zxy2pmid(1, 0, 1), parent_id(zxy2pmid(2, 1, 2)));
        assert_eq!(zxy2pmid(1, 0, 1), parent_id(zxy2pmid(2, 1, 3)));

        assert_eq!(zxy2pmid(1, 1, 0), parent_id(zxy2pmid(2, 2, 0)));
        assert_eq!(zxy2pmid(1, 1, 0), parent_id(zxy2pmid(2, 2, 1)));
        assert_eq!(zxy2pmid(1, 1, 0), parent_id(zxy2pmid(2, 3, 0)));
        assert_eq!(zxy2pmid(1, 1, 0), parent_id(zxy2pmid(2, 3, 1)));

        assert_eq!(zxy2pmid(1, 1, 1), parent_id(zxy2pmid(2, 2, 2)));
        assert_eq!(zxy2pmid(1, 1, 1), parent_id(zxy2pmid(2, 2, 3)));
        assert_eq!(zxy2pmid(1, 1, 1), parent_id(zxy2pmid(2, 3, 2)));
        assert_eq!(zxy2pmid(1, 1, 1), parent_id(zxy2pmid(2, 3, 3)));
    }
}
