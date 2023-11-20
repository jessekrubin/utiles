use pyo3::FromPyObject;
use utiles::zoom::ZoomOrZooms;

#[derive(FromPyObject)]
pub enum ZoomsOrInt {
    Zooms(Vec<u8>),
    Int(u8),
}

#[derive(FromPyObject)]
pub enum PyZoomOrZooms {
    #[pyo3(transparent, annotation = "int")]
    Zoom(u8),
    #[pyo3(transparent, annotation = "list[int]")]
    Zooms(Vec<u8>),
}

impl From<PyZoomOrZooms> for ZoomOrZooms {
    fn from(val: PyZoomOrZooms) -> Self {
        match val {
            PyZoomOrZooms::Zoom(z) => ZoomOrZooms::Zoom(z),
            PyZoomOrZooms::Zooms(zs) => ZoomOrZooms::Zooms(zs),
        }
    }
}
