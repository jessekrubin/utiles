use crate::UtilesError;
use crate::cli::args::{Cli, Commands};
use crate::cli::commands::{
    about_main, addo_main, agg_hash_main, bounding_tile_main, burn_main, children_main,
    commands_main, contains_main, copy_main, dev_main, edges_main, enumerate_main,
    fmtstr_main, info_main, lint_main, merge_main, metadata_main, metadata_set_main,
    neighbors_main, optimize_main, parent_main, pmtileid_main, quadkey_main,
    rimraf_main, serve_main, shapes_main, tilejson_main, tiles_main, touch_main,
    translate_main, update_main, vacuum_main, webpify_main, zxyify_main,
};
use crate::errors::UtilesResult;
use crate::internal::signal::shutdown_signal;
use crate::lager::{LagerConfig, LagerLevel, init_tracing};
use clap::{CommandFactory, FromArgMatches};
use tracing::{debug, error, trace};
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

pub(crate) async fn cli_main_inner(cliopts: Option<CliOpts>) -> UtilesResult<u8> {
    // print args
    let opts = cliopts.unwrap_or_default();
    let argv = opts.argv.unwrap_or_else(|| std::env::args().collect());
    let about_str = format!(
        "utiles cli ({}) ~ v{}",
        opts.clid.unwrap_or("rust"),
        VERSION
    );

    // check for `UTILES_MAX_TERM_WIDTH` env var and use it if it's set
    // otherwise use the default value of 120
    let max_term_width = {
        if let Ok(val) = std::env::var("UTILES_MAX_TERM_WIDTH") {
            val.parse::<usize>().unwrap_or(120)
        } else {
            120
        }
    };
    // set caller if provided
    let cli = Cli::command()
        .about(about_str)
        .max_term_width(max_term_width);
    let matches = cli.get_matches_from(
        // argv.clone()
        &argv,
    );
    let args = Cli::from_arg_matches(&matches).expect("from_arg_matches failed");

    // if the command is "dev" init tracing w/ debug
    let logcfg = if let Commands::Dev(_) = args.command {
        LagerConfig {
            level: LagerLevel::Debug,
            json: args.log_json,
        }
    } else {
        let level = if args.trace {
            LagerLevel::Trace
        } else if args.debug {
            LagerLevel::Debug
        } else {
            LagerLevel::Info
        };
        LagerConfig {
            level,
            json: args.log_json,
        }
    };
    init_tracing(logcfg)?;

    trace!(
        args = ?args,
        argv = ?argv,
    );

    let ti = std::time::Instant::now();
    let res: UtilesResult<()> = match args.command {
        Commands::About(args) => about_main(&args),
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
        Commands::Merge(args) => merge_main(args).await,
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
    let elapsed = ti.elapsed();
    let signed_duration = jiff::SignedDuration::try_from(elapsed)
        .expect("jiff::SignedDuration::try_from failed");
    trace!("utiles-cli-finished ~ {:#}", signed_duration);
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
