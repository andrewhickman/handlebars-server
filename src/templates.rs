use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use fn_error_context::context;
use handlebars::{Handlebars, TemplateFileError};
use tokio::sync::{broadcast, RwLock};

#[context("failed to load templates from directory: `{}`", options.base.display())]
pub fn load(
    options: &crate::Options,
    reload_tx: broadcast::Sender<()>,
) -> Result<Arc<RwLock<Handlebars<'static>>>> {
    log::info!("loading templates from directory `{}`", options.base.display());
    let handlebars = load_templates(&options.base)?;

    let handlebars = Arc::new(RwLock::new(handlebars));

    if options.watch {
        let handlebars_clone = handlebars.clone();
        let base = options.base.clone();
        if let Err(err) = crate::notify::watch(&options.base, move |events| {
            on_change(
                base.clone(),
                events,
                reload_tx.clone(),
                handlebars_clone.clone(),
            )
        }) {
            log::error!("{:#}", err);
        }
    }

    Ok(handlebars)
}

fn load_templates(path: &Path) -> Result<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".hbs", &path)
        .map_err(convert_template_file_error)?;
    handlebars.set_strict_mode(true);
    Ok(handlebars)
}

async fn on_change(
    path: PathBuf,
    events: Vec<notify::Event>,
    reload_tx: broadcast::Sender<()>,
    handlebars: Arc<RwLock<Handlebars<'static>>>,
) {
    let mut any_modified = false;
    let mut templates_modified = false;

    for event in events {
        match event.kind {
            notify::EventKind::Access(_) | notify::EventKind::Other => continue,
            _ => any_modified = true,
        }

        if event
            .paths
            .iter()
            .any(|event_path| event_path.extension() == Some("hbs".as_ref()))
        {
            templates_modified = true;
            break;
        }
    }

    if templates_modified {
        log::info!("reloading templates from directory `{}`", path.display());
        match load_templates(&path) {
            Ok(new_handlebars) => *handlebars.write().await = new_handlebars,
            Err(err) => log::error!("failed reloading files: {:#}", err),
        }
    }

    if any_modified {
        reload_tx.send(()).ok();
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
