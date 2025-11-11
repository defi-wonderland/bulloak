//! `bulloak`'s CLI config.
use clap::{Parser, Subcommand, ValueEnum};
use figment::{providers::Serialized, Figment};
use serde::{Deserialize, Serialize};

/// The target backend/language for code generation.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    ValueEnum,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    /// Solidity (Foundry) backend.
    #[default]
    Solidity,
    /// Rust backend.
    Rust,
    /// Noir backend.
    Noir,
}

/// `bulloak`'s configuration.
#[derive(Parser, Debug, Clone, Default, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
pub struct Cli {
    /// `bulloak`'s commands.
    #[clap(subcommand)]
    pub command: Commands,
}

/// `bulloak`'s commands.
#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum Commands {
    /// `bulloak scaffold`.
    #[command(name = "scaffold")]
    Scaffold(crate::scaffold::Scaffold),
    /// `bulloak check`.
    #[command(name = "check")]
    Check(crate::check::Check),
}

impl Default for Commands {
    fn default() -> Self {
        Self::Scaffold(Default::default())
    }
}

impl From<&Cli> for bulloak_foundry::config::Config {
    fn from(cli: &Cli) -> Self {
        match &cli.command {
            Commands::Scaffold(cmd) => Self {
                files: cmd.files.clone(),
                solidity_version: cmd.solidity_version.clone(),
                emit_vm_skip: cmd.with_vm_skip,
                skip_modifiers: cmd.skip_modifiers,
                format_descriptions: cmd.format_descriptions,
                ..Self::default()
            },
            Commands::Check(cmd) => Self {
                files: cmd.files.clone(),
                skip_modifiers: cmd.skip_modifiers,
                format_descriptions: cmd.format_descriptions,
                ..Self::default()
            },
        }
    }
}

impl From<&Cli> for bulloak_noir::Config {
    fn from(cli: &Cli) -> Self {
        match &cli.command {
            Commands::Scaffold(cmd) => Self {
                files: cmd
                    .files
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect(),
                skip_helpers: cmd.skip_modifiers,
                format_descriptions: cmd.format_descriptions,
            },
            Commands::Check(cmd) => Self {
                files: cmd
                    .files
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect(),
                skip_helpers: cmd.skip_modifiers,
                format_descriptions: cmd.format_descriptions,
            },
        }
    }
}

/// Main entrypoint of `bulloak`'s execution.
pub(crate) fn run() -> anyhow::Result<()> {
    let config: Cli =
        Figment::new().merge(Serialized::defaults(Cli::parse())).extract()?;

    match &config.command {
        Commands::Scaffold(command) => command.run(&config),
        Commands::Check(command) => command.run(&config),
    };

    Ok(())
}
