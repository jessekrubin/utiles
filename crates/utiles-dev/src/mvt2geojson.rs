use geozero::error::Result;
use geozero::mvt::{tile, tile::GeomType};
// vector_tile::{tile, tile::GeomType};
use geozero::{
    ColumnValue, FeatureProcessor, GeomProcessor, GeozeroDatasource, GeozeroGeometry,
};

use crate::mvt_commands::{Command, CommandInteger, ParameterInteger};
use geozero::mvt::{
    MvtError,
    // mvt_commands::{Command, CommandInteger, ParameterInteger},
    // mvt_error::MvtError,
};

use crate::mvt_types::{UtilesMvtFeature, UtilesMvtLayer};

/// A helper function to transform tile-local coordinates into WGS84 (EPSG:4326).
///
/// - `extent` is usually 4096 in MVT, but can be different per layer.
/// - `tile_x, tile_y, zoom` are the [z/x/y] of the tile in typical slippy-map notation.
fn tile_local_to_lnglat(
    x: i32,
    y: i32,
    extent: f64,
    // tile_x: i32,
    // tile_y: i32,
    // zoom: i32,
    xyz: utiles::Tile,
) -> (f64, f64) {
    // Number of tiles in each axis at this zoom.
    let n = 2.0_f64.powi(xyz.z as i32);

    // Convert MVT local coordinates (0..extent) → normalized [0..1],
    // then to tile-based [0..n], then into lon/lat.
    let lon = (xyz.x as f64 + x as f64 / extent) / n * 360.0 - 180.0;

    // Latitude is a bit trickier: we convert the "map fraction" into
    // a mercator Y, then invert the Web Mercator transform.
    let lat_rad = ((1.0 - 2.0 * (xyz.y as f64 + y as f64 / extent) / n)
        * std::f64::consts::PI)
        .sinh()
        .atan();

    let lat = lat_rad.to_degrees();
    (lon, lat)
}

/// Extend tile::Layer to be a GeoZero datasource *with lat/lon*:
///
/// We’ll embed tile/tile_x/tile_y/zoom in the processor so that
/// the geometry is converted on the fly.
impl GeozeroDatasource for UtilesMvtLayer<'_> {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        // We can pass tile_x, tile_y, zoom, or store them somewhere globally.
        // For example, if you have them in your struct, pass them in:
        process_layer_latlon(self, processor, self.xyz)
    }
}

/// The main entry point for processing a layer's features *in lat/lon*.
pub fn process_layer_latlon(
    layer: &UtilesMvtLayer,
    processor: &mut impl FeatureProcessor,
    xyz: utiles::Tile,
) -> Result<()> {
    processor.dataset_begin(Some(&layer.inner.name))?;

    for (idx, feature) in layer.inner.features.iter().enumerate() {
        processor.feature_begin(idx as u64)?;

        process_properties(layer.inner, feature, processor)?;

        processor.geometry_begin()?;
        let extent_f64 = f64::from(layer.inner.extent.unwrap_or(4096));
        process_geom_latlon(feature, extent_f64, xyz, processor)?;
        processor.geometry_end()?;

        processor.feature_end(idx as u64)?;
    }

    processor.dataset_end()
}

/// Collect MVT feature properties.
fn process_properties(
    layer: &tile::Layer,
    feature: &tile::Feature,
    processor: &mut impl FeatureProcessor,
) -> Result<()> {
    processor.properties_begin()?;
    for (i, pair) in feature.tags.chunks(2).enumerate() {
        let [key_idx, value_idx] = pair else {
            return Err(MvtError::InvalidFeatureTagsLength(feature.tags.len()).into());
        };
        let key = layer
            .keys
            .get(*key_idx as usize)
            .ok_or(MvtError::InvalidKeyIndex(*key_idx))?;
        let value = layer
            .values
            .get(*value_idx as usize)
            .ok_or(MvtError::InvalidValueIndex(*value_idx))?;

        if let Some(ref v) = value.string_value {
            processor.property(i, key, &ColumnValue::String(v))?;
        } else if let Some(v) = value.float_value {
            processor.property(i, key, &ColumnValue::Float(v))?;
        } else if let Some(v) = value.double_value {
            processor.property(i, key, &ColumnValue::Double(v))?;
        } else if let Some(v) = value.int_value {
            processor.property(i, key, &ColumnValue::Long(v))?;
        } else if let Some(v) = value.uint_value {
            processor.property(i, key, &ColumnValue::ULong(v))?;
        } else if let Some(v) = value.sint_value {
            processor.property(i, key, &ColumnValue::Long(v))?;
        } else if let Some(v) = value.bool_value {
            processor.property(i, key, &ColumnValue::Bool(v))?;
        } else {
            return Err(MvtError::UnsupportedKeyValueType(key.to_string()).into());
        }
    }
    processor.properties_end()
}

/// Make tile::Feature geometry workable via GeoZero's `GeomProcessor` *in lat/lon*.
impl GeozeroGeometry for UtilesMvtFeature<'_> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        // If you don’t want lat/lon transform, you’d call the original. But we do want it:
        process_geom_latlon(self.inner, f64::from(self.extent), self.xyz, processor)
    }
}

/// Main geometry dispatcher: picks point/line/polygon.
pub fn process_geom_latlon<P: GeomProcessor>(
    feature: &tile::Feature,
    extent: f64,
    xyz: utiles::Tile,
    // tile_x: i32,
    // tile_y: i32,
    // zoom: i32,
    processor: &mut P,
) -> Result<()> {
    process_geom_n_latlon(feature, 0, extent, xyz, processor)
}

fn process_geom_n_latlon<P: GeomProcessor>(
    feature: &tile::Feature,
    idx: usize,
    extent: f64,
    // tile_x: i32,
    // tile_y: i32,
    // zoom: i32,
    xyz: utiles::Tile,
    processor: &mut P,
) -> Result<()> {
    let mut cursor: [i32; 2] = [0, 0];
    match feature.r#type {
        Some(r#type) if r#type == GeomType::Point as i32 => process_point_latlon(
            &mut cursor,
            &feature.geometry,
            idx,
            extent,
            xyz,
            processor,
        ),
        Some(r#type) if r#type == GeomType::Linestring as i32 => {
            process_linestrings_latlon(
                &mut cursor,
                feature,
                idx,
                extent,
                xyz,
                processor,
            )
        }
        Some(r#type) if r#type == GeomType::Polygon as i32 => {
            process_polygons_latlon(&mut cursor, feature, idx, extent, xyz, processor)
        }
        // No geometry or unknown type → do nothing
        _ => Ok(()),
    }
}

/// Decode and transform a single coordinate in the MVT geometry stream.
fn process_coord_latlon<P: GeomProcessor>(
    cursor: &mut [i32; 2],
    coord: &[u32],
    idx: usize,
    extent: f64,
    xyz: utiles::Tile,
    processor: &mut P,
) -> Result<()> {
    // Delta-decode
    cursor[0] += ParameterInteger(coord[0]).value();
    cursor[1] += ParameterInteger(coord[1]).value();

    // Transform to lat/lon
    let (lng, lat) = tile_local_to_lnglat(cursor[0], cursor[1], extent, xyz);

    if processor.multi_dim() {
        processor.coordinate(lng, lat, None, None, None, None, idx)
    } else {
        processor.xy(lng, lat, idx)
    }
}

fn process_point_latlon<P: GeomProcessor>(
    cursor: &mut [i32; 2],
    geom: &[u32],
    idx: usize,
    extent: f64,
    // tile_x: i32,
    // tile_y: i32,
    // zoom: i32,
    xyz: utiles::Tile,
    processor: &mut P,
) -> Result<()> {
    let command = CommandInteger(geom[0]);
    let count = command.count() as usize;

    if count == 1 {
        processor.point_begin(idx)?;
        process_coord_latlon(cursor, &geom[1..3], 0, extent, xyz, processor)?;
        processor.point_end(idx)
    } else {
        processor.multipoint_begin(count, idx)?;
        for i in 0..count {
            let start = 1 + i * 2;
            let end = start + 2;
            process_coord_latlon(cursor, &geom[start..end], i, extent, xyz, processor)?;
        }
        processor.multipoint_end(idx)
    }
}

fn process_linestring_latlon<P: GeomProcessor>(
    cursor: &mut [i32; 2],
    geom: &[u32],
    tagged: bool,
    idx: usize,
    extent: f64,
    xyz: utiles::Tile,
    processor: &mut P,
) -> Result<()> {
    if geom[0] != CommandInteger::from(Command::MoveTo, 1) {
        return Err(MvtError::GeometryFormat.into());
    }
    let lineto = CommandInteger(geom[3]);
    if lineto.id() != Command::LineTo as u32 {
        return Err(MvtError::GeometryFormat.into());
    }

    let line_len = 1 + lineto.count() as usize;
    processor.linestring_begin(tagged, line_len, idx)?;

    // MoveTo
    process_coord_latlon(cursor, &geom[1..3], 0, extent, xyz, processor)?;
    // LineTo
    for i in 0..lineto.count() as usize {
        let start = 4 + i * 2;
        let end = start + 2;
        process_coord_latlon(cursor, &geom[start..end], i + 1, extent, xyz, processor)?;
    }
    processor.linestring_end(tagged, idx)
}

fn process_linestrings_latlon<P: GeomProcessor>(
    cursor: &mut [i32; 2],
    feature: &tile::Feature,
    idx: usize,
    extent: f64,
    xyz: utiles::Tile,
    processor: &mut P,
) -> Result<()> {
    let mut line_string_slices: Vec<&[u32]> = vec![];
    let mut remainder = &feature.geometry[..];

    // Break the geometry into multiple line slices if necessary
    while !remainder.is_empty() {
        let lineto = CommandInteger(remainder[3]);
        let slice_size = 4 + lineto.count() as usize * 2;
        let (slice, rest) = remainder.split_at(slice_size);
        line_string_slices.push(slice);
        remainder = rest;
    }

    if line_string_slices.len() > 1 {
        processor.multilinestring_begin(line_string_slices.len(), idx)?;
        for (i, line_string_slice) in line_string_slices.iter().enumerate() {
            process_linestring_latlon(
                cursor,
                line_string_slice,
                false,
                i,
                extent,
                xyz,
                processor,
            )?;
        }
        processor.multilinestring_end(idx)
    } else {
        // Single line
        process_linestring_latlon(
            cursor,
            line_string_slices[0],
            true,
            idx,
            extent,
            xyz,
            processor,
        )
    }
}

fn process_polygon_latlon<P: GeomProcessor>(
    cursor: &mut [i32; 2],
    rings: &[&[u32]],
    tagged: bool,
    idx: usize,
    extent: f64,
    xyz: utiles::Tile,
    processor: &mut P,
) -> Result<()> {
    processor.polygon_begin(tagged, rings.len(), idx)?;

    for (i, ring) in rings.iter().enumerate() {
        if ring[0] != CommandInteger::from(Command::MoveTo, 1) {
            return Err(MvtError::GeometryFormat.into());
        }
        if *ring.last().unwrap() != CommandInteger::from(Command::ClosePath, 1) {
            return Err(MvtError::GeometryFormat.into());
        }
        let lineto = CommandInteger(ring[3]);
        if lineto.id() != Command::LineTo as u32 {
            return Err(MvtError::GeometryFormat.into());
        }

        processor.linestring_begin(false, 1 + lineto.count() as usize, i)?;

        // Remember the start so we can "close" the ring properly
        let mut start_cursor = *cursor;

        // MoveTo
        process_coord_latlon(cursor, &ring[1..3], 0, extent, xyz, processor)?;
        // LineTo
        for j in 0..lineto.count() as usize {
            let start = 4 + j * 2;
            let end = start + 2;
            process_coord_latlon(
                cursor,
                &ring[start..end],
                j + 1,
                extent,
                xyz,
                processor,
            )?;
        }
        // ClosePath: repeat first coordinate
        process_coord_latlon(
            &mut start_cursor,
            &ring[1..3],
            1 + lineto.count() as usize,
            extent,
            xyz,
            processor,
        )?;

        processor.linestring_end(false, i)?;
    }

    processor.polygon_end(tagged, idx)
}

fn process_polygons_latlon<P: GeomProcessor>(
    cursor: &mut [i32; 2],
    feature: &tile::Feature,
    idx: usize,
    extent: f64,
    xyz: utiles::Tile,
    processor: &mut P,
) -> Result<()> {
    let mut polygon_slices: Vec<Vec<&[u32]>> = vec![];
    let mut remainder: &[u32] = &feature.geometry;

    while !remainder.is_empty() {
        let lineto = CommandInteger(remainder[3]);
        let slice_size = 4 + lineto.count() as usize * 2 + 1;
        let (slice, rest) = remainder.split_at(slice_size);

        // Evaluate ring direction using area sign
        let positive_area = is_area_positive(
            *cursor,
            &slice[1..3],
            &slice[4..4 + lineto.count() as usize * 2],
        );
        if positive_area {
            // new polygon with exterior ring
            polygon_slices.push(vec![slice]);
        } else if let Some(last_slice) = polygon_slices.last_mut() {
            // add interior ring to previous polygon
            last_slice.push(slice);
        } else {
            return Err(MvtError::GeometryFormat.into());
        }
        remainder = rest;
    }

    // MultiPolygon or single Polygon
    if polygon_slices.len() > 1 {
        processor.multipolygon_begin(polygon_slices.len(), idx)?;
        for (i, polygon_slice) in polygon_slices.iter().enumerate() {
            process_polygon_latlon(
                cursor,
                polygon_slice,
                false,
                i,
                extent,
                xyz,
                processor,
            )?;
        }
        processor.multipolygon_end(idx)
    } else {
        process_polygon_latlon(
            cursor,
            &polygon_slices[0],
            true,
            idx,
            extent,
            xyz,
            processor,
        )
    }
}

/// Using surveyor's formula to detect ring orientation.
fn is_area_positive(mut cursor: [i32; 2], first: &[u32], rest: &[u32]) -> bool {
    let nb = 1 + rest.len() / 2;
    let mut area = 0_i64;
    let mut coords = first
        .iter()
        .chain(rest)
        .chain(first.iter())
        .map(|&x| ParameterInteger(x).value());
    cursor[0] += coords.next().unwrap();
    cursor[1] += coords.next().unwrap();
    for _i in 0..nb {
        let [x0, y0] = cursor;
        cursor[0] += coords.next().unwrap();
        cursor[1] += coords.next().unwrap();
        area += (x0 as i64) * (cursor[1] as i64) - (y0 as i64) * (cursor[0] as i64);
    }
    area > 0
}
