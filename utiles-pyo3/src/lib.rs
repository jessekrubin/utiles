#![deny(clippy::all)]
#![deny(clippy::perf)]
#![deny(dead_code)]
#![warn(clippy::style)]
#![deny(clippy::pedantic)]
#![warn(clippy::unnecessary_wraps)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
// road to clippy pedantic
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::used_underscore_items)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::float_cmp)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unused_self)]
#![deny(clippy::unwrap_used)]

use pyo3::prelude::*;

use crate::pyutiles::PyTileType;
use pyutiles::PyBbox;
use pyutiles::PyLngLat;
use pyutiles::PyLngLatBbox;
use pyutiles::PyTile;
use utiles::tile_type;
mod cli;
mod fmt_nbytes;
// mod lager;
mod pylager;
mod pyutiles;
const PACKAGE: &str = "utiles";
const DESCRIPTION: &str = "Python bindings for the utiles library";

const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_PROFILE: &str = env!("PROFILE");
const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");

fn lib_constants(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__pkg_name__", PACKAGE)?;
    m.add("__description__", DESCRIPTION)?;
    m.add("__version__", VERSION)?;
    m.add("__build_profile__", BUILD_PROFILE)?;
    m.add("__build_timestamp__", BUILD_TIMESTAMP)?;
    m.add("__authors__", AUTHORS)?;
    //
    // m.add("__version_lib__", env!("CARGO_PKG_VERSION"))?;
    // m.add("__build_profile__", env!("PROFILE"))?;
    // m.add("__build_timestamp__", env!("BUILD_TIMESTAMP"))?;
    // m.add("__authors__", env!("CARGO_PKG_AUTHORS"))?;

    Ok(())
}

/// Utiles python module
#[pymodule]
fn _utiles(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // lib constants
    lib_constants(m)?;
    pylager::pymod_add(m)?;
    // mercantile functions
    m.add_function(wrap_pyfunction!(pyutiles::pyparsing::parse_tile_arg, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::pyparsing::_parse_tile_arg, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::minmax, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::ul, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::bounds, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::xy, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::_xy, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::lnglat, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::xy_bounds, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::tile, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::parent, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::quadkey, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::quadkey_to_tile, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::children, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::neighbors, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::tiles, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::bounding_tile, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::truncate_lnglat, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::pycoords::_coords, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::pycoords::coords, m)?)?;
    // m.add_function(wrap_pyfunction!(merge, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::simplify, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::geojson_bounds, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::feature, m)?)?;

    // utiles functions
    m.add_function(wrap_pyfunction!(pyutiles::geojson2tiles, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::tiles_count, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::tiles_list, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::xyz, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::pyparsing::parse_textiles, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::pyparsing::parse_tiles, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::xyz2quadkey, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::quadkey2xyz, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::from_tuple, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::pmtileid, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::pmtileid2xyz, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::qk2xyz, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::from_pmtileid, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::geotransform2optzoom, m)?)?;

    // tiletype
    m.add_class::<PyTileType>()?;
    m.add_function(wrap_pyfunction!(pyutiles::tiletype, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::tiletype_str, m)?)?;
    m.add_function(wrap_pyfunction!(pyutiles::tiletype2headers, m)?)?;
    m.add("TILETYPE_UNKNOWN", tile_type::TILETYPE_UNKNOWN)?;
    m.add("TILETYPE_GIF", tile_type::TILETYPE_GIF)?;
    m.add("TILETYPE_JPG", tile_type::TILETYPE_JPG)?;
    m.add("TILETYPE_JSON", tile_type::TILETYPE_JSON)?;
    m.add("TILETYPE_PBF", tile_type::TILETYPE_PBF)?;
    m.add("TILETYPE_PBFGZ", tile_type::TILETYPE_PBFGZ)?;
    m.add("TILETYPE_PNG", tile_type::TILETYPE_PNG)?;
    m.add("TILETYPE_WEBP", tile_type::TILETYPE_WEBP)?;

    // m.add_class::<TileTuple>()?;
    m.add_class::<PyTile>()?;
    m.add_class::<PyLngLat>()?;
    m.add_class::<PyLngLatBbox>()?;
    m.add_class::<PyBbox>()?;

    // tile str formatter
    m.add_class::<pyutiles::PyTileFmts>()?;

    // mbutiles...
    // m.add_class::<PyMbtiles>()?;
    // m.add_function(wrap_pyfunction!(query_db, m)?)?;

    // rust-cli
    m.add_function(wrap_pyfunction!(cli::ut_cli, m)?)?;

    // misc
    m.add_function(wrap_pyfunction!(fmt_nbytes::fmt_nbytes, m)?)?;

    // lager
    // lager::pymod_add(m)?;

    Ok(())
}
