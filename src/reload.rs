use std::convert::Infallible;

use futures::StreamExt as _;
use http::header::{HeaderValue, CONTENT_TYPE};
use warp::Filter as _;

pub fn reload(
    reload_stream: impl futures::Stream + Send + Sync + Clone,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path::path("sse").and(warp::get()).and(
        warp::path::end()
            .map(move || {
                warp::sse::reply(
                    warp::sse::keep_alive().stream(reload_stream.clone().map(|_| {
                        log::debug!("sending reload event");
                        Result::<_, Infallible>::Ok(warp::sse::data("reload"))
                    })),
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
