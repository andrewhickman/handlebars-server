mod server;
mod template;

use std::path::PathBuf;

use structopt::StructOpt;

use anyhow::Result;
use warp::Filter as _;

use self::template::template;

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(flatten)]
    server: server::Options,
    #[structopt(value_name = "BASE_DIR", help = "Base directory", parse(from_os_str))]
    base: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let options = Options::from_args();
    log::debug!("{:#?}", options);

    server::run(
        &options.server,
        template(&options.base)?
            .or(warp::fs::dir(options.base))
            .with(warp::log(module_path!())),
    )
    .await
}
