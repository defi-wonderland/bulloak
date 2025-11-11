//! Utility functions for the Rust backend.

/// Convert string to snake_case.
///
/// Strips common BDD prefixes (when, given, it) and converts to snake_case.
pub(crate) fn to_snake_case(s: &str) -> String {
    let s = s.trim();
    let s = s
        .strip_prefix("when ")
        .or_else(|| s.strip_prefix("When "))
        .or_else(|| s.strip_prefix("WHEN "))
        .or_else(|| s.strip_prefix("given "))
        .or_else(|| s.strip_prefix("Given "))
        .or_else(|| s.strip_prefix("GIVEN "))
        .or_else(|| s.strip_prefix("it "))
        .or_else(|| s.strip_prefix("It "))
        .or_else(|| s.strip_prefix("IT "))
        .unwrap_or(s);

    let mut result = String::new();
    let mut prev_is_alphanumeric = false;

    for c in s.chars() {
        if c.is_alphanumeric() {
            if c.is_uppercase() && prev_is_alphanumeric && !result.is_empty() {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_is_alphanumeric = true;
        } else if c.is_whitespace() || c == '-' {
            if prev_is_alphanumeric {
                result.push('_');
                prev_is_alphanumeric = false;
            }
        } else {
            prev_is_alphanumeric = false;
        }
    }

    result.trim_end_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(
            to_snake_case("when first arg is smaller"),
            "first_arg_is_smaller"
        );
        assert_eq!(
            to_snake_case("It should return the sum"),
            "should_return_the_sum"
        );
        assert_eq!(to_snake_case("given a valid input"), "a_valid_input");
    }
}
