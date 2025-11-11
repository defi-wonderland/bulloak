//! Comment formatting utilities.

/// Format a comment by capitalizing the first letter and ensuring it ends with
/// a period.
pub(crate) fn format_comment(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut chars = trimmed.chars();
    let first = chars.next().unwrap();
    let rest: String = chars.collect();

    let capitalized = format!("{}{}", first.to_uppercase(), rest);

    if capitalized.ends_with('.')
        || capitalized.ends_with('!')
        || capitalized.ends_with('?')
    {
        capitalized
    } else {
        format!("{}.", capitalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_comment() {
        assert_eq!(format_comment("should return sum"), "Should return sum.");
        assert_eq!(format_comment("Should return sum."), "Should return sum.");
        assert_eq!(format_comment("should panic!"), "Should panic!");
        assert_eq!(format_comment(""), "");
    }
}
