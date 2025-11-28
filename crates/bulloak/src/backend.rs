//! Bulloak backend trait
//!
//! This module defines the core trait that all bulloak backends must implement,
//! along with their concrete implementations
use bulloak_foundry::check::context::Context;
use owo_colors::OwoColorize;
use regex::Regex;
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::cli::{Cli, Commands};

pub struct Violation {
    pub is_fixable: bool,
    message: String,
    tree_file: PathBuf,
    test_file: Option<PathBuf>,
    line: Option<usize>,
}

/// Trait for backends that generate test files from `.tree` specifications.
///
/// Implementors of this trait must transform a tree specification (in string
/// form) into generated test code for a specific testing framework.
pub trait Backend: Send + Sync {
    /// Scaffolds test code from a tree specification.
    /// Must output it already formatted, as it won't be processed further
    fn scaffold(&self, text: &str) -> Result<String>;

    /// Given a treefile, checks the testfile has the correct structure
    /// May fix them, depending on self.config, in which case it'll return
    /// the updated source code.
    /// Returns an array of the Violations that were not fixed
    fn check(
        &self,
        tree_file: &PathBuf,
    ) -> Result<(Option<(usize, String)>, Vec<Violation>)>;

    /// Returns the output test file path for a given tree file path.
    fn test_filename(&self, tree_file: &PathBuf) -> Result<PathBuf>;
}

/// Available backend types for CLI argument parsing.
#[derive(Debug, Serialize, Deserialize, Clone, ValueEnum)]
pub enum BackendKind {
    /// original Foundry backend.
    Solidity,
    Noir,
}

#[derive(Error, Debug)]
enum BackendError {
    #[error("invalid filename: {0}")]
    InvalidFilename(PathBuf),

    #[error("missing .tree extension: {0}")]
    MissingTreeExtension(PathBuf),
}

/// Solidity/Foundry backend with baked-in config.
pub(crate) struct SolidityBackend {
    config: bulloak_foundry::config::Config,
    fix: bool,
}

/// Noir/Aztec backend with baked-in config.
pub(crate) struct NoirBackend {
    config: bulloak_noir::Config,
}

impl BackendKind {
    /// Creates a boxed backend instance with config derived from CLI.
    pub fn get(&self, cli: &Cli) -> Box<dyn Backend> {
        match self {
            Self::Solidity => Box::new(SolidityBackend {
                config: cli.into(),
                fix: if let Commands::Check(c) = &cli.command {
                    c.fix
                } else {
                    false
                },
            }),
            Self::Noir => Box::new(NoirBackend { config: cli.into() }),
        }
    }
}

fn validate_extension(input: &PathBuf) -> Result<(), BackendError> {
    let extension = input
        .extension()
        .ok_or(BackendError::InvalidFilename(input.to_owned()))?;
    if extension != "tree" {
        return Err(BackendError::MissingTreeExtension(input.to_owned()));
    }
    Ok(())
}

impl Backend for SolidityBackend {
    fn scaffold(&self, text: &str) -> Result<String> {
        let emitted = bulloak_foundry::scaffold::scaffold(text, &self.config)?;
        Ok(forge_fmt::fmt(&emitted).unwrap_or(emitted))
    }

    fn check(
        &self,
        tree_file: &PathBuf,
    ) -> Result<(Option<(usize, String)>, Vec<Violation>)> {
        let mut violations = Vec::new();
        let ctx = Context::new(tree_file.clone(), &self.config);
        let _ = ctx.map_err(|violation| violations.push(violation));
        if self.fix {
            todo!();
        }
        Ok((None, violations.iter().map(|x| x.into()).collect()))
    }

    fn test_filename(&self, tree_file: &PathBuf) -> Result<PathBuf> {
        validate_extension(tree_file)?;
        Ok(tree_file.with_extension("t.sol"))
    }
}

impl Backend for NoirBackend {
    fn scaffold(&self, text: &str) -> Result<String> {
        bulloak_noir::scaffold(&text, &self.config)
    }

    fn check(
        &self,
        _tree_file: &PathBuf,
    ) -> Result<(Option<(usize, String)>, Vec<Violation>)> {
        todo!();
    }

    fn test_filename(&self, tree_file: &PathBuf) -> Result<PathBuf> {
        let regex = Regex::new(r"\.tree$").unwrap();
        validate_extension(tree_file)?;
        let input_filename = tree_file.to_str().ok_or(anyhow::anyhow!(
            "invalid filename: {}",
            tree_file.display()
        ))?;
        let output_filename = regex.replace_all(input_filename, "_test.nr");
        if output_filename == input_filename {
            return Err(anyhow::anyhow!(
                "invalid filename, {}",
                tree_file.display()
            ));
        }
        Ok(PathBuf::from(output_filename.into_owned()))
    }
}

impl From<&bulloak_foundry::Violation> for Violation {
    fn from(f: &bulloak_foundry::Violation) -> Violation {
        let mut message = format!("{}", f.kind);
        if let Some(help_text) = f.kind.help() {
            message =
                format!("{}\n     {} help: {}", message, "=".blue(), help_text);
        }
        Violation {
            message,
            tree_file: PathBuf::from(f.location.file()),
            // TODO: populate test/tree file based on which one is referred to
            test_file: None,
            line: if let bulloak_foundry::check::location::Location::Code(
                _,
                line,
            ) = f.location
            {
                Some(line)
            } else {
                None
            },
            is_fixable: f.is_fixable(),
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {}", "warn".yellow(), self.message)?;
        if self.is_fixable {
            let tree_file = self.tree_file.display();
            write!(f, "     {} fix: run ", "+".blue())?;
            writeln!(f, "`bulloak check --fix {tree_file}`")?;
        }
        if let Some(test_file) = self.test_file.clone() {
            if let Some(line) = self.line {
                writeln!(
                    f,
                    "   {} {}:{}",
                    "-->".blue(),
                    test_file.display(),
                    line
                )?;
            } else {
                writeln!(f, "   {} {}", "-->".blue(), test_file.display())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tree_file() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from("MyContract.tree");
        let result = noir_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("MyContract_test.nr"));

        let result = foundry_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("MyContract.t.sol"));
    }

    #[test]
    fn test_with_directory_path() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from("src/contracts/MyContract.tree");
        let result = noir_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("src/contracts/MyContract_test.nr"));

        let result = foundry_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("src/contracts/MyContract.t.sol"));
    }

    #[test]
    fn test_with_multiple_dots() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from("My.Complex.Contract.tree");
        let result = noir_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("My.Complex.Contract_test.nr"));
        let result = foundry_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("My.Complex.Contract.t.sol"));
    }

    #[test]
    fn test_already_has_test_suffix() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from("MyContract_test.tree");
        let result = noir_backend.test_filename(&input).unwrap();
        // Should append another _test
        assert_eq!(result, PathBuf::from("MyContract_test_test.nr"));
        let result = foundry_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("MyContract_test.t.sol"));
    }

    #[test]
    fn test_with_absolute_path() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from("/home/user/project/Contract.tree");
        let result = noir_backend.test_filename(&input).unwrap();
        assert_eq!(
            result,
            PathBuf::from("/home/user/project/Contract_test.nr")
        );
        let result = foundry_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("/home/user/project/Contract.t.sol"));
    }

    #[test]
    fn test_preserves_parent_directories() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from("tests/specs/nested/MyTest.tree");
        let result = noir_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("tests/specs/nested/MyTest_test.nr"));
        let result = foundry_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("tests/specs/nested/MyTest.t.sol"));
    }

    #[test]
    fn test_no_extension() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from("MyContract");
        let result = noir_backend.test_filename(&input);
        assert!(result.is_err());
        let result = foundry_backend.test_filename(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_extension() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from("MyContract.txt");
        let result = noir_backend.test_filename(&input);
        assert!(result.is_err());
        let result = foundry_backend.test_filename(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_fails() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from("");
        let result = noir_backend.test_filename(&input);
        assert!(result.is_err());
        let result = foundry_backend.test_filename(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_directory_only_fails() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from("src/");
        let result = noir_backend.test_filename(&input);
        assert!(result.is_err());
        let result = foundry_backend.test_filename(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_with_unicode() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from("🐻.tree");
        let result = noir_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("🐻_test.nr"));
        let result = foundry_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("🐻.t.sol"));
    }

    #[test]
    fn test_with_spaces() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from("My Contract.tree");
        let result = noir_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("My Contract_test.nr"));
        let result = foundry_backend.test_filename(&input).unwrap();
        assert_eq!(result, PathBuf::from("My Contract.t.sol"));
    }

    #[test]
    fn test_only_extension() {
        let noir_backend =
            NoirBackend { config: bulloak_noir::Config::default() };
        let foundry_backend = SolidityBackend {
            config: bulloak_foundry::config::Config::default(),
            fix: false,
        };

        let input = PathBuf::from(".tree");
        let result = noir_backend.test_filename(&input);
        assert!(result.is_err());
        let result = foundry_backend.test_filename(&input);
        assert!(result.is_err());

        let input = PathBuf::from("/foo/.tree");
        let result = noir_backend.test_filename(&input);
        assert!(result.is_err());
        let result = foundry_backend.test_filename(&input);
        assert!(result.is_err());

        let input = PathBuf::from("src/.tree");
        let result = noir_backend.test_filename(&input);
        assert!(result.is_err());
        let result = foundry_backend.test_filename(&input);
        assert!(result.is_err());
    }
}
