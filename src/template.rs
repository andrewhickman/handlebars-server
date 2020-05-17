use std::path::Path;

use anyhow::Result;
use fn_error_context::context;
use handlebars::{Handlebars, TemplateFileError};
use warp::{Filter as _, Reply as _};

const TEMPLATE_EXTENSION: &'static str = "hbs";

pub fn template(
    path: &Path,
) -> Result<impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone> {
    let matcher = Matcher::new(path)?;

    Ok(warp::get()
        .and(warp::path::full())
        .and_then(|path: warp::path::FullPath| async move {
            let (path, file) = match rsplit2(path.as_str(), "/") {
                Some(split) => split,
                None => return Err(warp::reject::not_found()),
            };
            let name = match rsplit2(file, ".") {
                Some((name, "html")) => name,
                Some((name, TEMPLATE_EXTENSION)) => return Ok(http::StatusCode::NOT_FOUND.into_response()),
                _ => return Err(warp::reject::not_found()),
            };

            Result::<_, warp::Rejection>::Ok(warp::reply().into_response())
        }))
}

struct Matcher {
    handlebars: Handlebars<'static>,
}

impl Matcher {
    #[context("failed to load templates from directory `{}`", path.display())]
    fn new(path: &Path) -> Result<Self> {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_templates_directory(TEMPLATE_EXTENSION, path)
            .map_err(convert_template_file_error)?;

        Ok(Matcher { handlebars })
    }
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
