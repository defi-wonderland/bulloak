//! Defines the `bulloak check` command.
//!
//! This command performs checks on the relationship between a bulloak tree and
//! a Solidity file.

use std::{fs, path::PathBuf};

use bulloak_syntax::utils::pluralize;
use clap::Parser;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::{backend::BackendKind, cli::Cli, glob::expand_glob};

/// Check that the tests match the spec.
#[doc(hidden)]
#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct Check {
    /// The set of tree files to use as spec.
    ///
    /// Solidity file names are inferred from the specs.
    pub files: Vec<PathBuf>,
    /// Whether to fix any issues found.
    #[arg(long, group = "fix-violations", default_value_t = false)]
    pub fix: bool,
    /// When `--fix` is passed, use `--stdout` to direct output
    /// to standard output instead of writing to files.
    #[arg(long, requires = "fix-violations", default_value_t = false)]
    pub stdout: bool,
    /// Whether to emit modifiers.
    #[arg(short = 'm', long, default_value_t = false)]
    pub skip_modifiers: bool,
    /// Whether to capitalize and punctuate branch descriptions.
    #[arg(long = "format-descriptions", default_value_t = false)]
    pub format_descriptions: bool,
    /// The target language for checking.
    #[arg(short = 'l', long = "lang", value_enum, default_value_t = BackendKind::Solidity)]
    pub backend_kind: BackendKind,
}

impl Default for Check {
    fn default() -> Self {
        Check::parse_from(Vec::<String>::new())
    }
}

impl Check {
    /// Entrypoint for `bulloak check`.
    ///
    /// Note that we don't deal with `solang_parser` errors at all.
    pub(crate) fn run(&self, cfg: &Cli) {
        let mut specs = Vec::new();
        for pattern in &self.files {
            match expand_glob(pattern.clone()) {
                Ok(iter) => specs.extend(iter),
                Err(e) => eprintln!(
                    "{}: could not expand {}: {}",
                    "warn".yellow(),
                    pattern.display(),
                    e
                ),
            }
        }

        let backend = self.backend_kind.get(cfg);

        let mut non_fixed_violations = Vec::new();
        let mut total_fixed = 0;

        for tree_path in specs {
            let check_result = backend.check(&tree_path);

            let (fixed, violations) = match check_result {
                Ok((f, v)) => (f, v),
                Err(e) => {
                    eprintln!(
                        "{}: check failed for {}: {}",
                        "warn".yellow(),
                        tree_path.display(),
                        e
                    );
                    continue;
                }
            };

            // if backend didn't choose to fix, then this'll be None
            if let Some((fixed_count, fixed_text)) = fixed {
                self.write(
                    &fixed_text,
                    backend
                        .test_filename(&tree_path)
                        .expect("shouldn't have been able to fix the testfile"),
                );
                total_fixed += fixed_count;
            }

            for violation in &violations {
                eprintln!("{}", violation);
            }

            non_fixed_violations.extend(violations);
        }

        if total_fixed > 0 {
            let issue_literal =
                if total_fixed == 1 { "issue" } else { "issues" };
            println!(
                "\n{}: {} {} fixed.",
                "success".bold().green(),
                total_fixed,
                issue_literal
            );
        } else if non_fixed_violations.is_empty() {
            println!(
                "{}",
                "All checks completed successfully! No issues found.".green()
            );
        } else {
            let check_literal =
                pluralize(non_fixed_violations.len(), "check", "checks");
            eprint!(
                "{}: {} {} failed",
                "warn".bold().yellow(),
                non_fixed_violations.len(),
                check_literal
            );
            let fixable_count =
                non_fixed_violations.iter().filter(|v| v.is_fixable).count();
            if fixable_count > 0 {
                let fix_literal = pluralize(fixable_count, "fix", "fixes");
                eprintln!(
                " (run `bulloak check --fix <.tree files>` to apply {fixable_count} {fix_literal})"
            );
            } else {
                eprintln!();
            }

            std::process::exit(1);
        }
    }

    /// Handles writing the output of the `check` command.
    ///
    /// If the `--stdout` flag was passed, then the output is printed to
    /// stdout, else it is written to the corresponding file.
    fn write(&self, output: &str, sol: PathBuf) {
        if self.stdout {
            println!("{} {}", "-->".blue(), sol.to_string_lossy());
            println!("{}", output.trim());
            println!("{}", "<--".blue());
        } else if let Err(e) = fs::write(sol, output) {
            eprintln!("{}: {e}", "warn".yellow());
        }
    }
}
