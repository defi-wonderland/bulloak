//! Constants used in the Rust backend.

/// Keywords that indicate a test should panic.
pub(crate) const PANIC_KEYWORDS: &[&str] = &[
    "panic", "panics", "revert", "reverts", "error", "errors", "fail", "fails",
];

/// Name of the test context struct.
pub(crate) const CONTEXT_STRUCT_NAME: &str = "TestContext";
