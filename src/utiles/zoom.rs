pub enum ZoomOrZooms {
    Zoom(u8),
    Zooms(Vec<u8>),
}

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
