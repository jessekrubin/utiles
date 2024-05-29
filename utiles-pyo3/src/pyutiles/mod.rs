pub use pyfns::*;
pub use pytile_type::{tiletype, tiletype2headers, tiletype_str};
pub use simplipy::simplify;
use utiles::BBox;

use crate::pyutiles::pylnglatbbox::PyLngLatBbox;

pub mod pybbox;
pub mod pyiters;
pub mod pylnglat;
pub mod pylnglatbbox;
pub mod pytile;
pub mod pytilelike;
pub mod pytiles;
pub mod tuple_slice;
pub mod zoom;

pub mod pycoords;
mod pyfns;
pub mod pyparsing;
pub(crate) mod pytile_tuple;
mod pytile_type;
pub mod pytiles_generator;
mod simplipy;

impl From<PyLngLatBbox> for BBox {
    fn from(val: PyLngLatBbox) -> Self {
        val.bbox
    }
}
