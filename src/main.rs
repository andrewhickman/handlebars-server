mod reload;
mod server;
mod template;
mod value;

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use structopt::{StructOpt};

use anyhow::Result;
use warp::Filter as _;

use self::reload::reload;
use self::template::template;

const VERSION: &'static str = concat!(clap::crate_version!(), " (", env!("VERGEN_SHA_SHORT"), ")");
const LONG_VERSION: &'static str = concat!(clap::crate_version!(), " (", env!("VERGEN_SHA"), ")");

/// A simple server that generates HTML at runtime, based on JSON values piped to stdin.
#[derive(Debug, StructOpt)]
#[structopt(version = VERSION, long_version = LONG_VERSION)]
#[structopt(setting = clap::AppSettings::UnifiedHelpMessage)]
pub struct Options {
    #[structopt(flatten)]
    server: server::Options,
    #[structopt(value_name = "BASE_DIR", help = "Base directory", parse(try_from_os_str = parse_dir))]
    base: PathBuf,
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().filter_or("HANDLEBARS_SERVER_LOG", "warn"));
    if let Err(err) = run().await {
        log::error!("Fatal error: {:#}", err);
    }
}

async fn run() -> Result<()> {
    let options = Options::from_args();
    log::debug!("{:#?}", options);

    log::info!("Reading JSON value from stdin");
    let value_rx = value::receiver()?;

    server::run(
        &options.server,
        reload(value_rx.clone())
            .or(template(&options.base, value_rx)?)
            .or(warp::fs::dir(options.base))
            .with(warp::log(module_path!())),
    )
    .await
}

fn parse_dir(value: &OsStr) -> Result<PathBuf, OsString> {
    let path = PathBuf::from(value);
    if path.is_dir() {
        Ok(path)
    } else {
        Err(format!("`{}` is not a directory", path.display()).into())
    }
}
