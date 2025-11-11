//! A `bulloak` backend for Rust tests.
//!
//! `bulloak-rust` provides an implementation of turning a `bulloak-syntax`
//! AST into a `_test.rs` file containing scaffolded Rust tests based on the
//! Branching Tree Technique.
//!
//! It also includes validation functionality to check that Rust test files
//! correspond to their `.tree` specifications.

pub mod check;
pub mod config;
pub mod constants;
pub mod rust;
pub mod scaffold;
mod utils;

pub use check::{Violation, ViolationKind};
pub use config::Config;
pub use scaffold::scaffold;
