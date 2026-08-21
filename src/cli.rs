use std::{fs::create_dir_all, sync::Arc};

use log::{LevelFilter, info, trace};

use crate::file::File;
use crate::source::SourceManager;

use clap::Parser;
use miette::{Context, IntoDiagnostic, Result};

#[derive(Parser, Debug)]
pub(crate) struct Build {
    /// The source file to use. Defaults to STDIN.
    /// STDIN is currently not supported.
    pub(crate) source: Option<Arc<str>>,
    /// Sets the path of the compilation folder.
    #[arg(short, long, default_value = ".skribi")]
    compile_path: String,
    /// Sets the name of the output program.
    #[arg(short, long, default_value = ".skribi/result.out")]
    output: String,
}

/// Creates a folder to store everything
fn create_skribi_directory(path: &str) -> Result<()> {
    trace!("About to create directory `{}`", path);
    create_dir_all(path).into_diagnostic().context(format!(
        "While creating `{}` directory to store compiled files",
        path
    ))?;
    info!("Directory `{}` created for compiled files", path);
    Ok(())
}

impl Build {
    /// Compile the source code
    pub(crate) fn execute(self) -> Result<()> {
        self.action(|build, manager| manager.compile(&build.compile_path, &build.output))
    }

    pub(crate) fn action(self, action: fn(Build, SourceManager) -> Result<()>) -> Result<()> {
        create_skribi_directory(&self.compile_path)?;

        if let Some(path) = self.source.clone() {
            let file =
                Arc::new(File::from_file(path).context("While reading file passed as argument")?);
            let mut manager = SourceManager::empty();
            manager.add_file(file)?;

            action(self, manager)
        } else {
            todo!("STDIN is currently not supported")
        }
    }
}

#[derive(Parser, Debug)]
pub(crate) struct Run {
    #[command(flatten)]
    pub(crate) build: Build,
}

impl Run {
    /// Compile the source code, then execute the compiled code
    pub(crate) fn execute(self) -> Result<()> {
        self.build.execute()?;
        todo!("Execute the compiled code")
    }
}

#[derive(Parser, Debug)]
pub(crate) struct Pretty {
    #[command(flatten)]
    pub(crate) build: Build,
}

impl Pretty {
    /// Pretty print the code instead of compiling it
    pub(crate) fn execute(self) -> Result<()> {
        self.build.action(|_, manager| manager.pretty())
    }
}

#[derive(Parser, Debug)]
pub(crate) enum Command {
    /// Build the source code into machine code
    Build(Build),
    /// Build the source code and run it directly after
    Run(Run),
    /// Pretty print the code instead of compiling it
    Pretty(Pretty),
}

impl Command {
    /// Run the subcommand's specific code
    pub(crate) fn execute(self) -> Result<()> {
        match self {
            Command::Build(build) => build.execute(),
            Command::Run(run) => run.execute(),
            Command::Pretty(pretty) => pretty.execute(),
        }
    }
}

/// The Skribi compiler CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Arguments {
    /// Log more information. Fine-grained control.
    ///
    /// The SKRIBI_C_LOG variable can also be used.
    /// To specify a style, use SKRIBI_C_LOG_STYLE.
    /// The variable is overriden by the argument.
    /// With nothing set, defaults to warn.
    ///
    /// Possible values: off, error, warn, info, debug, trace
    #[arg(short, long, global = true)]
    pub(crate) verbose: Option<LevelFilter>,
    #[clap(subcommand)]
    pub cmd: Command,
}

#[cfg(test)]
mod test {
    use std::{io::Write, path::PathBuf, process::Command};

    use tempfile::{NamedTempFile, TempDir, tempdir};

    use crate::cli::Build;

    fn compile(content: &str) -> (TempDir, PathBuf) {
        // .lls files dir
        let out = tempdir().unwrap();
        let mut src = NamedTempFile::new().unwrap();

        write!(src, "{}", content).unwrap();

        let bin = out.path().join("result.out");
        let build = Build {
            source: Some(src.path().to_str().unwrap().into()),
            compile_path: out.path().to_str().unwrap().into(),
            output: bin.to_str().unwrap().into(),
        };

        build.execute().unwrap();

        // Returning out allows to avoid the drop and removal of the tempdir
        (out, bin)
    }

    #[test]
    fn test_full_exit_program() {
        // _dir is used instead of _ to avoid the drop
        let (_dir, bin) = compile("exit()");
        let res = Command::new(bin).status().unwrap();
        assert_eq!(res.code().unwrap(), 42);
    }

    #[test]
    fn test_full_deprecated_program() {
        // _dir is used instead of _ to avoid the drop
        let (_dir, bin) = compile("skr_app");
        let res = Command::new(bin).status().unwrap();
        assert_eq!(res.code().unwrap(), 0);
    }
}
