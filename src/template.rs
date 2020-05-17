use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use fn_error_context::context;
use handlebars::{Handlebars, TemplateFileError};
use serde_json::Value;
use tokio::sync::watch::Receiver;
use warp::{Filter as _, Reply as _};

pub fn template(
    path: &Path,
    value_rx: Receiver<Value>,
) -> Result<impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone> {
    let templates = Arc::new(load_templates(path)?);

    Ok(warp::get()
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
        }))
}

#[context("failed to load templates from directory: `{}`", path.display())]
fn load_templates(path: &Path) -> Result<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".hbs", path)
        .map_err(convert_template_file_error)?;
    handlebars.set_strict_mode(true);
    Ok(handlebars)
}

fn convert_template_file_error(err: TemplateFileError) -> anyhow::Error {
    match err {
        TemplateFileError::TemplateError(err) => err.into(),
        TemplateFileError::IOError(err, path) => {
            anyhow::Error::from(err).context(format!("failed to read file `{}`", path))
        }
    }
}

fn rsplit2<'a>(string: &'a str, pat: &str) -> Option<(&'a str, &'a str)> {
    let mut iter = string.rsplitn(2, pat);
    let last = iter.next()?;
    let init = iter.next()?;
    Some((init, last))
}
