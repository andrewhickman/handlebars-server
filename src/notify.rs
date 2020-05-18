use std::future::Future;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use fn_error_context::context;
use notify::Watcher;
use tokio::time::delay_for;

#[context("failed to watch filesystem at `{}`", path.display())]
pub fn watch<F, R>(path: &Path, on_change: F) -> Result<()>
where
    F: Fn(Vec<notify::Event>) -> R + Send + Sync + 'static,
    R: Future<Output = ()> + Send,
{
    let (sender, receiver) = crossbeam_channel::unbounded();
    let poll_interval = Duration::from_millis(150);

    let mut watcher = notify::watcher(sender, poll_interval)?;
    watcher.watch(path, notify::RecursiveMode::Recursive)?;

    tokio::spawn(async move {
        let _watcher = watcher;
        loop {
            let events: Vec<_> = receiver
                .try_iter()
                .filter_map(|event| {
                    log::trace!("got filesystem event {:?}", event);
                    match event {
                        Ok(event) => Some(event),
                        Err(err) => {
                            log::error!("error watching filesystem: {:#}", err);
                            None
                        }
                    }
                })
                .collect();
            if !events.is_empty() {
                on_change(events).await;
            }
            delay_for(poll_interval).await;
        }
    });
    Ok(())
}
