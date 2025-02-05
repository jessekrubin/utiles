use geozero::mvt::tile;

pub struct UtilesMvtFeature<'a> {
    pub inner: &'a tile::Feature,
    pub extent: u32,
    pub xyz: utiles::Tile,
}

pub struct UtilesMvtLayer<'a> {
    pub inner: &'a tile::Layer,
    pub xyz: utiles::Tile,
}
