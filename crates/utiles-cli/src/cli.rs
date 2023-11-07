use clap::{Parser, Subcommand, ValueEnum};
use std::io;
use std::io::BufRead;
use tracing::debug;

use tracing_subscriber::util::SubscriberInitExt;
use utiles::bbox::BBox;
use utiles::tiles;
use utiles::zoom::ZoomOrZooms;

pub enum StdInteratorSource {
    Single(String),
    Multiple(Box<dyn BufRead>),
}

pub struct StdInterator {
    source: StdInteratorSource,
}

impl StdInterator {
    fn new(input: Option<String>) -> io::Result<Self> {
        let source = match input {
            Some(file_content) => {
                if file_content == "-" {
                    debug!("reading from stdin - got '-'");
                    let reader = Box::new(io::BufReader::new(io::stdin()));
                    StdInteratorSource::Multiple(reader)
                } else {
                    debug!("reading from args: {:?}", file_content);
                    StdInteratorSource::Single(file_content)
                }
            }
            None => {
                let reader = Box::new(io::BufReader::new(io::stdin()));
                debug!("reading from stdin - no args");
                StdInteratorSource::Multiple(reader)
            }
        };
        Ok(Self { source })
    }
}

impl Iterator for StdInterator {
    type Item = io::Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.source {
            StdInteratorSource::Single(content) => {
                if content.is_empty() {
                    None
                } else {
                    Some(Ok(std::mem::take(content)))
                }
            }
            StdInteratorSource::Multiple(reader) => {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => None, // EOF
                    Ok(_) => Some(Ok(line.trim_end().to_string())),
                    Err(e) => Some(Err(e)),
                }
            }
        }
    }
}

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "ut")]
#[command(about = "utiles cli (rust)", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    // debug flag
    #[arg(
    long,
    short,
    global = true,
    default_value = "false",
    help = "debug mode"
    )]
    debug: bool,
    // #[command(flatten , help="verbosity level (-v, -vv, -vvv, -vvvv)" )]
    // verbose: Verbosity,
}

#[derive(Debug, Parser)] // requires `derive` feature
pub struct QuadkeyArgs {
    /// The remote to clone
    // #[arg(required = false, default_value = "-")]
    // quadkey: MaybeStdin<String>,
    #[arg(required = false)]
    quadkey: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// quadkey
    // Quadkey {
    //     #[arg(required = true)]
    //     quadkey: String,
    // },
    #[command(name = "quadkey", visible_alias = "qk", about = "convert xyz <-> quadkey", long_about = None)]
    Quadkey(QuadkeyArgs),

    /// tiles
    Tiles {
        #[arg(required = true)]
        zoom: u8,

        #[arg(required = false)]
        input: Option<String>,

        #[arg(required = false, default_value = "false", long)]
        seq: Option<bool>,
    },
    // /// Clones repos
    // #[command(arg_required_else_help = true)]
    // Clone {
    //     /// The remote to clone
    //     remote: String,
    // },
    // /// Compare two commits
    // Diff {
    //     #[arg(value_name = "COMMIT")]
    //     base: Option<OsString>,
    //     #[arg(value_name = "COMMIT")]
    //     head: Option<OsString>,
    //     #[arg(last = true)]
    //     path: Option<OsString>,
    //     #[arg(
    //     long,
    //     require_equals = true,
    //     value_name = "WHEN",
    //     num_args = 0..=1,
    //     default_value_t = ColorWhen::Auto,
    //     default_missing_value = "always",
    //     value_enum
    //     )]
    //     color: ColorWhen,
    // },
    // /// pushes things
    // #[command(arg_required_else_help = true)]
    // Push {
    //     /// The remote to target
    //     remote: String,
    // },
    // /// adds things
    // #[command(arg_required_else_help = true)]
    // Add {
    //     /// Stuff to add
    //     #[arg(required = true)]
    //     path: Vec<PathBuf>,
    // },

    // Stash(StashArgs),
    // #[command(external_subcommand)]
    // External(Vec<OsString>),
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorWhen {
    Always,
    Auto,
    Never,
}

impl std::fmt::Display for ColorWhen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

pub fn cli_main(argv: Option<Vec<String>>, loop_fn: Option<&dyn Fn() -> ()>) {
    // print args
    let argv = match argv {
        Some(argv) => argv,
        None => std::env::args().collect::<Vec<_>>(),
    };

    let args = Cli::parse_from(&argv);

    // level is info by default and debug if --debug is passed
    let level = if args.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::WARN
    };

    // install global collector configured based on RUST_LOG env var.
    // tracing_subscriber::fmt::init();
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .finish()
        .init();
    debug!("args: {:?}", std::env::args().collect::<Vec<_>>());
    debug!("argv: {:?}", argv);

    debug!("args: {:?}", args);

    match args.command {
        Commands::Quadkey(quadkey) => {
            let thingy = StdInterator::new(quadkey.quadkey).unwrap();
            for line in thingy {
                println!("Line from stdin: `{}`", line.unwrap());
            }
        }
        Commands::Tiles { zoom, input, seq } => {
            let input_lines = StdInterator::new(input).unwrap();
            // println!("zoom: {}", zoom);
            let mut niter = 0;
            for line in input_lines
                .filter(|l| !l.is_err())
                .filter(|l| !l.as_ref().unwrap().is_empty())
            {
                let lstr = line.unwrap();
                // println!("Line from stdin: `{}`", lstr);
                // let json: serde_json::Value = serde_json::from_str(the_file)l;
                let thingy = BBox::from(lstr);

                for tile in tiles(
                    (thingy.west, thingy.south, thingy.east, thingy.north),
                    ZoomOrZooms::Zoom(zoom),
                ) {
                    println!("{}", tile.json_arr());

                //     call loop_fn if it's defined
                    niter += 1;

                    // call fn every 1000 iterations
                    if niter % 1000 == 0{

                        if let Some(f) = loop_fn {
                            f();
                        }
                    }
                }
            }
        }
    }
}
