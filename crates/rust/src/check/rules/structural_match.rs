//! Structural matching rule that checks if Rust code matches the spec.

use std::collections::HashSet;

use anyhow::Result;
use bulloak_syntax::Ast;

use crate::{
    check::violation::{Violation, ViolationKind},
    config::Config,
    rust::ParsedRustFile,
    utils::to_snake_case,
};

/// Expected test structure extracted from AST.
struct ExpectedTests {
    helpers: HashSet<String>,
    test_functions: Vec<TestInfo>,
}

struct TestInfo {
    name: String,
    should_panic: bool,
}

/// Check that the Rust file structurally matches the spec.
///
/// # Errors
///
/// Returns an error if checking fails.
pub fn check_structural_match(
    ast: &Ast,
    rust_source: &str,
    file_path: &str,
    cfg: &Config,
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    // Parse the Rust file
    let parsed = match ParsedRustFile::parse(rust_source) {
        Ok(p) => p,
        Err(e) => {
            violations.push(Violation::new(
                ViolationKind::RustFileInvalid(e.to_string()),
                file_path.to_string(),
            ));
            return Ok(violations);
        }
    };

    // Check test module exists
    if parsed.find_test_module().is_none() {
        violations.push(Violation::new(
            ViolationKind::TestModuleMissing,
            file_path.to_string(),
        ));
        return Ok(violations);
    }

    // Extract expected structure from AST
    let expected = extract_expected_structure(ast, cfg)?;

    // Check helpers (if not skipped)
    if !cfg.skip_helpers {
        let found_helpers: HashSet<String> = parsed
            .find_helper_functions()
            .iter()
            .map(|f| f.sig.ident.to_string())
            .collect();

        for expected_helper in &expected.helpers {
            if !found_helpers.contains(expected_helper) {
                violations.push(Violation::new(
                    ViolationKind::HelperFunctionMissing(
                        expected_helper.clone(),
                    ),
                    file_path.to_string(),
                ));
            }
        }
    }

    // Check test functions
    let found_tests = parsed.find_test_functions();
    let found_test_names: HashSet<String> =
        found_tests.iter().map(|f| f.sig.ident.to_string()).collect();

    for expected_test in &expected.test_functions {
        if !found_test_names.contains(&expected_test.name) {
            violations.push(Violation::new(
                ViolationKind::TestFunctionMissing(expected_test.name.clone()),
                file_path.to_string(),
            ));
        } else {
            // Check attributes
            let found_fn = found_tests
                .iter()
                .find(|f| f.sig.ident.to_string() == expected_test.name)
                .unwrap();

            let has_should_panic = ParsedRustFile::has_should_panic(found_fn);

            if expected_test.should_panic && !has_should_panic {
                violations.push(Violation::new(
                    ViolationKind::TestAttributeIncorrect {
                        function: expected_test.name.clone(),
                        expected: "#[should_panic]".to_string(),
                        found: "none".to_string(),
                    },
                    file_path.to_string(),
                ));
            } else if !expected_test.should_panic && has_should_panic {
                violations.push(Violation::new(
                    ViolationKind::TestAttributeIncorrect {
                        function: expected_test.name.clone(),
                        expected: "none".to_string(),
                        found: "#[should_panic]".to_string(),
                    },
                    file_path.to_string(),
                ));
            }
        }
    }

    Ok(violations)
}

/// Extract expected test structure from AST.
fn extract_expected_structure(
    ast: &Ast,
    cfg: &Config,
) -> Result<ExpectedTests> {
    let ast_root = match ast {
        Ast::Root(r) => r,
        _ => anyhow::bail!("Expected Root node"),
    };

    let mut helpers = HashSet::new();
    let mut test_functions = Vec::new();

    // Collect helpers
    if !cfg.skip_helpers {
        collect_helpers_recursive(&ast_root.children, &mut helpers);
    }

    // Collect test functions
    collect_tests_recursive(&ast_root.children, &[], &mut test_functions);

    Ok(ExpectedTests { helpers, test_functions })
}

/// Recursively collect helper function names.
fn collect_helpers_recursive(children: &[Ast], helpers: &mut HashSet<String>) {
    for child in children {
        if let Ast::Condition(condition) = child {
            let name = to_snake_case(&condition.title);
            helpers.insert(name);
            collect_helpers_recursive(&condition.children, helpers);
        }
    }
}

/// Recursively collect test function info.
fn collect_tests_recursive(
    children: &[Ast],
    parent_helpers: &[String],
    tests: &mut Vec<TestInfo>,
) {
    for child in children {
        match child {
            Ast::Condition(condition) => {
                let helper_name = to_snake_case(&condition.title);
                let mut new_helpers = parent_helpers.to_vec();
                new_helpers.push(helper_name);

                // Collect all direct action children of this condition
                let actions: Vec<&bulloak_syntax::Action> = condition
                    .children
                    .iter()
                    .filter_map(|c| {
                        if let Ast::Action(a) = c {
                            Some(a)
                        } else {
                            None
                        }
                    })
                    .collect();

                if !actions.is_empty() {
                    // Generate a single test for all actions under this
                    // condition
                    let test_name = if new_helpers.is_empty() {
                        let action_part = to_snake_case(&actions[0].title);
                        format!("test_{}", action_part)
                    } else {
                        let last_helper = &new_helpers[new_helpers.len() - 1];
                        format!("test_when_{}", last_helper)
                    };

                    // Check if any action should panic
                    let should_panic = actions.iter().any(|action| {
                        action.title.to_lowercase().split_whitespace().any(
                            |w| {
                                matches!(
                                    w,
                                    "panic"
                                        | "panics"
                                        | "revert"
                                        | "reverts"
                                        | "error"
                                        | "errors"
                                        | "fail"
                                        | "fails"
                                )
                            },
                        )
                    });

                    tests.push(TestInfo { name: test_name, should_panic });
                }

                // Process nested conditions
                collect_tests_recursive(
                    &condition.children,
                    &new_helpers,
                    tests,
                );
            }
            Ast::Action(action) => {
                // Root-level action (no condition)
                if parent_helpers.is_empty() {
                    let action_part = to_snake_case(&action.title);
                    let test_name = format!("test_{}", action_part);

                    let should_panic =
                        action.title.to_lowercase().split_whitespace().any(
                            |w| {
                                matches!(
                                    w,
                                    "panic"
                                        | "panics"
                                        | "revert"
                                        | "reverts"
                                        | "error"
                                        | "errors"
                                        | "fail"
                                        | "fails"
                                )
                            },
                        );

                    tests.push(TestInfo { name: test_name, should_panic });
                }
            }
            _ => {}
        }
    }
}
