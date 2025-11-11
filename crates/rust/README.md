# bulloak-rust

A backend for `bulloak` that generates Rust test files.

This crate provides an implementation of turning a `bulloak-syntax` AST into a `_test.rs` file containing scaffolded Rust tests based on the Branching Tree Technique.

It also includes validation functionality to check that Rust test files correspond to their `.tree` specifications.
