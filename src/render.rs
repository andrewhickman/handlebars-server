use std::sync::Arc;

use handlebars::Handlebars;
use serde_json::Value;
use tokio::sync::watch::Receiver;
use tokio::sync::RwLock;
use warp::{Filter as _, Reply as _};

pub fn render(
    templates: Arc<RwLock<Handlebars<'static>>>,
    value_rx: Receiver<Value>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path::full())
        .and_then(move |path: warp::path::FullPath| {
            let templates = templates.clone();
            let value_rx = value_rx.clone();
            async move {
                let path = match urlencoding::decode(path.as_str()) {
                    Ok(path) => path,
                    Err(_) => return Err(warp::reject::not_found()),
                };
                let (path, file) = match rsplit2(&path, "/") {
                    Some(split) => split,
                    None => return Err(warp::reject::not_found()),
                };

                let templates = templates.read().await;

                let name = match rsplit2(file, ".") {
                    Some((name, "html")) if templates.has_template(name) => name,
                    Some((_, "hbs")) => return Ok(http::StatusCode::NOT_FOUND.into_response()),
                    _ => return Err(warp::reject::not_found()),
                };

                let value = value_rx.borrow();
                let subvalue = match value.pointer(path) {
                    Some(subvalue) => subvalue,
                    None => {
                        log::warn!("pointer error: {}", path);
                        return Ok(http::StatusCode::NOT_FOUND.into_response());
                    }
                };

                let result = match templates.render(name, subvalue) {
                    Ok(result) => result,
                    Err(err) => {
                        log::error!("template error: {}", err);
                        return Ok(http::StatusCode::NOT_FOUND.into_response());
                    }
                };

                Ok(warp::reply::html(result).into_response())
            }
        })
}

fn rsplit2<'a>(string: &'a str, pat: &str) -> Option<(&'a str, &'a str)> {
    let mut iter = string.rsplitn(2, pat);
    let last = iter.next()?;
    let init = iter.next()?;
    Some((init, last))
}
