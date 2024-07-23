use crate::cli::args::InfoArgs;
use crate::errors::UtilesResult;
use crate::mbt::mbinfo;

pub async fn info_main(args: &InfoArgs) -> UtilesResult<()> {
    let stats = mbinfo(&args.common.filepath).await?;
    let str = if args.common.min {
        serde_json::to_string(&stats)
    } else {
        serde_json::to_string_pretty(&stats)
    }?;
    println!("{str}");
    Ok(())
}
