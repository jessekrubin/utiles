pub fn xyz2id(x: u32, y: u32, z: u8) -> u64 {
    if z == 0 {
        return 0;
    }
    let base_id: u64 = 1 + (1..z).map(|i| 4u64.pow(u32::from(i))).sum::<u64>();
    let h = fast_hilbert::xy2h(x, y, z);
    base_id + h
}

#[allow(dead_code)]
pub fn zxy2id(z: u8, x: u32, y: u32) -> u64 {
    xyz2id(x, y, z)
}

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

pub fn id2xyz(i: u64) -> (u32, u32, u8) {
    if i == 0 {
        return (0, 0, 0);
    }
    let (i_o, z) = calculate_h_o(i);
    let (x, y) = fast_hilbert::h2xy(i_o, z);
    (x, y, z)
}

#[allow(dead_code)]
pub fn id2zxy(i: u64) -> (u8, u32, u32) {
    let (x, y, z) = id2xyz(i);
    (z, x, y)
}

// Fast parent ID calculation without converting to ZXY (ported from pmtiles go)
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
        assert_eq!(0, zxy2id(0, 0, 0));
        assert_eq!(1, zxy2id(1, 0, 0));
        assert_eq!(2, zxy2id(1, 0, 1));
        assert_eq!(3, zxy2id(1, 1, 1));
        assert_eq!(4, zxy2id(1, 1, 0));
        assert_eq!(5, zxy2id(2, 0, 0));
    }

    #[test]
    fn test_id_to_zxy() {
        let (z, x, y) = id2zxy(0);
        assert_eq!(0, z);
        assert_eq!(0, x);
        assert_eq!(0, y);
        let (z, x, y) = id2zxy(1);
        assert_eq!(1, z);
        assert_eq!(0, x);
        assert_eq!(0, y);
        let (z, x, y) = id2zxy(19078479);
        assert_eq!(12, z);
        assert_eq!(3423, x);
        assert_eq!(1763, y);
    }

    #[test]
    fn test_many_tile_ids() {
        for z in 0..10 {
            for x in 0..(1 << z) {
                for y in 0..(1 << z) {
                    let id = zxy2id(z, x, y);
                    let (rz, rx, ry) = id2zxy(id);
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
            let (z, x, y) = id2zxy(zxy2id(tz, 0, 0));
            assert_eq!(tz, z);
            assert_eq!(0, x);
            assert_eq!(0, y);
            let (z, x, y) = id2zxy(zxy2id(z, dim, 0));
            assert_eq!(tz, z);
            assert_eq!(dim, x);
            assert_eq!(0, y);
            let (z, x, y) = id2zxy(zxy2id(z, 0, dim));
            assert_eq!(tz, z);
            assert_eq!(0, x);
            assert_eq!(dim, y);
            let (z, x, y) = id2zxy(zxy2id(z, dim, dim));
            assert_eq!(tz, z);
            assert_eq!(dim, x);
            assert_eq!(dim, y);
        }
    }

    #[test]
    fn test_parent() {
        assert_eq!(zxy2id(0, 0, 0), parent_id(zxy2id(1, 0, 0)));

        assert_eq!(zxy2id(1, 0, 0), parent_id(zxy2id(2, 0, 0)));
        assert_eq!(zxy2id(1, 0, 0), parent_id(zxy2id(2, 0, 1)));
        assert_eq!(zxy2id(1, 0, 0), parent_id(zxy2id(2, 1, 0)));
        assert_eq!(zxy2id(1, 0, 0), parent_id(zxy2id(2, 1, 1)));

        assert_eq!(zxy2id(1, 0, 1), parent_id(zxy2id(2, 0, 2)));
        assert_eq!(zxy2id(1, 0, 1), parent_id(zxy2id(2, 0, 3)));
        assert_eq!(zxy2id(1, 0, 1), parent_id(zxy2id(2, 1, 2)));
        assert_eq!(zxy2id(1, 0, 1), parent_id(zxy2id(2, 1, 3)));

        assert_eq!(zxy2id(1, 1, 0), parent_id(zxy2id(2, 2, 0)));
        assert_eq!(zxy2id(1, 1, 0), parent_id(zxy2id(2, 2, 1)));
        assert_eq!(zxy2id(1, 1, 0), parent_id(zxy2id(2, 3, 0)));
        assert_eq!(zxy2id(1, 1, 0), parent_id(zxy2id(2, 3, 1)));

        assert_eq!(zxy2id(1, 1, 1), parent_id(zxy2id(2, 2, 2)));
        assert_eq!(zxy2id(1, 1, 1), parent_id(zxy2id(2, 2, 3)));
        assert_eq!(zxy2id(1, 1, 1), parent_id(zxy2id(2, 3, 2)));
        assert_eq!(zxy2id(1, 1, 1), parent_id(zxy2id(2, 3, 3)));
    }
}
