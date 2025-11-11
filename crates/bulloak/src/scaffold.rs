//! Defines the `bulloak scaffold` command.
//!
//! This command scaffolds a Solidity file from a spec `.tree` file.

use std::{
    fs,
    path::{Path, PathBuf},
};

use bulloak_foundry::{constants::DEFAULT_SOL_VERSION, scaffold::scaffold};
use clap::Parser;
use forge_fmt::fmt;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::{
    cli::{Backend, Cli},
    glob::expand_glob,
};

/// Generate test files based on your spec.
#[doc(hidden)]
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub struct Scaffold {
    /// The set of tree files to generate from.
    ///
    /// Each Solidity file will be named after its matching
    /// tree spec.
    pub files: Vec<PathBuf>,
    /// Whether to write to files instead of stdout.
    ///
    /// This will write the output for each input file to the file
    /// specified at the root of the input file if the output file
    /// doesn't already exist. To overwrite, use `--force-write`
    /// together with `--write-files`.
    #[arg(short = 'w', long, group = "file-handling", default_value_t = false)]
    pub write_files: bool,
    /// When `--write-files` is passed, use `--force-write` to
    /// overwrite the output files.
    #[arg(
        short = 'f',
        long,
        requires = "file-handling",
        default_value_t = false
    )]
    pub force_write: bool,
    /// Sets a Solidity version for the test contracts.
    #[arg(short = 's', long, default_value = DEFAULT_SOL_VERSION)]
    pub solidity_version: String,
    /// Whether to add vm.skip(true) at the beginning of each test.
    #[arg(short = 'S', long = "vm-skip", default_value_t = false)]
    pub with_vm_skip: bool,
    /// Whether to emit modifiers.
    #[arg(short = 'm', long, default_value_t = false)]
    pub skip_modifiers: bool,
    /// Whether to capitalize and punctuate branch descriptions.
    #[arg(short = 'F', long = "format-descriptions", default_value_t = false)]
    pub format_descriptions: bool,
    /// The target language for code generation.
    #[arg(short = 'l', long = "lang", value_enum, default_value_t = Backend::Solidity)]
    pub backend: Backend,
}

impl Default for Scaffold {
    fn default() -> Self {
        Scaffold::parse_from(Vec::<String>::new())
    }
}

impl Scaffold {
    /// Runs the scaffold command, processing all specified files.
    ///
    /// This method iterates through all input files, processes them, and either
    /// writes the output to files or prints to stdout based on the config.
    ///
    /// If any errors occur during processing, they are collected and reported.
    pub(crate) fn run(&self, cfg: &Cli) {
        let mut files = Vec::with_capacity(self.files.len());
        for pattern in &self.files {
            match expand_glob(pattern.clone()) {
                Ok(iter) => files.extend(iter),
                Err(e) => {
                    eprintln!(
                        "{}: could not expand {}: {}",
                        "warn".yellow(),
                        pattern.display(),
                        e
                    );
                }
            }
        }

        let errors = files
            .iter()
            .filter_map(|file| {
                self.process_file(file, cfg)
                    .map_err(|e| (file.as_path(), e))
                    .err()
            })
            .collect::<Vec<_>>();

        if !errors.is_empty() {
            Scaffold::report_errors(&errors);
            std::process::exit(1);
        }
    }

    /// Processes a single input file.
    ///
    /// This method reads the input file, scaffolds the code, formats
    /// it, and either writes it to a file or prints it to stdout.
    fn process_file(&self, file: &Path, cfg: &Cli) -> anyhow::Result<()> {
        let text = fs::read_to_string(file)?;

        let (emitted, output_file) = match self.backend {
            Backend::Rust => {
                let ast = bulloak_syntax::parse_one(&text)?;
                let rust_cfg = bulloak_rust::Config {
                    files: self
                        .files
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect(),
                    skip_helpers: self.skip_modifiers,
                    format_descriptions: self.format_descriptions,
                };
                let emitted = bulloak_rust::scaffold(&ast, &rust_cfg)?;
                let output_file = Self::build_output_path(file, "_test.rs")?;
                (emitted, output_file)
            }
            Backend::Noir => {
                let forest = bulloak_syntax::parse(&text)?;
                let noir_cfg: bulloak_noir::Config = cfg.into();
                let emitted = bulloak_noir::scaffold(&forest, &noir_cfg)?;
                let output_file = Self::build_output_path(file, "_test.nr")?;
                (emitted, output_file)
            }
            Backend::Solidity => {
                let emitted = scaffold(&text, &cfg.into())?;
                let formatted = fmt(&emitted).unwrap_or_else(|err| {
                    eprintln!("{}: {}", "WARN".yellow(), err);
                    emitted
                });
                let output_file = file.with_extension("t.sol");
                (formatted, output_file)
            }
        };

        self.output(&emitted, &output_file);
        Ok(())
    }

    /// Builds the output file path for a given input file.
    fn build_output_path(file: &Path, suffix: &str) -> anyhow::Result<PathBuf> {
        let file_stem =
            file.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
                anyhow::anyhow!("Invalid file name: {}", file.display())
            })?;
        Ok(file.with_file_name(format!("{file_stem}{suffix}")))
    }

    /// Outputs the scaffolded text either to stdout or to a file.
    fn output(&self, text: &str, file: &PathBuf) {
        if self.write_files {
            self.write_file(text, file);
        } else {
            println!("{text}");
        }
    }

    /// Writes the provided `text` to `file`.
    ///
    /// If the file doesn't exist it will create it. If it exists,
    /// and `--force-write` was not passed, it will skip writing to the file.
    fn write_file(&self, text: &str, file: &PathBuf) {
        // Don't overwrite files unless `--force-write` was passed.
        if file.exists() && !self.force_write {
            eprintln!(
                "{}: Skipped emitting {:?}",
                "warn".yellow(),
                file.as_path().blue()
            );
            eprintln!(
                "    {} The corresponding `.t.sol` file already exists",
                "=".blue()
            );
            return;
        }

        if let Err(err) = fs::write(file, text) {
            eprintln!("{}: {err}", "error".red());
        };
    }

    /// Reports errors that occurred during file processing.
    ///
    /// This method prints error messages for each file that failed to process,
    /// along with a summary of the total number of failed files.
    fn report_errors(errors: &[(&Path, anyhow::Error)]) {
        for (file, err) in errors {
            eprintln!("{err}");
            eprintln!("file: {}", file.display());
        }

        eprintln!(
            "\n{}: Could not scaffold {} files. Check the output above or run {}, which might prove helpful.",
            "warn".yellow(),
            errors.len().yellow(),
            "bulloak check".blue()
        );
    }
}
