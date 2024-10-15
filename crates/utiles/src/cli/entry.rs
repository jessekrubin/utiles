use crate::cli::args::{Cli, Commands};
use crate::cli::commands::{
    about_main, addo_main, agg_hash_main, bounding_tile_main, burn_main, children_main,
    commands_main, contains_main, copy_main, dev_main, edges_main, enumerate_main,
    fmtstr_main, info_main, lint_main, metadata_main, metadata_set_main,
    neighbors_main, optimize_main, parent_main, pmtileid_main, quadkey_main,
    rimraf_main, serve_main, shapes_main, tilejson_main, tiles_main, touch_main,
    translate_main, update_main, vacuum_main, webpify_main, zxyify_main,
};
use crate::errors::UtilesResult;
use crate::lager::{init_tracing, LagerConfig};
use crate::signal::shutdown_signal;
use crate::UtilesError;
use clap::{CommandFactory, FromArgMatches};
use tracing::{debug, error};
use utiles_core::VERSION;

pub struct CliOpts {
    pub argv: Option<Vec<String>>,
    pub clid: Option<&'static str>,
}

impl Default for CliOpts {
    fn default() -> Self {
        Self {
            argv: None,
            clid: Option::from("rust"),
        }
    }
}

impl CliOpts {
    #[must_use]
    pub fn aboot_str(&self) -> String {
        format!(
            "utiles cli ({}) ~ v{}",
            self.clid.unwrap_or("rust"),
            VERSION
        )
    }
}

pub async fn cli_main(cliops: Option<CliOpts>) -> UtilesResult<u8> {
    tokio::select! {
        res = async {
            cli_main_inner(
                cliops
            ).await
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
pub async fn cli_main_inner(cliopts: Option<CliOpts>) -> UtilesResult<u8> {
    // print args
    let opts = cliopts.unwrap_or_default();
    let argv = opts.argv.unwrap_or_else(|| std::env::args().collect());
    let about_str = format!(
        "utiles cli ({}) ~ v{}",
        opts.clid.unwrap_or("rust"),
        VERSION
    );

    // set caller if provided
    let cli = Cli::command().about(about_str);
    let matches = cli.get_matches_from(
        // argv.clone()
        &argv,
    );
    let args = Cli::from_arg_matches(&matches).expect("from_arg_matches failed");

    // if the command is "dev" init tracing w/ debug
    let logcfg = if let Commands::Dev(_) = args.command {
        LagerConfig {
            trace: false,
            debug: true,
            json: args.log_json,
        }
    } else {
        LagerConfig {
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
        Commands::Commands(args) => {
            let c = Cli::command();
            commands_main(&c, &args)
        }
        Commands::Sqlite(dbcmds) => dbcmds.run().await,
        Commands::Lint(args) => lint_main(&args).await,
        Commands::Touch(args) => touch_main(&args).await,
        Commands::Vacuum(args) => vacuum_main(&args).await,
        Commands::Metadata(args) => metadata_main(&args).await,
        Commands::MetadataSet(args) => metadata_set_main(&args).await,
        Commands::Update(args) => update_main(&args).await,
        Commands::Tilejson(args) => tilejson_main(&args).await,
        Commands::Copy(args) => copy_main(args).await,
        Commands::Info(args) => info_main(&args).await,
        Commands::AggHash(args) => agg_hash_main(&args).await,
        Commands::Dev(args) => dev_main(args).await,
        Commands::Rimraf(args) => rimraf_main(args).await,
        Commands::Contains { filepath, lnglat } => {
            contains_main(&filepath, lnglat).await
        }
        Commands::Enumerate(args) => enumerate_main(&args).await,

        Commands::Burn(args) => burn_main(args).await,
        Commands::Edges(args) => edges_main(args).await,
        Commands::Zxyify(args) => zxyify_main(args).await,
        // mercantile cli like
        Commands::Fmt(args) => fmtstr_main(args),
        Commands::Quadkey(args) => quadkey_main(args),
        Commands::Pmtileid(args) => pmtileid_main(args),
        Commands::BoundingTile(args) => bounding_tile_main(args),
        Commands::Tiles(args) => tiles_main(args, None).await,
        Commands::Neighbors(args) => neighbors_main(args),
        Commands::Children(args) => children_main(args),
        Commands::Parent(args) => parent_main(args),
        Commands::Shapes(args) => shapes_main(args),
        Commands::Optimize(args) => optimize_main(args).await,
        Commands::Webpify(args) => webpify_main(args).await,
        // server WIP
        Commands::Serve(args) => serve_main(args).await,
        // unimplemented
        Commands::Addo => addo_main(None).await,
        Commands::Translate => translate_main(None).await,
    };

    match res {
        Ok(()) => Ok(0),
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
    }
}

// not sure why this is needed... cargo thinks it's unused???
pub fn cli_main_sync(opts: Option<CliOpts>) -> UtilesResult<u8> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect(
            "tokio::runtime::Builder::new_multi_thread().enable_all().build() failed.",
        )
        .block_on(async { cli_main(opts).await })
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
