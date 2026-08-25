use std::{collections::HashMap, path::Path, process::Command};

use log::{debug, info, trace, warn};
use miette::{Context, IntoDiagnostic, Report, Result};
use std::sync::Arc;

use crate::{
    ast::{
        nodes::FileTreeRoot,
        visitors::{
            code_generator::CodeGenerator, deprecated::DeprecatedNodesVisitor,
            unreachable::UnreachableVisitor,
        },
    },
    file::File,
    lexer::tokenise,
    parse::parse,
};

pub struct Source {
    // May be removed later as also stored in the FileTreeRoot
    // However, as it is not initialised directly (option) we may keep this
    // TODO: add first user of the file to remove this
    file: Arc<File>,
    root: FileTreeRoot,
}

fn get_root<'root, 'file: 'root>(file: Arc<File>) -> Result<FileTreeRoot> {
    let tokens = tokenise(&file.content).context("Failed to tokenise the input")?;
    let size = tokens.len();
    info!(
        "File `{}` splitted into at least {} tokens",
        file.name, size,
    );

    // Not able to log tokens without consuming them (ownership)
    parse(tokens, file.content.len())
        .map_err(|errs| errs.with_source_code(file.create_source()))
        .map(|mut root| {
            root.file = Some(file.clone());
            root
        })
}

impl Source {
    pub fn new(file: Arc<File>) -> Result<Source> {
        trace!("Entenring source creation for `{}`", file.name);
        let root = get_root(file.clone())?;
        Ok(Source { file, root })
    }

    pub fn compile(&mut self, folder: &str) -> Result<()> {
        // Placeholder for later checks
        // May be moved later to the new function
        // Only do not do too much on a pull request
        if let Some(error) = DeprecatedNodesVisitor::find(&self.root)? {
            let report: Report = error.into();
            let report = report.with_source_code(self.file.create_source());
            warn!("Warning: {:?}", report);
        }

        if let Some(error) = UnreachableVisitor::find(&mut self.root)? {
            let report: Report = error.into();
            let report = report.with_source_code(self.file.create_source());
            warn!("Warning: {:?}", report);
        }

        CodeGenerator::compile(&self.root, &self.file.name, folder)
            .context(format!("While compiling file `{}`", self.file.name))
    }
}

pub struct SourceManager {
    files: HashMap<Arc<str>, Source>,
}

fn link_files(inputs: Vec<String>, output: &str) -> Result<()> {
    let output = Path::new(output);
    let command = Command::new("clang")
        // For nix
        .arg("-Wno-unused-command-line-argument")
        .args(["-o", output.to_str().expect("Cannot create output")])
        .args(inputs)
        .status();
    command.into_diagnostic().context("While linking files")?;
    Ok(())
}

impl SourceManager {
    pub fn empty() -> Self {
        SourceManager {
            files: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, file: Arc<File>) -> Result<()> {
        debug!("Adding file {} into source files", file.name);
        self.files.insert(
            file.name.clone(),
            Source::new(file).context("Failed to parse the file")?,
        );
        Ok(())
    }

    pub fn compile(&mut self, folder: &str, output: &str) -> Result<()> {
        trace!("Start compiling sources");
        let mut paths = vec![];
        for (name, file) in &mut self.files {
            info!("Compiling `{}`", name);
            file.compile(folder)?;
            let name = Path::new(".skribi")
                .join(name.as_ref())
                .with_added_extension("ll")
                .to_str()
                .context("Compiled file has an invalid name")?
                .to_owned();
            paths.push(name);
        }
        link_files(paths, output)
            .context(format!("After building all files needed for {}", output))?;
        info!("Result saved into {}", output);
        Ok(())
    }

    pub fn pretty(&self) -> Result<()> {
        for (name, file) in &self.files {
            std::println!("File {} AST is:\n{}", name, file.root);
        }
        Ok(())
    }
}
