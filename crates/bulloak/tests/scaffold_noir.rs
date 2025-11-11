#![allow(missing_docs)]
use std::{env, fs};

use common::{cmd, get_binary_path};
use pretty_assertions::assert_eq;

mod common;

#[cfg(not(target_os = "windows"))]
#[test]
fn scaffolds_noir_trees() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let trees_path = cwd.join("tests").join("scaffold");
    let outputs_path = cwd.join("tests").join("scaffold_noir");
    let trees = [
        "basic.tree",
        "complex.tree",
        "disambiguation.tree",
        "duplicated_condition.tree",
        "duplicated_top_action.tree",
        "empty.tree",
        "format_descriptions.tree",
        "hash_pair.tree",
        "removes_invalid_title_chars.tree",
        "revert_when.tree",
        "skip_modifiers.tree",
        "spurious_comments.tree",
    ];

    for tree_name in trees {
        let tree_path = trees_path.join(tree_name);
        let output =
            cmd(&binary_path, "scaffold", &tree_path, &["--lang", "noir"]);
        let actual = String::from_utf8(output.stdout).unwrap();

        let output_file =
            outputs_path.join(tree_name.replace(".tree", "_test.nr"));

        let expected = fs::read_to_string(&output_file).unwrap_or_else(|_| {
            panic!(
                "Failed to read expected output file: {}",
                output_file.display()
            )
        });

        // We trim here because we don't care about ending newlines.
        assert_eq!(
            expected.trim(),
            actual.trim(),
            "Mismatch for {}",
            tree_name
        );
    }
}

#[cfg(not(target_os = "windows"))]
#[test]
fn scaffolds_noir_trees_skip_helpers() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let trees_path = cwd.join("tests").join("scaffold");
    let tree_path = trees_path.join("basic.tree");

    let output =
        cmd(&binary_path, "scaffold", &tree_path, &["--lang", "noir", "-m"]);
    let actual = String::from_utf8(output.stdout).unwrap();

    // Should not contain helper functions
    assert!(!actual.contains("fn first_arg_is_smaller_than_second_arg"));
    assert!(!actual.contains("fn first_arg_is_bigger_than_second_arg"));

    // Should still contain test functions
    assert!(actual.contains("#[test]"));
    assert!(actual.contains("unconstrained fn"));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn scaffolds_noir_trees_format_descriptions() {
    let cwd = env::current_dir().unwrap();
    let binary_path = get_binary_path();
    let trees_path = cwd.join("tests").join("scaffold");
    let tree_path = trees_path.join("basic.tree");

    let output = cmd(
        &binary_path,
        "scaffold",
        &tree_path,
        &["--lang", "noir", "--format-descriptions"],
    );
    let actual = String::from_utf8(output.stdout).unwrap();

    // Comments should be capitalized and have periods
    assert!(actual.contains(
        "// It should match the result of `keccak256(abi.encodePacked(a,b))`."
    ));
    assert!(actual.contains(
        "// It should match the result of `keccak256(abi.encodePacked(b,a))`."
    ));
}
