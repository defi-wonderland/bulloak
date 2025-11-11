//! Noir test scaffolding.

mod generator;

use anyhow::Result;
use bulloak_syntax::Ast;

use crate::Config;

/// Generate Noir test code from an AST.
///
/// # Errors
///
/// Returns an error if code generation fails.
pub fn generate(forest: &Vec<Ast>, cfg: &Config) -> Result<String> {
    generator::generate(forest, cfg)
}
