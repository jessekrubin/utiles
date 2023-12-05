use std::io::{self};

use clap::Parser;
use tracing::{debug, warn};
use tracing_subscriber::fmt::{self};
use tracing_subscriber::EnvFilter;

use crate::args::{Cli, Commands};
use crate::commands::copy::copy_main;
use crate::commands::dev::dev_main;
use crate::commands::lint::lint_main;
use crate::commands::rimraf::rimraf_main;
use crate::commands::shapes::shapes_main;
use crate::commands::tiles::tiles_main;
use crate::commands::{
    bounding_tile_main, contains_main, metadata_main, neighbors_main, pmtileid_main,
    quadkey_main, tilejson_main,
};
use crate::commands::{children_main, parent_main};

fn init_tracing(debug: bool) {
    let filter = if debug {
        EnvFilter::new("DEBUG")
    } else {
        EnvFilter::new("INFO")
    };
    let subscriber = fmt::Subscriber::builder()
        .compact()
        .with_target(true)
        .with_line_number(false)
        .with_env_filter(filter)
        .with_writer(io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("tracing::subscriber::set_global_default(...) failed.");
}

#[allow(clippy::unused_async)]
pub async fn cli_main(argv: Option<Vec<String>>, loop_fn: Option<&dyn Fn()>) -> u8 {
    // print args
    let argv = match argv {
        Some(argv) => argv,
        None => std::env::args().collect::<Vec<_>>(),
    };
    let args = Cli::parse_from(&argv);

    // if the command is "dev" init tracing w/ debug
    if let Commands::Dev(_) = args.command {
        init_tracing(true);
    } else {
        init_tracing(args.debug);
    }

    debug!("args: {:?}", std::env::args().collect::<Vec<_>>());
    debug!("argv: {:?}", argv);
    debug!("args: {:?}", args);

    match args.command {
        Commands::Lint(args) => {
            if args.fix {
                warn!("fix not implemented");
            }
            lint_main(&args);
        }
        Commands::Meta(args) => metadata_main(&args),
        Commands::Tilejson(args) => tilejson_main(&args),
        Commands::Copy(args) => {
            // copy_main(args);
            copy_main(args).await;
        }
        Commands::Dev(args) => {
            let _r = dev_main(args).await;
        }
        Commands::Rimraf(args) => {
            rimraf_main(args).await;
        }
        Commands::Contains { filepath, lnglat } => contains_main(&filepath, lnglat),
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
        Cli::command().debug_assert();
    }
}
