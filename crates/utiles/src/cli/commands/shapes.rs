use crate::cli::stdinterator::StdInterator;
use clap::{Args, Parser};
use serde_json::{Map, Value};
use tracing::debug;
use utiles_core::projection::Projection;
use utiles_core::tile::FeatureOptions;
use utiles_core::Tile;

// #[group(required = false, id="projected")]
#[derive(Args, Debug)]
#[group(required = false, multiple = false, id = "project")]
pub struct ShapesProject {
    /// Output in geographic coordinates (the default).
    #[arg(long , default_value = "false", conflicts_with = "mercator", action = clap::ArgAction::SetTrue)]
    geographic: bool,

    /// Output in Web Mercator coordinates.
    #[arg(long , default_value = "false", conflicts_with = "geographic", action = clap::ArgAction::SetTrue)]
    mercator: bool,
}

impl Default for ShapesProject {
    fn default() -> Self {
        ShapesProject {
            geographic: true,
            mercator: false,
        }
    }
}

#[derive(Args, Debug)]
#[group(required = false, multiple = false, id = "output-mode")]
pub struct ShapesOutputMode {
    #[arg(long , default_value = "false", conflicts_with = "bbox", action = clap::ArgAction::SetTrue)]
    feature: bool,

    /// Output in Web Mercator coordinates.
    #[arg(long , default_value = "false", conflicts_with = "feature", action = clap::ArgAction::SetTrue)]
    bbox: bool,
}

impl Default for ShapesOutputMode {
    fn default() -> Self {
        ShapesOutputMode {
            feature: true,
            bbox: false,
        }
    }
}

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "shapes", about = "echo shapes of tile(s) as GeoJSON", long_about = None)]
pub struct ShapesArgs {
    #[arg(required = false)]
    input: Option<String>,

    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    seq: bool,

    /// Decimal precision of coordinates.
    #[arg(long, value_parser)]
    precision: Option<i32>,

    #[command(flatten)]
    project: Option<ShapesProject>,

    #[command(flatten)]
    output_mode: Option<ShapesOutputMode>,

    /// Output as a GeoJSON feature collections.
    #[arg(long, action)]
    collect: bool,

    /// Write shape extents as ws-separated strings (default is False).
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    extents: bool,

    /// Shift shape x and y values by a constant number.
    #[arg(long, value_parser)]
    buffer: Option<f64>,
}

impl Default for ShapesArgs {
    fn default() -> Self {
        ShapesArgs {
            input: None,
            seq: false,
            precision: None,
            project: Some(ShapesProject::default()),
            output_mode: Some(ShapesOutputMode::default()),
            collect: false,
            extents: false,
            buffer: None,
        }
    }
}

struct TileWithProperties {
    tile: Tile,
    id: Option<String>,
    properties: Option<Map<String, Value>>,
}

pub fn shapes_main(args: ShapesArgs) {
    debug!("{:?}", args);
    let input_lines = StdInterator::new(args.input);
    let lines = input_lines
        .filter(|l| !l.is_err())
        .filter(|l| !l.as_ref().unwrap().is_empty())
        .filter(|l| l.as_ref().unwrap() != "\x1e");
    let parsed_lines = lines.map(|l| {
        let ln = l.unwrap();
        let val: Value = serde_json::from_str::<Value>(&ln).unwrap();
        let properties: Option<Map<String, Value>> = if val["properties"].is_object() {
            let properties = val["properties"].as_object().unwrap().clone();
            Option::from(properties)
        } else {
            None
        };
        // let properties: Option<Map<String, Value>> = match val["properties"].is_object()
        // {
        //     true => {
        //         let properties = val["properties"].as_object().unwrap().clone();
        //         Option::from(properties)
        //     }
        //     false => None,
        // };

        let id = if val["id"].is_string() {
            let id = val["id"].as_str().unwrap().to_string();
            Option::from(id)
        } else {
            None
        };
        let t = Tile::from(&val);
        TileWithProperties {
            tile: t,
            id,
            properties,
        }

        // Tile::from_json_loose(&ln)
    });
    let feature_options: FeatureOptions = FeatureOptions {
        fid: None,
        projection: match args.project {
            Some(ShapesProject {
                geographic: false,
                mercator: true,
            }) => Projection::Mercator,
            // ShapesProject {
            //     geographic: false,
            //     mercator: true,
            // } => Projection::Mercator,
            _ => Projection::Geographic,
        },
        props: None,
        buffer: args.buffer,
        precision: args.precision,
    };

    if args.collect {
        println!("{{");
        println!("\"type\": \"FeatureCollection\",");
        println!("\"features\": [");
    }
    let mut lons: Vec<f64> = Vec::new();
    let mut lats: Vec<f64> = Vec::new();
    let output_bbox = match args.output_mode {
        Some(output_mode) => matches!(
            output_mode,
            ShapesOutputMode {
                feature: false,
                bbox: true,
            }
        ),
        // Some(output_mode) => match output_mode {
        //     ShapesOutputMode {
        //         feature: false,
        //         bbox: true,
        //     } => true,
        //     _ => false,
        // },
        None => false,
    };

    let mut first = true;

    for tile_n_properties in parsed_lines {
        let tile = tile_n_properties.tile;
        let properties = tile_n_properties.properties;
        let mut f = tile.feature(&feature_options).unwrap();

        if let Some(properties) = properties {
            f.properties.extend(properties);
        }
        if let Some(id) = tile_n_properties.id {
            f.id = id;
        }
        lons.extend(f.bbox_lons());
        lats.extend(f.bbox_lats());
        if args.extents {
            println!("{}", f.extents_string());
        } else if args.collect {
            if !first {
                println!(",");
            }
            println!("  {}", f.to_json());
            first = false;
        } else {
            if args.seq {
                println!("\x1e");
            }
            if output_bbox {
                println!("{}", f.bbox_json());
            } else {
                println!("{}", f.to_json());
            }
        }
    }
    if args.collect {
        println!("]");
        println!("}}");
    }
}
