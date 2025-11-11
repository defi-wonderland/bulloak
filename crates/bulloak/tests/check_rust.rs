#![allow(missing_docs)]
use std::{env, fs};

use common::{cmd, get_binary_path};

mod common;

#[cfg(not(target_os = "windows"))]
#[test]
fn check_rust_passes_when_correct() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("check_rust");
    let tree_path = tests_path.join("basic.tree");

    let output = cmd(&binary_path, "check", &tree_path, &["--lang", "rust"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("All checks completed successfully"));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn check_rust_fails_when_missing_file() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("check_rust");

    let temp_tree = tests_path.join("temp_missing.tree");
    fs::write(&temp_tree, "test_func\n└── It should work.").unwrap();

    let output = cmd(&binary_path, "check", &temp_tree, &["--lang", "rust"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Rust test file is missing"));

    fs::remove_file(temp_tree).ok();
}

#[cfg(not(target_os = "windows"))]
#[test]
fn check_rust_fails_when_missing_test_function() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("check_rust");
    let tree_path = tests_path.join("missing_test.tree");

    let output = cmd(&binary_path, "check", &tree_path, &["--lang", "rust"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Test function") && stderr.contains("is missing"));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn check_rust_fails_when_missing_helper() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("check_rust");
    let tree_path = tests_path.join("missing_helper.tree");

    let output = cmd(&binary_path, "check", &tree_path, &["--lang", "rust"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Helper function") && stderr.contains("is missing")
    );
}

#[cfg(not(target_os = "windows"))]
#[test]
fn check_rust_passes_with_skip_helpers() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("check_rust");
    let tree_path = tests_path.join("no_helpers.tree");

    let output =
        cmd(&binary_path, "check", &tree_path, &["--lang", "rust", "-m"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("All checks completed successfully"));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn check_rust_fails_when_unexpected_should_panic() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let tests_path = cwd.join("tests").join("check_rust");
    let tree_path = tests_path.join("unexpected_should_panic.tree");

    let output = cmd(&binary_path, "check", &tree_path, &["--lang", "rust"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("expected none, found #[should_panic]")
            || stderr.contains("has incorrect attributes")
    );
}
