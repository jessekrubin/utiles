use crate::fns;
use crate::lnglat::LngLat;
use crate::pmtiles;
use crate::quadkey;

// pub trait TileXyz{
//     fn x(&self) -> u32;
//     fn y(&self) -> u32;
//     fn z(&self) -> u8;
// }
//

pub trait TileLike {
    #[must_use]
    fn new(x: u32, y: u32, z: u8) -> Self;

    fn x(&self) -> u32;
    fn y(&self) -> u32;
    fn z(&self) -> u8;

    #[must_use]
    fn zoom(&self) -> u8 {
        self.z()
    }

    #[must_use]
    fn valid(&self) -> bool {
        fns::valid(self.x(), self.y(), self.z())
    }

    #[must_use]
    fn ul(&self) -> LngLat {
        fns::ul(self.x(), self.y(), self.z())
    }

    #[must_use]
    fn ur(&self) -> LngLat {
        fns::ul(self.x() + 1, self.y(), self.z())
    }

    #[must_use]
    fn lr(&self) -> LngLat {
        fns::ul(self.x() + 1, self.y() + 1, self.z())
    }

    #[must_use]
    fn ll(&self) -> LngLat {
        fns::ul(self.x(), self.y() + 1, self.z())
    }

    #[must_use]
    fn quadkey(&self) -> String {
        quadkey::xyz2quadkey(self.x(), self.y(), self.z())
    }

    #[must_use]
    fn qk(&self) -> String {
        quadkey::xyz2quadkey(self.x(), self.y(), self.z())
    }

    #[must_use]
    fn pmtileid(&self) -> u64 {
        pmtiles::xyz2pmid(self.x(), self.y(), self.z())
    }

    #[must_use]
    fn pmid(&self) -> u64 {
        self.pmtileid()
    }

    #[must_use]
    fn bbox(&self) -> (f64, f64, f64, f64) {
        let ul = self.ul();
        let lr = self.lr();
        (ul.lng(), lr.lat(), lr.lng(), ul.lat())
    }

    #[must_use]
    fn center(&self) -> LngLat {
        let ul = self.ul();
        let lr = self.lr();
        LngLat::new((ul.lng() + lr.lng()) / 2.0, (ul.lat() + lr.lat()) / 2.0)
    }
}

pub trait BoundingBoxLike {
    fn west(&self) -> f64;
    fn south(&self) -> f64;
    fn east(&self) -> f64;
    fn north(&self) -> f64;

    fn left(&self) -> f64;
    fn bottom(&self) -> f64;
    fn right(&self) -> f64;
    fn top(&self) -> f64;
}

pub trait TLngLat {
    fn lng(&self) -> f64;
    fn lat(&self) -> f64;
}

pub trait TGeoBbox {
    fn north(&self) -> f64;
    fn south(&self) -> f64;
    fn east(&self) -> f64;
    fn west(&self) -> f64;
}

pub trait Utiles<TLngLat, TGeoBbox> {
    // fn ul(&self) -> TLngLat;
    fn ur(&self) -> TLngLat;
    fn lr(&self) -> TLngLat;
    fn ll(&self) -> TLngLat;
    fn bbox(&self) -> TGeoBbox;
}
