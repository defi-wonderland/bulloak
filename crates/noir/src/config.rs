//! Configuration for Noir backend.

/// Configuration for Noir test generation and checking.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// List of files being processed.
    pub files: Vec<String>,
    /// Skip generation of helper functions for conditions.
    pub skip_helpers: bool,
    /// Format action descriptions (capitalize, etc).
    pub format_descriptions: bool,
}
