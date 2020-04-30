use std::net::{SocketAddr, ToSocketAddrs};

use anyhow::{Context, Result};
use structopt::StructOpt;
use warp::reject::Rejection;

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(
        long,
        short = "n",
        value_name = "HOST",
        default_value = "localhost",
        help = "Hostname to listen on for HTTP connections"
    )]
    hostname: String,
    #[structopt(
        long,
        short = "p",
        value_name = "PORT",
        default_value = "3000",
        help = "Port to use for HTTP connections"
    )]
    port: u16,
}

pub async fn run<F>(options: &Options, filter: F) -> anyhow::Result<()>
where
    F: warp::Filter<Error = Rejection> + Clone + Send + Sync + 'static,
    F::Extract: warp::Reply,
{
    let addr = options.resolve_hostname()?;

    let server = warp::serve(filter);

    let (addr, run) = server
        .try_bind_ephemeral(addr)
        .with_context(|| format!("failed to bind to `{}`", addr))?;
    log::info!("listening on http://{}", addr);

    Ok(run.await)
}

impl Options {
    fn resolve_hostname(&self) -> Result<SocketAddr> {
        let error_message = || format!("failed to resolve hostname `{}`", self.hostname);
        Ok((self.hostname.as_ref(), self.port)
            .to_socket_addrs()
            .with_context(error_message)?
            .next()
            .with_context(error_message)?)
    }
}
