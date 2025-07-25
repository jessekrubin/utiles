use crate::bbox::WebBBox;
use crate::{
    BBox, LngLat, Tile, TileZBox, flipy, neighbors, neighbors_wrap_x, xyz2rmid,
};

#[cfg(feature = "pmtiles")]
use crate::pmtiles;

/// Trait def for tile-like objects/structs/things/whatevers
pub trait TileLike {
    /// x coordinate (column)
    fn x(&self) -> u32;

    /// y coordinate (row -- flipped for TMS)
    fn y(&self) -> u32;

    /// z coordinate (zoom level)
    fn z(&self) -> u8;

    /// zoom level
    #[must_use]
    fn zoom(&self) -> u8 {
        self.z()
    }

    /// x coordinate
    fn yflip(&self) -> u32 {
        flipy(self.y(), self.z())
    }

    /// both bc I keep forgetting which is which
    fn flipy(&self) -> u32 {
        flipy(self.y(), self.z())
    }

    fn yup(&self) -> u32 {
        flipy(self.y(), self.z())
    }

    fn xyz_str_fslash(&self) -> String {
        format!("{}/{}/{}", self.x(), self.y(), self.z())
    }

    fn zxy_str_fslash(&self) -> String {
        format!("{}/{}/{}", self.z(), self.x(), self.y())
    }

    fn xyz_str_sep(&self, sep: &str) -> String {
        format!("{}{}{}{}{}", self.x(), sep, self.y(), sep, self.z())
    }

    fn zxy_str_sep(&self, sep: &str) -> String {
        format!("{}{}{}{}{}", self.z(), sep, self.x(), sep, self.y())
    }

    /// Return Tile struct
    #[must_use]
    fn tile(&self) -> Tile {
        Tile::new(self.x(), self.y(), self.z())
    }

    /// Return if the tile is valid (x, y is in bounds for zoom level z)
    #[must_use]
    fn valid(&self) -> bool {
        crate::valid(self.x(), self.y(), self.z())
    }

    /// Return the ul (upper left) corner of the tile
    #[must_use]
    fn ul(&self) -> LngLat {
        crate::ul(self.x(), self.y(), self.z())
    }

    /// Return the ur (upper right) corner of the tile
    #[must_use]
    fn ur(&self) -> LngLat {
        crate::ul(self.x() + 1, self.y(), self.z())
    }

    /// Return the lr (lower right) corner of the tile
    #[must_use]
    fn lr(&self) -> LngLat {
        crate::ul(self.x() + 1, self.y() + 1, self.z())
    }

    /// Return the ll (lower left) corner of the tile
    #[must_use]
    fn ll(&self) -> LngLat {
        crate::ul(self.x(), self.y() + 1, self.z())
    }

    /// Return the quadkey for the tile
    #[must_use]
    fn quadkey(&self) -> String {
        crate::xyz2quadkey(self.x(), self.y(), self.z())
    }

    /// Return the quadkey for the tile (alias for quadkey)
    #[must_use]
    fn qk(&self) -> String {
        crate::xyz2quadkey(self.x(), self.y(), self.z())
    }

    /// Return the pmtile-id for the tile
    #[cfg(feature = "pmtiles")]
    #[must_use]
    fn pmtileid(&self) -> u64 {
        pmtiles::xyz2pmid(self.x(), self.y(), self.z())
    }

    /// Return the pmtile id
    #[cfg(feature = "pmtiles")]
    #[must_use]
    fn pmid(&self) -> u64 {
        self.pmtileid()
    }

    /// Return the row major id for the tile
    #[must_use]
    fn row_major_id(&self) -> u64 {
        xyz2rmid(self.x(), self.y(), self.z())
    }

    /// Return the row major id for the tile (alias for `row_major_id`)
    #[must_use]
    fn rmid(&self) -> u64 {
        self.row_major_id()
    }

    /// Return the geo-bbox tuple for the tile (west, south, east, north)
    #[must_use]
    fn bbox(&self) -> (f64, f64, f64, f64) {
        let ul = self.ul();
        let lr = self.lr();
        (ul.lng(), lr.lat(), lr.lng(), ul.lat())
    }

    #[must_use]
    fn geobbox(&self) -> BBox {
        let ul = self.ul();
        let lr = self.lr();
        BBox::new(ul.lng(), lr.lat(), lr.lng(), ul.lat())
    }

    #[must_use]
    fn webbbox(&self) -> WebBBox {
        self.geobbox().into()
    }

    #[must_use]
    fn bbox_string(&self) -> String {
        let (w, s, e, n) = self.bbox();
        format!("[{w},{s},{e},{n}]")
    }

    /// Return the center of the tile as a `LngLat`
    #[must_use]
    fn center(&self) -> LngLat {
        let ul = self.ul();
        let lr = self.lr();
        LngLat::new(
            f64::midpoint(ul.lng(), lr.lng()),
            f64::midpoint(ul.lat(), lr.lat()),
        )
    }

    /// Return json array string for tile with spaces after commas
    #[must_use]
    fn json_arr(&self) -> String {
        format!("[{}, {}, {}]", self.x(), self.y(), self.z())
    }

    /// Return json array string for tile with no spaces after commas
    #[must_use]
    fn json_arr_min(&self) -> String {
        format!("[{},{},{}]", self.x(), self.y(), self.z())
    }

    /// Return json object string for tile
    #[must_use]
    fn json_obj(&self) -> String {
        format!(
            "{{\"x\":{}, \"y\":{}, \"z\":{}}}",
            self.x(),
            self.y(),
            self.z()
        )
    }

    /// Return json object string for tile
    #[must_use]
    fn json(&self) -> String {
        self.json_obj()
    }

    /// Return tuple string for tile `(x, y, z)`
    #[must_use]
    fn tuple_string(&self) -> String {
        format!("({}, {}, {})", self.x(), self.y(), self.z())
    }

    /// Return sql `WHERE` clause for querying mbtiles (y is up)
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

    /// return zbox for tile-like
    #[must_use]
    fn zbox(&self) -> TileZBox {
        TileZBox::from_tile(self)
    }

    /// Return children-zbox for tile-like at optional depth (default 1)
    #[must_use]
    fn children_zbox(&self, depth: Option<u8>) -> TileZBox {
        let d = depth.unwrap_or(1);
        let target_zoom = self.z() + d;
        let mut zbox = TileZBox::from_tile(self);
        while zbox.z() < target_zoom {
            zbox = zbox.zoom_in();
        }
        zbox
    }

    /// Return neighbor tiles for tile-like optionally wrapping x
    #[must_use]
    fn neighbors(&self, wrapx: bool) -> Vec<Tile> {
        if wrapx {
            neighbors_wrap_x(self.x(), self.y(), self.z())
        } else {
            neighbors(self.x(), self.y(), self.z())
        }
    }
}
