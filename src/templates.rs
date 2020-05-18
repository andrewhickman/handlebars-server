use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use fn_error_context::context;
use handlebars::{Handlebars, TemplateFileError};
use tokio::sync::RwLock;

#[context("failed to load templates from directory: `{}`", path.display())]
pub fn load(path: &Path) -> Result<Arc<RwLock<Handlebars<'static>>>> {
    log::info!("loading templates from directory `{}`", path.display());
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".hbs", path)
        .map_err(convert_template_file_error)?;
    handlebars.set_strict_mode(true);
    Ok(Arc::new(RwLock::new(handlebars)))
}

fn convert_template_file_error(err: TemplateFileError) -> anyhow::Error {
    match err {
        TemplateFileError::TemplateError(err) => err.into(),
        TemplateFileError::IOError(err, path) => {
            anyhow::Error::from(err).context(format!("failed to read file `{}`", path))
        }
    }
}
