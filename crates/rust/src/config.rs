//! Configuration for the Rust backend.

/// Configuration for the Rust backend.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// List of files to process.
    pub files: Vec<String>,
    /// Whether to skip emitting helper functions.
    pub skip_helpers: bool,
    /// Whether to format/capitalize branch descriptions.
    pub format_descriptions: bool,
}

impl Config {
    /// Create a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
