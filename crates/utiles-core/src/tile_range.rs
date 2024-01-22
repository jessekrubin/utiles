#[derive(Debug)]
pub struct TileRange {
    minx: u32,
    maxx: u32,
    miny: u32,
    maxy: u32,
    zoom: u8,
}

#[derive(Debug)]
pub struct TileRangeIterator {
    range: TileRange,
    curx: u32,
    cury: u32,
}

#[derive(Debug)]
pub struct TileRanges {
    ranges: Vec<TileRange>,
}

impl TileRange {
    #[must_use]
    pub fn new(minx: u32, maxx: u32, miny: u32, maxy: u32, zoom: u8) -> Self {
        Self {
            minx,
            maxx,
            miny,
            maxy,
            zoom,
        }
    }

    #[must_use]
    pub fn minx(&self) -> u32 {
        self.minx
    }
    #[must_use]
    pub fn maxx(&self) -> u32 {
        self.maxx
    }
    #[must_use]
    pub fn miny(&self) -> u32 {
        self.miny
    }
    #[must_use]
    pub fn maxy(&self) -> u32 {
        self.maxy
    }
    #[must_use]
    pub fn zoom(&self) -> u8 {
        self.zoom
    }

    #[must_use]
    pub fn length(&self) -> u64 {
        u64::from((self.maxx - self.minx + 1) * (self.maxy - self.miny + 1))
    }

    #[must_use]
    pub fn flipy(&self) -> Self {
        Self {
            minx: self.minx,
            maxx: self.maxx,
            miny: crate::fns::flipy(self.miny, self.zoom),
            maxy: crate::fns::flipy(self.maxy, self.zoom),
            zoom: self.zoom,
        }
    }

    #[must_use]
    pub fn mbtiles_sql_where(&self) -> String {
        // classic mbtiles sqlite query:
        // 'SELECT tile_data FROM tiles WHERE zoom_level = ? AND tile_column = ? AND tile_row = ?',
        let miny = crate::fns::flipy(self.miny, self.zoom);
        let maxy = crate::fns::flipy(self.maxy, self.zoom);
        format!(
            "(zoom_level = {} AND tile_column >= {} AND tile_column <= {} AND tile_row >= {} AND tile_row <= {})",
            self.zoom,
            self.minx, self.maxx,
            maxy, miny
        )
    }
}

impl TileRangeIterator {
    #[must_use]
    pub fn new(minx: u32, maxx: u32, miny: u32, maxy: u32, zoom: u8) -> Self {
        Self {
            range: TileRange::new(minx, maxx, miny, maxy, zoom),
            curx: minx,
            cury: miny,
        }
    }
}

impl From<TileRange> for TileRangeIterator {
    fn from(range: TileRange) -> Self {
        Self::new(
            range.minx(),
            range.maxx(),
            range.miny(),
            range.maxy(),
            range.zoom(),
        )
    }
}

impl Iterator for TileRangeIterator {
    type Item = (u32, u32, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.curx > self.range.maxx {
            self.curx = self.range.minx;
            self.cury += 1;
        }
        if self.cury > self.range.maxy {
            return None;
        }
        let tile = (self.curx, self.cury, self.range.zoom);
        self.curx += 1;
        Some(tile)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = ((self.range.maxx - self.range.minx + 1)
            * (self.range.maxy - self.range.miny + 1)) as usize;
        (size, Some(size))
    }
}

impl IntoIterator for TileRange {
    type Item = (u32, u32, u8);
    type IntoIter = TileRangeIterator;

    fn into_iter(self) -> Self::IntoIter {
        TileRangeIterator::from(self)
    }
}

impl IntoIterator for TileRanges {
    type Item = (u32, u32, u8);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges
            .into_iter()
            .flat_map(|r| r.into_iter())
            .collect::<Vec<Self::Item>>()
            .into_iter()
    }
}

impl TileRanges {
    #[must_use]
    pub fn new(minx: u32, maxx: u32, miny: u32, maxy: u32, zoom: u8) -> Self {
        Self {
            ranges: vec![TileRange::new(minx, maxx, miny, maxy, zoom)],
        }
    }

    #[must_use]
    pub fn flipy(&self) -> Self {
        Self {
            ranges: self.ranges.iter().map(TileRange::flipy).collect(),
        }
    }

    #[must_use]
    pub fn length(&self) -> u64 {
        self.ranges.iter().map(TileRange::length).sum()
    }

    #[must_use]
    pub fn mbtiles_sql_where(&self) -> String {
        self.ranges
            .iter()
            .map(|r| r.mbtiles_sql_where())
            .collect::<Vec<String>>()
            .join(" OR ")
    }
}

impl From<TileRange> for TileRanges {
    fn from(range: TileRange) -> Self {
        Self {
            ranges: vec![range],
        }
    }
}

impl From<Vec<TileRange>> for TileRanges {
    fn from(ranges: Vec<TileRange>) -> Self {
        Self { ranges }
    }
}
