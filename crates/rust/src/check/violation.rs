//! Violation types for check command.

use std::fmt;

/// A violation found during checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The kind of violation.
    pub kind: ViolationKind,
    /// The file path where the violation occurred.
    pub file_path: String,
    /// Optional line number.
    pub line: Option<usize>,
}

impl Violation {
    /// Create a new violation.
    #[must_use]
    pub fn new(kind: ViolationKind, file_path: String) -> Self {
        Self { kind, file_path, line: None }
    }

    /// Create a new violation with a line number.
    #[must_use]
    pub fn with_line(
        kind: ViolationKind,
        file_path: String,
        line: usize,
    ) -> Self {
        Self { kind, file_path, line: Some(line) }
    }
}

/// The kind of violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationKind {
    /// The Rust file is missing.
    RustFileMissing,
    /// The Rust file could not be parsed.
    RustFileInvalid(String),
    /// The test module is missing.
    TestModuleMissing,
    /// A test function is missing.
    TestFunctionMissing(String),
    /// A helper function is missing.
    HelperFunctionMissing(String),
    /// A test function has incorrect attributes.
    TestAttributeIncorrect {
        /// The function name.
        function: String,
        /// The expected attribute.
        expected: String,
        /// The found attribute.
        found: String,
    },
    /// Test function order does not match spec.
    TestOrderIncorrect,
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RustFileMissing => write!(f, "Rust test file is missing"),
            Self::RustFileInvalid(err) => write!(f, "Rust file could not be parsed: {}", err),
            Self::TestModuleMissing => write!(f, "Test module (#[cfg(test)] mod tests) is missing"),
            Self::TestFunctionMissing(name) => write!(f, "Test function '{}' is missing", name),
            Self::HelperFunctionMissing(name) => write!(f, "Helper function '{}' is missing", name),
            Self::TestAttributeIncorrect {
                function,
                expected,
                found,
            } => write!(
                f,
                "Test function '{}' has incorrect attributes: expected {}, found {}",
                function, expected, found
            ),
            Self::TestOrderIncorrect => {
                write!(f, "Test function order does not match spec order")
            }
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(f, "{}:{}: {}", self.file_path, line, self.kind)
        } else {
            write!(f, "{}: {}", self.file_path, self.kind)
        }
    }
}
