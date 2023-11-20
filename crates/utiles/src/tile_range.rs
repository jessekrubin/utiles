#[derive(Debug)]
pub struct TileRange {
    curx: u32,
    cury: u32,
    minx: u32,
    maxx: u32,
    miny: u32,
    maxy: u32,
    zoom: u8,
}

impl TileRange {
    #[must_use]
    pub fn new(minx: u32, maxx: u32, miny: u32, maxy: u32, zoom: u8) -> Self {
        Self {
            curx: minx,
            cury: miny,
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
    pub fn sql_where(&self, flip: Option<bool>) -> String {
        // classic mbtiles sqlite query:
        // 'SELECT tile_data FROM tiles WHERE zoom_level = ? AND tile_column = ? AND tile_row = ?',
        let miny = if flip.unwrap_or(true) {
            crate::fns::flipy(self.miny, self.zoom)
        } else {
            self.miny
        };
        let maxy = if flip.unwrap_or(true) {
            crate::fns::flipy(self.maxy, self.zoom)
        } else {
            self.maxy
        };
        format!(
            "(zoom_level = {} AND tile_column >= {} AND tile_column <= {} AND tile_row >= {} AND tile_row <= {})",
            self.zoom, self.minx, self.maxx, miny, maxy
        )
    }
}

impl Iterator for TileRange {
    type Item = (u32, u32, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.curx > self.maxx {
            self.curx = self.minx;
            self.cury += 1;
        }
        if self.cury > self.maxy {
            return None;
        }
        let tile = (self.curx, self.cury, self.zoom);
        self.curx += 1;
        Some(tile)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = ((self.maxx - self.minx + 1) * (self.maxy - self.miny + 1)) as usize;
        (size, Some(size))
    }
}

#[derive(Debug)]
pub struct TileRanges {
    ranges: Vec<TileRange>,
}

impl TileRanges {
    #[must_use]
    pub fn new(minx: u32, maxx: u32, miny: u32, maxy: u32, zoom: u8) -> Self {
        Self {
            ranges: vec![TileRange::new(minx, maxx, miny, maxy, zoom)],
        }
    }

    #[must_use]
    pub fn length(&self) -> u64 {
        self.ranges.iter().map(TileRange::length).sum()
    }

    #[must_use]
    pub fn sql_where(&self, flip: Option<bool>) -> String {
        self.ranges
            .iter()
            .map(|r| r.sql_where(flip))
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

impl Iterator for TileRanges {
    type Item = (u32, u32, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ranges.is_empty() {
            return None;
        }
        let mut range = self.ranges.remove(0);
        let tile = range.next();
        if let Some((_, _, _)) = tile {
            self.ranges.push(range);
        }
        tile
    }
}
