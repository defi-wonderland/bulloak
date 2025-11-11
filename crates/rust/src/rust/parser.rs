//! Rust code parser using syn.

use anyhow::{Context, Result};
use syn::{File, Item, ItemFn, ItemMod, ItemStruct};

/// Parsed Rust test file.
pub struct ParsedRustFile {
    /// The parsed syntax tree.
    pub syntax: File,
}

impl ParsedRustFile {
    /// Parse a Rust file from source code.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn parse(source: &str) -> Result<Self> {
        let syntax =
            syn::parse_file(source).context("Failed to parse Rust file")?;
        Ok(Self { syntax })
    }

    /// Find the test module in the file.
    #[must_use]
    pub fn find_test_module(&self) -> Option<&ItemMod> {
        for item in &self.syntax.items {
            if let Item::Mod(module) = item {
                // Check if it has #[cfg(test)] attribute
                if Self::has_cfg_test(&module.attrs) {
                    return Some(module);
                }
            }
        }
        None
    }

    /// Find all test functions in the file.
    #[must_use]
    pub fn find_test_functions(&self) -> Vec<&ItemFn> {
        let mut functions = Vec::new();

        // Check in test module
        if let Some(test_module) = self.find_test_module() {
            if let Some((_, items)) = &test_module.content {
                for item in items {
                    if let Item::Fn(func) = item {
                        if Self::has_test_attr(&func.attrs) {
                            functions.push(func);
                        }
                    }
                }
            }
        }

        functions
    }

    /// Find all helper functions (non-test functions at module level).
    #[must_use]
    pub fn find_helper_functions(&self) -> Vec<&ItemFn> {
        let mut functions = Vec::new();

        for item in &self.syntax.items {
            if let Item::Fn(func) = item {
                // Not a test function
                if !Self::has_test_attr(&func.attrs) {
                    functions.push(func);
                }
            }
        }

        functions
    }

    /// Find the context struct.
    #[must_use]
    pub fn find_context_struct(&self) -> Option<&ItemStruct> {
        for item in &self.syntax.items {
            if let Item::Struct(s) = item {
                // Look for a struct with "Context" in the name
                if s.ident.to_string().contains("Context") {
                    return Some(s);
                }
            }
        }
        None
    }

    /// Check if a function has #[test] attribute.
    fn has_test_attr(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attr| attr.path().is_ident("test"))
    }

    /// Check if an item has #[cfg(test)] attribute.
    fn has_cfg_test(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attr| {
            if attr.path().is_ident("cfg") {
                if let Ok(meta_list) = attr.meta.require_list() {
                    return meta_list.tokens.to_string().contains("test");
                }
            }
            false
        })
    }

    /// Check if a function has #[should_panic] attribute.
    #[must_use]
    pub fn has_should_panic(func: &ItemFn) -> bool {
        func.attrs.iter().any(|attr| attr.path().is_ident("should_panic"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_file() {
        let source = r#"
            #[cfg(test)]
            mod tests {
                #[test]
                fn test_something() {}
            }
        "#;

        let parsed = ParsedRustFile::parse(source).unwrap();
        assert!(parsed.find_test_module().is_some());

        let test_fns = parsed.find_test_functions();
        assert_eq!(test_fns.len(), 1);
        assert_eq!(test_fns[0].sig.ident.to_string(), "test_something");
    }

    #[test]
    fn test_find_helper_functions() {
        let source = r#"
            fn helper_function() {}

            #[cfg(test)]
            mod tests {
                use super::*;

                #[test]
                fn test_something() {}
            }
        "#;

        let parsed = ParsedRustFile::parse(source).unwrap();
        let helpers = parsed.find_helper_functions();
        assert_eq!(helpers.len(), 1);
        assert_eq!(helpers[0].sig.ident.to_string(), "helper_function");
    }

    #[test]
    fn test_has_should_panic() {
        let source = r#"
            #[cfg(test)]
            mod tests {
                #[test]
                #[should_panic]
                fn test_panics() {}

                #[test]
                fn test_normal() {}
            }
        "#;

        let parsed = ParsedRustFile::parse(source).unwrap();
        let test_fns = parsed.find_test_functions();

        assert_eq!(test_fns.len(), 2);
        assert!(ParsedRustFile::has_should_panic(test_fns[0]));
        assert!(!ParsedRustFile::has_should_panic(test_fns[1]));
    }
}
