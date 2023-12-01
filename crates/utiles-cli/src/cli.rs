use std::io::{self};

use clap::Parser;
use tracing::{debug, error, warn};
use tracing_subscriber::EnvFilter;

use crate::args::{Cli, Commands};
use crate::commands::{bounding_tile_main, metadata_main, neighbors_main, pmtileid_main, quadkey_main, tilejson_main};
use crate::commands::{children_main, parent_main};
use crate::commands::copy::copy_main;
use crate::commands::dev::dev_main;
use crate::commands::lint::lint_main;
use crate::commands::rimraf::rimraf_main;
use crate::commands::shapes::shapes_main;
use crate::commands::tiles::tiles_main;

#[allow(clippy::unused_async)]
pub async fn cli_main(argv: Option<Vec<String>>, loop_fn: Option<&dyn Fn()>) -> u8 {
    // print args
    let argv = match argv {
        Some(argv) => argv,
        None => std::env::args().collect::<Vec<_>>(),
    };
    let args = Cli::parse_from(&argv);
    let filter = if args.debug {
        EnvFilter::new("DEBUG")
    } else {
        EnvFilter::new("WARN")
    };
    // Install the global collector configured based on the filter.
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(io::stderr)
        .init();

    debug!("args: {:?}", std::env::args().collect::<Vec<_>>());
    debug!("argv: {:?}", argv);

    debug!("args: {:?}", args);

    match args.command {
        Commands::Lint {
            fspaths: filepath,
            fix,
        } => {
            if fix {
                warn!("fix not implemented");
            }
            lint_main(&filepath, fix);
        }
        Commands::Meta(args) => metadata_main(args),
        Commands::Tilejson(args) => tilejson_main(args),
        Commands::Copy(_args) => {
            // copy_main(args);
            warn!("copy not implemented");
            copy_main().await;
        }
        Commands::Dev {} => {
            let r = dev_main().await;
            match r {
                Ok(_) => {}
                Err(e) => {
                    error!("dev_main error: {:?}", e);
                }
            }
        }
        Commands::Rimraf(args) => {
            rimraf_main(args).await;
        }

        // mercantile cli like
        Commands::Quadkey(args) => quadkey_main(args),
        Commands::Pmtileid(args) => pmtileid_main(args),
        Commands::BoundingTile(args) => bounding_tile_main(args),
        Commands::Tiles(args) => tiles_main(args, loop_fn),
        Commands::Neighbors(args) => neighbors_main(args),
        Commands::Children(args) => children_main(args),
        Commands::Parent(args) => parent_main(args),
        Commands::Shapes(args) => {
            shapes_main(args);
        }
    }
    0
}

// not sure why this is needed... cargo thinks it's unused???
#[allow(dead_code)]
#[must_use]
pub fn cli_main_sync(argv: Option<Vec<String>>, loop_fn: Option<&dyn Fn()>) -> u8 {
    let r = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { cli_main(argv, loop_fn).await });
    r
}

#[cfg(test)]
mod tests {
    use crate::args::Cli;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
