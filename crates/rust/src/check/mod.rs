//! Check module for validating Rust test files against specs.

pub mod rules;
pub mod violation;

use std::path::Path;

use anyhow::{Context, Result};
pub use violation::{Violation, ViolationKind};

use crate::config::Config;

/// Check that a Rust test file matches its tree specification.
///
/// # Errors
///
/// Returns an error if checking fails.
pub fn check(tree_path: &Path, cfg: &Config) -> Result<Vec<Violation>> {
    // Read tree file
    let tree_source =
        std::fs::read_to_string(tree_path).with_context(|| {
            format!("Failed to read tree file: {}", tree_path.display())
        })?;

    // Parse tree
    let ast = bulloak_syntax::parse_one(&tree_source)?;

    // Determine Rust file path (replace .tree with _test.rs)
    let file_stem = tree_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;
    let rust_path = tree_path.with_file_name(format!("{}_test.rs", file_stem));

    // Check if Rust file exists
    if !rust_path.exists() {
        return Ok(vec![Violation::new(
            ViolationKind::RustFileMissing,
            rust_path.display().to_string(),
        )]);
    }

    // Read Rust file
    let rust_source =
        std::fs::read_to_string(&rust_path).with_context(|| {
            format!("Failed to read Rust file: {}", rust_path.display())
        })?;

    // Run structural match rule
    rules::check_structural_match(
        &ast,
        &rust_source,
        &rust_path.display().to_string(),
        cfg,
    )
}
