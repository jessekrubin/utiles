use std::io::{self};

use clap::Parser;
use tracing::{debug, error, warn};
use tracing_subscriber::fmt::{self};
use tracing_subscriber::EnvFilter;

use crate::cli::args::{Cli, Commands};
use crate::cli::commands::{
    about_main, bounding_tile_main, children_main, contains_main, copy_main, dev_main,
    fmtstr_main, info_main, lint_main, metadata_main, metadata_set_main,
    neighbors_main, parent_main, pmtileid_main, quadkey_main, rimraf_main, serve_main,
    shapes_main, tilejson_main, tiles_main, touch_main, update_main, vacuum_main,
};
use crate::errors::UtilesResult;
use crate::signal::shutdown_signal;
use crate::UtilesError;

struct LogConfig {
    pub debug: bool,
    pub trace: bool,
    pub json: bool,
}

fn init_tracing(log_config: &LogConfig) -> UtilesResult<()> {
    let filter = if log_config.trace {
        EnvFilter::new("TRACE")
    } else if log_config.debug {
        EnvFilter::new("DEBUG")
    } else {
        EnvFilter::new("INFO")
    };
    let debug_or_trace = log_config.debug || log_config.trace;
    #[allow(clippy::match_bool)]
    match log_config.json {
        true => {
            let subscriber = fmt::Subscriber::builder()
                .json()
                .with_env_filter(filter)
                .with_writer(io::stderr)
                .finish();
            let set_global_res = tracing::subscriber::set_global_default(subscriber);
            if let Err(e) = set_global_res {
                warn!("tracing::subscriber::set_global_default(...) failed: {}", e);
            }
        }
        false => {
            let subscriber = fmt::Subscriber::builder()
                .compact()
                .with_env_filter(filter)
                .with_writer(io::stderr)
                .with_target(debug_or_trace)
                .finish();
            let set_global_res = tracing::subscriber::set_global_default(subscriber);
            if let Err(e) = set_global_res {
                warn!("tracing::subscriber::set_global_default(...) failed: {}", e);
            }
        }
    }
    Ok(())
}

pub async fn cli_main(
    argv: Option<Vec<String>>,
    // loop_fn: Option<&dyn Fn()>,
) -> UtilesResult<u8> {
    tokio::select! {
        res = async {
            cli_main_inner(argv).await
        } => {
            debug!("Done. :)");
            res
        }
        () = async {
            shutdown_signal().await;
        } => {
            debug!("Aborted. :(");
            Err(
                UtilesError::Error(
                    "Aborted.".to_string()
                )
            )
        }
    }
}

#[allow(clippy::unused_async)]
pub async fn cli_main_inner(argv: Option<Vec<String>>) -> UtilesResult<u8> {
    // print args
    let argv = argv.unwrap_or_else(|| std::env::args().collect::<Vec<_>>());

    // set caller if provided
    let args = Cli::parse_from(&argv);

    // if the command is "dev" init tracing w/ debug
    let logcfg = if let Commands::Dev(_) = args.command {
        LogConfig {
            trace: false,
            debug: true,
            json: args.log_json,
        }
    } else {
        LogConfig {
            trace: args.trace,
            debug: args.debug,
            json: args.log_json,
        }
    };
    init_tracing(&logcfg)?;
    debug!("args: {:?}", std::env::args().collect::<Vec<_>>());
    debug!("argv: {:?}", argv);
    debug!("args: {:?}", args);

    let res: UtilesResult<()> = match args.command {
        Commands::About => about_main(),
        Commands::Lint(args) => lint_main(&args).await,
        Commands::Touch(args) => touch_main(&args),
        Commands::Vacuum(args) => vacuum_main(&args),
        Commands::Metadata(args) => metadata_main(&args),
        Commands::MetadataSet(args) => metadata_set_main(&args),
        Commands::Update(args) => update_main(&args).await,
        Commands::Tilejson(args) => tilejson_main(&args),
        Commands::Copy(args) => copy_main(args).await,
        Commands::Info(args) => info_main(&args),
        Commands::Dev(args) => dev_main(args).await,
        Commands::Rimraf(args) => rimraf_main(args).await,
        Commands::Contains { filepath, lnglat } => {
            contains_main(&filepath, lnglat).await
        }
        // mercantile cli like
        Commands::Fmt(args) => fmtstr_main(args),
        Commands::Quadkey(args) => quadkey_main(args),
        Commands::Pmtileid(args) => pmtileid_main(args),
        Commands::BoundingTile(args) => bounding_tile_main(args),
        Commands::Tiles(args) => tiles_main(args, None).await,
        // Commands::Tiles(args) => sleep(args, None).await,
        Commands::Neighbors(args) => neighbors_main(args),
        Commands::Children(args) => children_main(args),
        Commands::Parent(args) => parent_main(args),
        Commands::Shapes(args) => shapes_main(args),
        // Commands::Webpify(args) => webpify_main(args),
        // server WIP
        Commands::Serve(args) => serve_main(args).await,
    };

    match res {
        Ok(()) => Ok(0),
        Err(e) => {
            error!("Error: {}", e);
            Err(e)
        }
    }
}

// not sure why this is needed... cargo thinks it's unused???
pub fn cli_main_sync(
    argv: Option<Vec<String>>,
    // loop_fn: Option<&dyn Fn()>,
) -> UtilesResult<u8> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect(
            "tokio::runtime::Builder::new_multi_thread().enable_all().build() failed.",
        )
        .block_on(async { cli_main(argv).await })
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
