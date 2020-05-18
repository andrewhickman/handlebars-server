use std::io::stdin;

use anyhow::{Error, Result};
use serde_json::{Deserializer, Value};
use tokio::sync::broadcast;
use tokio::sync::watch::{channel, Receiver};

pub fn receiver(reload_tx: broadcast::Sender<()>) -> Result<Receiver<Value>> {
    let mut stream = Deserializer::from_reader(stdin()).into_iter();

    let initial_value = match stream.next() {
        Some(Ok(value)) => value,
        Some(Err(err)) => return Err(Error::new(err).context("failed to read JSON from stdin")),
        None => return Err(anyhow::format_err!("failed to read from stdin")),
    };

    let (sender, receiver) = channel(initial_value);

    tokio::task::spawn_blocking(move || {
        for value in stream {
            match value {
                Ok(value) => {
                    log::info!("got updated JSON value");
                    sender.broadcast(value).ok();
                    reload_tx.send(()).ok();
                }
                Err(err) => {
                    log::error!("failed to read JSON from stdin: {}", err);
                    return;
                }
            }
        }
        log::info!("stdin closed");
    });

    Ok(receiver)
}
