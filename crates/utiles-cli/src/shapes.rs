use clap::{Args, Parser};
use crate::stdinterator::StdInterator;
use utiles::Tile;

// #[group(required = false, id="projected")]
#[derive(Args, Debug)]
#[group(required =false , multiple = false, id = "project")]
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
#[group(required =false , multiple = false, id = "output-mode")]
pub struct ShapesOutputMode{
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

    /// Indentation level for JSON output.
    #[arg(long, value_parser)]
    indent: Option<i32>,

    /// Use compact separators (',', ':').
    #[arg(long, action)]
    compact: bool,


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
            indent: None,
            compact: false,
            project: Option::Some(ShapesProject::default()),
            output_mode:  Option::Some(ShapesOutputMode::default()),
            collect: false,
            extents: false,
            buffer: None,
        }
    }
}

pub fn shapes_main(args: ShapesArgs) {
    println!("{:?}", args);
    let input_lines = StdInterator::new(args.input).unwrap();
    let lines = input_lines
        .filter(|l| !l.is_err())
        .filter(|l| !l.as_ref().unwrap().is_empty())
        .filter(|l| l.as_ref().unwrap() != "\x1e");
    let tiles = lines.map(|l| {
        // Tile::from_json(&l.unwrap())
        let val = l.unwrap();

    });

    for tile in tiles {
        println!("{:?}", tile);
    }
}