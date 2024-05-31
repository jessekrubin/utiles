//! `TileZBox` - zoom-x-y bounding box
use crate::{Point2d, TileLike};

/// A struct representing a bbox of tiles at a specific zoom level
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileZBox {
    zoom: u8,
    min: Point2d<u32>,
    max: Point2d<u32>,
}

/// A struct representing a set of `TileZBox`es
#[derive(Debug)]
pub struct TileZBoxes {
    ranges: Vec<TileZBox>,
}

/// An iterator over a `TileZBox` that yields tiles
#[derive(Debug)]
pub struct TileZBoxIterator {
    range: TileZBox,
    cur_x: u32,
    cur_y: u32,
}

impl TileZBox {
    /// Create a new `TileZBox`
    #[must_use]
    pub fn new(min_x: u32, max_x: u32, min_y: u32, max_y: u32, zoom: u8) -> Self {
        Self {
            zoom,
            min: Point2d::new(min_x, min_y),
            max: Point2d::new(max_x, max_y),
        }
    }

    /// Return the minimum x value
    #[must_use]
    pub fn minx(&self) -> u32 {
        self.min.x
    }

    /// Return the maximum x value
    #[must_use]
    pub fn maxx(&self) -> u32 {
        self.max.x
    }

    /// Return the minimum y value
    #[must_use]
    pub fn miny(&self) -> u32 {
        self.min.y
    }

    /// Return the maximum y value
    #[must_use]
    pub fn maxy(&self) -> u32 {
        self.max.y
    }

    /// Return the zoom level
    #[must_use]
    pub fn zoom(&self) -> u8 {
        self.zoom
    }

    /// Return dimensions of the `TileZBox`
    #[must_use]
    pub fn dimensions(&self) -> (u32, u32) {
        (self.max.x - self.min.x + 1, self.max.y - self.min.y + 1)
    }

    /// Return the number of tiles contained by the `TileZBox`
    #[must_use]
    pub fn length(&self) -> u64 {
        u64::from((self.max.x - self.min.x + 1) * (self.max.y - self.min.y + 1))
    }

    /// Return the size of the `TileZBox` in tiles
    #[must_use]
    pub fn size(&self) -> u64 {
        self.length()
    }

    /// Return a new `TileZBox` with the y values flipped for the given zoom level
    #[must_use]
    pub fn flipy(&self) -> Self {
        Self {
            min: Point2d::new(self.min.x, crate::fns::flipy(self.max.y, self.zoom)),
            max: Point2d::new(self.max.x, crate::fns::flipy(self.min.y, self.zoom)),
            zoom: self.zoom,
        }
    }

    /// Return whether the `TileZBox` contains the given tile-like input
    #[must_use]
    pub fn contains_tile<T: TileLike>(&self, tile: &T) -> bool {
        tile.z() == self.zoom
            && tile.x() >= self.min.x
            && tile.x() <= self.max.x
            && tile.y() >= self.min.y
            && tile.y() <= self.max.y
    }

    /// Return the SQL `WHERE` clause for an mbtiles database
    #[must_use]
    pub fn mbtiles_sql_where(&self) -> String {
        // classic mbtiles sqlite query:
        // 'SELECT tile_data FROM tiles WHERE zoom_level = ? AND tile_column = ? AND tile_row = ?',
        let miny = crate::fns::flipy(self.min.y, self.zoom);
        let maxy = crate::fns::flipy(self.max.y, self.zoom);
        format!(
            "(zoom_level = {} AND tile_column >= {} AND tile_column <= {} AND tile_row >= {} AND tile_row <= {})",
            self.zoom,
            self.min.x, self.max.x,
            maxy, miny
        )
    }
}

impl TileZBoxIterator {
    /// Create a new `TileZBoxIterator`
    #[must_use]
    pub fn new(minx: u32, maxx: u32, miny: u32, maxy: u32, zoom: u8) -> Self {
        Self {
            range: TileZBox::new(minx, maxx, miny, maxy, zoom),
            cur_x: minx,
            cur_y: miny,
        }
    }
}

impl From<TileZBox> for TileZBoxIterator {
    fn from(range: TileZBox) -> Self {
        Self::new(
            range.minx(),
            range.maxx(),
            range.miny(),
            range.maxy(),
            range.zoom(),
        )
    }
}

impl Iterator for TileZBoxIterator {
    type Item = (u32, u32, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_x > self.range.max.x {
            self.cur_x = self.range.min.x;
            self.cur_y += 1;
        }
        if self.cur_y > self.range.max.y {
            return None;
        }
        let tile = (self.cur_x, self.cur_y, self.range.zoom);
        self.cur_x += 1;
        Some(tile)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = ((self.range.max.x - self.range.min.x + 1)
            * (self.range.max.y - self.range.min.y + 1)) as usize;
        (size, Some(size))
    }
}

impl IntoIterator for TileZBox {
    type Item = (u32, u32, u8);
    type IntoIter = TileZBoxIterator;

    fn into_iter(self) -> Self::IntoIter {
        TileZBoxIterator::from(self)
    }
}

impl IntoIterator for TileZBoxes {
    type Item = (u32, u32, u8);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges
            .into_iter()
            .flat_map(std::iter::IntoIterator::into_iter)
            .collect::<Vec<Self::Item>>()
            .into_iter()
    }
}

impl TileZBoxes {
    /// Create a new `TileZBoxes` from a single `TileZBox`
    #[must_use]
    pub fn new(minx: u32, maxx: u32, miny: u32, maxy: u32, zoom: u8) -> Self {
        Self {
            ranges: vec![TileZBox::new(minx, maxx, miny, maxy, zoom)],
        }
    }

    /// Create a new `TileZBoxes` from a single `TileZBox`
    #[must_use]
    pub fn flipy(&self) -> Self {
        Self {
            ranges: self.ranges.iter().map(TileZBox::flipy).collect(),
        }
    }

    /// Return the number of tiles contained by the `TileZBoxes`
    #[must_use]
    pub fn length(&self) -> u64 {
        self.ranges.iter().map(TileZBox::length).sum()
    }

    /// Return the size of the `TileZBoxes` in tiles
    #[must_use]
    pub fn mbtiles_sql_where(&self) -> String {
        self.ranges
            .iter()
            .map(TileZBox::mbtiles_sql_where)
            .collect::<Vec<String>>()
            .join(" OR ")
    }
}

impl From<TileZBox> for TileZBoxes {
    fn from(range: TileZBox) -> Self {
        Self {
            ranges: vec![range],
        }
    }
}

impl From<Vec<TileZBox>> for TileZBoxes {
    fn from(ranges: Vec<TileZBox>) -> Self {
        Self { ranges }
    }
}
