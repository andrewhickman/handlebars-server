mod notify;
mod reload;
mod render;
mod server;
mod templates;
mod value;

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use structopt::StructOpt;
use tokio::runtime::Runtime;
use tokio::sync::broadcast;
use warp::Filter as _;

use self::reload::reload;
use self::render::render;

const VERSION: &'static str = concat!(clap::crate_version!(), " (", env!("VERGEN_SHA_SHORT"), ")");
const LONG_VERSION: &'static str = concat!(clap::crate_version!(), " (", env!("VERGEN_SHA"), ")");

/// A simple server that generates HTML at runtime, based on JSON values piped to stdin.
#[derive(Debug, StructOpt)]
#[structopt(version = VERSION, long_version = LONG_VERSION)]
#[structopt(setting = clap::AppSettings::UnifiedHelpMessage)]
pub struct Options {
    #[structopt(flatten)]
    server: server::Options,
    #[structopt(value_name = "BASE_DIR", help = "Base directory", default_value = ".", parse(try_from_os_str = parse_dir))]
    base: PathBuf,
}

fn main() {
    let mut runtime = Runtime::new().unwrap();

    env_logger::init_from_env(env_logger::Env::new().filter_or("HANDLEBARS_SERVER_LOG", "info"));
    if let Err(err) = runtime.block_on(run()) {
        log::error!("Fatal error: {:#}", err);
    }

    runtime.shutdown_timeout(Duration::from_secs(0))
}

async fn run() -> Result<()> {
    let options = Options::from_args();
    log::debug!("{:#?}", options);

    let (reload_tx, _) = broadcast::channel(1);

    let templates = templates::load(options.base.clone(), reload_tx.clone())?;

    log::info!("reading JSON value from stdin");
    let value_rx = value::receiver(reload_tx.clone())?;

    server::run(
        &options.server,
        reload(reload_tx)
            .or(render(templates, value_rx))
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
