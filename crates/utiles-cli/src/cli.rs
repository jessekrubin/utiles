use clap::{Parser, Subcommand, ValueEnum};
use std::io;
use std::io::BufRead;
use utiles::bbox::BBox;
use utiles::tiles;
use utiles::zoom::ZoomOrZooms;

pub enum LineSource {
    Single(String),
    Multiple(Box<dyn BufRead>),
}

pub struct StdInterator {
    source: LineSource,
}

impl StdInterator {
    fn new(input: Option<String>) -> io::Result<Self> {
        let source = match input {
            Some(file_content) => LineSource::Single(file_content),
            None => {
                let reader = Box::new(io::BufReader::new(io::stdin()));
                LineSource::Multiple(reader)
            }
        };
        Ok(Self { source })
    }
}

impl Iterator for StdInterator {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.source {
            LineSource::Single(content) => {
                if content.is_empty() {
                    None
                } else {
                    Some(Ok(std::mem::take(content)))
                }
            }
            LineSource::Multiple(reader) => {
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

pub struct Stdinput {
    pub arg: Option<String>,
}

// struct LineIterator {
//     reader: Box<dyn BufRead>,
// }
//
// impl LineIterator {
//     fn new(input: Option<String>) -> io::Result<Self> {
//         let reader: Box<dyn BufRead> = match input {
//             Some(input) => {
//             //     fake iterator
//                 Box::new(io::BufReader::new(input.as_bytes()))
//             },
//             None => Box::new(io::BufReader::new(io::stdin()))
//         };
//         Ok(Self { reader })
//     }
// }
//
// impl Iterator for LineIterator {
//     type Item = io::Result<String>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let mut line = String::new();
//         match self.reader.read_line(&mut line) {
//             Ok(0) => None, // EOF
//             Ok(_) => Some(Ok(line.trim_end().to_string())),
//             Err(e) => Some(Err(e)),
//         }
//     }
// }
impl Stdinput {
    fn new(arg: Option<String>) -> Self {
        Self { arg }
    }
}

impl Iterator for Stdinput {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle.read_line(&mut line).ok().map(|_| line)
    }
}

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "ut")]
#[command(about = "utiles cli (rust)", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
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
    Quadkey(QuadkeyArgs),

    /// tiles
    Tiles {
        #[arg(required = true)]
        zoom: u8,

        #[arg(required = false)]
        input: Option<String>,
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

pub fn cli_main() {
    let args = Cli::parse();

    match args.command {
        Commands::Quadkey(quadkey) => {
            let thingy = StdInterator::new(quadkey.quadkey).unwrap();
            for line in thingy {
                println!("Line from stdin: {}", line.unwrap());
            }
        }
        Commands::Tiles { zoom, input } => {
            let thingy = StdInterator::new(input).unwrap();
            for line in thingy {
                let lstr = line.unwrap();
                // println!("Line from stdin: {}", lstr);
                let thingy = BBox::from(lstr);

                for tile in tiles(
                    (thingy.west, thingy.south, thingy.east, thingy.north),
                    ZoomOrZooms::Zoom(zoom),
                ) {
                    println!("{}", tile.json_arr());
                }
                // )
                // let tr = tile_ranges((
                //     thingy.west,
                //     thingy.south,
                //     thingy.east,
                //     thingy.north,
                //                          ), ZoomOrZooms::Zoom(zoom));
                //
                // println!("tr: {:?}", tr);
                // for tile in tr {
                //     println!("tile: {:?}", tile);
                //     println!(tile.to_json());
                // }
            }
        } // Commands::External(args) => {
          //     println!("Calling out to {:?} with {:?}", &args[0], &args[1..]);
    }

    // let c_res = Connection::open(
    //     "D:\\maps\\reptiles\\mbtiles\\osm\\planet_z0z14_2022_10_13.mbtiles"
    // ).await;

    let filepath = "D:\\maps\\reptiles\\mbtiles\\osm\\planet_z0z14_2022_10_13.mbtiles";
    // "D:\\maps\\reptiles\\mbtiles\\osm\\planet_z0z14_2022_10_13.mbtiles",
    // "D:\\maps\\reptiles\\mbtiles\\globallandcover.mbtiles",
    // let mbt = MbtilesAsync::open(
    //     "D:\\maps\\reptiles\\mbtiles\\osm\\planet_z0z14_2022_10_13.mbtiles",
    // ).await?;
    //
    // let mdata = mbt.metadata().await?;
    //
    // let mut metadataMap: HashMap<String, Vec<String>> = HashMap::new();
    //
    // for thing in mdata {
    //     println!("{}: {}", thing.name, thing.value);
    //
    //     //     if it does not exist, create empty vector
    //     //     if it does exist, append to vector
    //     let mut v = metadataMap.entry(thing.name).or_insert(Vec::new());
    //     v.push(thing.value);
    // }
    //
    // println!("metadataMap: {:?}", metadataMap);
    //
    // println!("metadata_has_unique_index_name: {}", mbt.metadata_has_unique_index_name().await?);
    //
    // let mut mbtiles_manager = MbtilesManager::new();
    //
    // // Open the database connection
    // mbtiles_manager.open(
    //     filepath
    // ).unwrap();
    //
    // let mapfn = |row: &rusqlite::Row| -> rusqlite::Result<String> {
    //     Ok(row.get(0)?)
    // };
    //
    // let metadata = mbtiles_manager.metadata();
    // // Execute a query
    // let result= mbtiles_manager.query("SELECT name, value FROM metadata",
    //     mapfn
    // );
    // match result {
    //     Ok(rows) => {
    //         for row in rows {
    //             println!("{}", row);
    //         }
    //     }
    //     Err(err) => eprintln!("Query failed: {}", err),
    // }
    //
    // println!("metadata: {:?}", metadata);
    // // Close the database connection
    // mbtiles_manager.close().unwrap();
    //
    // // match c_res {
    // //     Ok(c) => println!("Connection opened"),
    // //     Err(e) => println!("Error opening connection: {}", e),
    // // }
    // let conn = match  c_res {
    //     Ok(c) => c,
    //     Err(e) => return Err(e),
    // };
    //
    // let mdata = conn
    //     .call(|conn| {
    //         let mut stmt = conn.prepare("SELECT name, value FROM metadata")?;
    //         let mdata = stmt
    //             .query_map([], |row| {
    //                 Ok(
    //                     MetadataRow {
    //                         name: row.get(0)?,
    //                         value: row.get(1)?,
    //                     }
    //                 )
    //             })?
    //             .collect::<Result<Vec<MetadataRow>, rusqlite::Error>>()?;
    //
    //         Ok::<_, rusqlite::Error>(mdata)
    //     })
    //     .await?;
    //
    //
    //
    // for thing in mdata {
    //     println!("{}: {}", thing.name, thing.value);
    // }

    // let mbt = Connection
}
