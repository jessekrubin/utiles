use std::io::{self};

use clap::Parser;
use tracing::debug;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::EnvFilter;

use crate::cli::args::{Cli, Commands};
use crate::cli::commands::{
    bounding_tile_main, children_main, contains_main, copy_main, dev_main, lint_main,
    mbtiles_info_main, metadata_main, metadata_set_main, neighbors_main, parent_main,
    pmtileid_main, quadkey_main, rimraf_main, shapes_main, tilejson_main, tiles_main,
    touch_main, vacuum_main,
};

struct LogConfig {
    pub debug: bool,
    pub json: bool,
}

fn init_tracing(log_config: &LogConfig) {
    let filter = if log_config.debug {
        EnvFilter::new("DEBUG")
    } else {
        EnvFilter::new("INFO")
    };
    #[allow(clippy::match_bool)]
    match log_config.json {
        true => {
            let subscriber = fmt::Subscriber::builder()
                .json()
                .with_env_filter(filter)
                .with_writer(io::stderr)
                .finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("tracing::subscriber::set_global_default(...) failed.");
        }
        false => {
            let subscriber = fmt::Subscriber::builder()
                .with_env_filter(filter)
                .with_writer(io::stderr)
                .finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("tracing::subscriber::set_global_default(...) failed.");
        }
    }
}

#[allow(clippy::unused_async)]
pub async fn cli_main(argv: Option<Vec<String>>, loop_fn: Option<&dyn Fn()>) -> u8 {
    // print args
    let argv = argv.unwrap_or_else(|| std::env::args().collect::<Vec<_>>());

    // set caller if provided
    let args = Cli::parse_from(&argv);

    // if the command is "dev" init tracing w/ debug
    let logcfg = if let Commands::Dev(_) = args.command {
        LogConfig {
            debug: true,
            json: args.log_json,
        }
    } else {
        LogConfig {
            debug: args.debug,
            json: args.log_json,
        }
    };
    init_tracing(&logcfg);
    debug!("args: {:?}", std::env::args().collect::<Vec<_>>());
    debug!("argv: {:?}", argv);
    debug!("args: {:?}", args);

    match args.command {
        Commands::Lint(args) => lint_main(&args),
        Commands::Touch(args) => {
            touch_main(&args).unwrap();
        }
        Commands::Vacuum(args) => {
            vacuum_main(&args);
        }
        Commands::Metadata(args) => metadata_main(&args),
        Commands::MetadataSet(args) => metadata_set_main(&args),
        Commands::Tilejson(args) => tilejson_main(&args),
        Commands::Copy(args) => {
            copy_main(args).await;
        }
        Commands::Mbinfo(args) => mbtiles_info_main(&args),
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
        Commands::Shapes(args) => shapes_main(args),
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
    use crate::cli::args::Cli;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
