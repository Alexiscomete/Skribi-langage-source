use std::{collections::HashMap, path::Path, process::Command};

use chumsky::error::Rich;
use log::{debug, info, trace, warn};
use miette::{
    Context, Diagnostic, IntoDiagnostic, LabeledSpan, NamedSource, Report, Result, SourceSpan,
};
use thiserror::Error;

use crate::{
    ast::{
        nodes::FileTreeRoot,
        visitors::{code_generator::CodeGenerator, deprecated::DeprecatedNodesVisitor},
    },
    file::File,
    lexer::{Tokens, tokenise},
    parse::parse,
};

pub struct Source<'file> {
    // TODO: add first user of the file to remove this
    #[allow(dead_code)]
    file: &'file File<'file>,
    root: FileTreeRoot<'file>,
}

#[derive(Error, Debug, Diagnostic)]
#[error("{message}")]
#[diagnostic()]
struct ParsingSingleError {
    message: String,
    #[label(primary, "{span_message}")]
    span: SourceSpan,
    span_message: String,
    #[label(collection)]
    spans: Vec<LabeledSpan>,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Parsing error")]
#[diagnostic(help("Always try to fix the first parsing error as they might be cascades"))]
struct ParsingErrors {
    #[source_code]
    src: NamedSource<String>,
    #[related]
    related: Vec<ParsingSingleError>,
}

fn convert_to_err(file: &File<'_>, errs: Vec<Rich<'_, Tokens<'_>>>) -> ParsingErrors {
    // Greatly inspired from
    // https://codeberg.org/zesterer/chumsky/src/branch/main/examples/nano_rust.rs
    ParsingErrors {
        src: file.create_source(),
        related: errs
            .iter()
            .map(|err| ParsingSingleError {
                message: err.to_string(),
                span: err.span().into_range().into(),
                span_message: err.reason().to_string(),
                spans: err
                    .contexts()
                    .map(|(label, span)| {
                        LabeledSpan::new_with_span(
                            Some(format!("parsing {label}")),
                            span.into_range(),
                        )
                    })
                    .collect(),
            })
            .collect(),
    }
}

impl Source<'_> {
    pub fn new<'file>(file: &'file File<'file>) -> Result<Source<'file>> {
        trace!("Entenring source creation for `{}`", file.name);
        let tokens = tokenise(&file.content);
        let size = tokens.size_hint();
        info!(
            // In general, 0 is detected as we have an indefinite size
            // The tokens are parsed on demand I suppose
            "File `{}` splitted into at least {} tokens",
            file.name, size.0,
        );

        // Not able to log tokens without consuming them (ownership)
        let result = parse(tokens, file.content.len());
        match result {
            Ok(root) => Ok(Source { file, root }),
            Err(errs) => Err(convert_to_err(file, errs).into()),
        }
    }

    pub fn compile(&self, folder: &str) -> Result<()> {
        // Placeholder for later checks
        // May be moved later to the new function
        // Only do not do too much on a pull request
        if let Some(error) = DeprecatedNodesVisitor::find(&self.root)? {
            let report: Report = error.into();
            warn!("Warning: {:?}", report);
        }

        CodeGenerator::compile(&self.root, self.file.name, folder)
            .context(format!("While compiling file `{}`", self.file.name))
    }
}

pub struct SourceManager<'sources> {
    files: HashMap<&'sources str, Source<'sources>>,
}

fn link_files(inputs: Vec<String>, output: &str) -> Result<()> {
    let output = Path::new(output).with_added_extension("out");
    let command = Command::new("clang")
        // For nix
        .arg("-Wno-unused-command-line-argument")
        .args(["-o", output.to_str().expect("Cannot create output")])
        .args(inputs)
        .status();
    command.into_diagnostic().context("While linking files")?;
    Ok(())
}

impl<'manager> SourceManager<'manager> {
    pub fn empty() -> Self {
        SourceManager {
            files: HashMap::new(),
        }
    }

    pub fn add_file<'file: 'manager>(&mut self, file: &'file File<'file>) -> Result<()> {
        debug!("Adding file {} into source files", file.name);
        self.files.insert(file.name, Source::new(file)?);
        Ok(())
    }

    pub fn compile(&self, folder: &str, output: &str) -> Result<()> {
        trace!("Start compiling sources");
        let mut paths = vec![];
        for (name, file) in &self.files {
            info!("Compiling `{}`", name);
            file.compile(folder)?;
            let name = Path::new(".skribi")
                .join(name)
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
