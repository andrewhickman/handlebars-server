use std::convert::Infallible;

use futures::StreamExt as _;
use http::header::{HeaderValue, CONTENT_TYPE};
use tokio::sync::broadcast;
use warp::Filter as _;

#[derive(Debug, Copy, Clone)]
pub enum ReloadKind {
    Value,
    Page,
}

pub fn reload(
    reload_tx: broadcast::Sender<ReloadKind>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path::path("sse").and(warp::get()).and(
        warp::path::end()
            .map(move || {
                warp::sse::reply(
                    warp::sse::keep_alive().stream(
                        reload_tx
                            .subscribe()
                            .filter_map(|kind| async { kind.ok() })
                            .map(|kind| {
                                let data = match kind {
                                    ReloadKind::Value => "reload_value",
                                    ReloadKind::Page => "reload_page",
                                };
                                log::info!("sending '{}' event", data);
                                Result::<_, Infallible>::Ok(warp::sse::data(data))
                            }),
                    ),
                )
            })
            .or(warp::path!("reload.js").map(|| {
                warp::reply::with_header(
                    include_str!("reload.js"),
                    CONTENT_TYPE,
                    HeaderValue::from_static("application/javascript"),
                )
            })),
    )
}
