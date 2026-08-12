use std::sync::Arc;

use log::{trace, warn};
use miette::{Context, IntoDiagnostic, NamedSource, Result};

pub struct File {
    pub(crate) name: Arc<str>,
    pub(crate) content: String,
}

impl File {
    pub fn from_file(path: Arc<str>) -> Result<File> {
        trace!("Reading file `{}`", path);
        if !path.ends_with(".skrb") {
            warn!("File `{}` does not end in .skrb", path);
        }
        let content = std::fs::read_to_string(path.as_ref())
            .into_diagnostic()
            .context(format!("While reading file `{}`", path))?;
        Ok(File {
            name: path,
            content,
        })
    }

    pub fn create_source(&self) -> NamedSource<String> {
        NamedSource::new(self.name.as_ref(), self.content.clone())
    }
}
