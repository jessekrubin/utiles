use crate::{flipy, pmtiles, xyz2rmid, LngLat, Tile};

pub trait TileLike {
    fn x(&self) -> u32;
    fn y(&self) -> u32;
    fn z(&self) -> u8;

    #[must_use]
    fn zoom(&self) -> u8 {
        self.z()
    }

    fn yflip(&self) -> u32 {
        flipy(self.y(), self.z())
    }

    #[must_use]
    fn tile(&self) -> Tile {
        Tile::new(self.x(), self.y(), self.z())
    }

    /// both bc I keep forgetting which is which
    fn flipy(&self) -> u32 {
        flipy(self.y(), self.z())
    }

    #[must_use]
    fn valid(&self) -> bool {
        crate::valid(self.x(), self.y(), self.z())
    }

    #[must_use]
    fn ul(&self) -> LngLat {
        crate::ul(self.x(), self.y(), self.z())
    }

    #[must_use]
    fn ur(&self) -> LngLat {
        crate::ul(self.x() + 1, self.y(), self.z())
    }

    #[must_use]
    fn lr(&self) -> LngLat {
        crate::ul(self.x() + 1, self.y() + 1, self.z())
    }

    #[must_use]
    fn ll(&self) -> LngLat {
        crate::ul(self.x(), self.y() + 1, self.z())
    }

    #[must_use]
    fn quadkey(&self) -> String {
        crate::xyz2quadkey(self.x(), self.y(), self.z())
    }

    #[must_use]
    fn qk(&self) -> String {
        crate::xyz2quadkey(self.x(), self.y(), self.z())
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
    fn row_major_id(&self) -> u64 {
        xyz2rmid(self.x(), self.y(), self.z())
    }

    #[must_use]
    fn rmid(&self) -> u64 {
        self.row_major_id()
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

    #[must_use]
    fn json_arr(&self) -> String {
        format!("[{}, {}, {}]", self.x(), self.y(), self.z())
    }

    #[must_use]
    fn json_arr_min(&self) -> String {
        format!("[{},{},{}]", self.x(), self.y(), self.z())
    }

    #[must_use]
    fn json(&self) -> String {
        format!(
            "{{\"x\":{}, \"y\":{}, \"z\":{}}}",
            self.x(),
            self.y(),
            self.z()
        )
    }

    #[must_use]
    fn json_obj(&self) -> String {
        self.tile().json_obj()
    }

    #[must_use]
    fn tuple_string(&self) -> String {
        format!("({}, {}, {})", self.x(), self.y(), self.z())
    }

    #[must_use]
    fn mbtiles_sql_where(&self) -> String {
        // classic mbtiles sqlite query:
        // 'SELECT tile_data FROM tiles WHERE zoom_level = ? AND tile_column = ? AND tile_row = ?',

        // flip y for tms (default for mbtiles)
        format!(
            "(zoom_level = {} AND tile_column = {} AND tile_row = {})",
            self.z(),
            self.x(),
            flipy(self.y(), self.z())
        )
    }

    #[must_use]
    fn mbtiles_sql_where_tms(&self) -> String {
        format!(
            "(zoom_level = {} AND tile_column = {} AND tile_row = {})",
            self.z(),
            self.x(),
            self.y()
        )
    }
}
