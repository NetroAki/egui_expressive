pub const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
    "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
    "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
    "type", "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become",
    "box", "do", "final", "macro", "override", "priv", "typeof", "union", "unsized", "virtual", "yield",
    "try",
];

pub fn is_valid_module_name(s: &str) -> bool {
    if s.is_empty() || s == "_" || s.starts_with(|c: char| c.is_ascii_digit()) {
        return false;
    }
    if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return false;
    }
    !RUST_KEYWORDS.contains(&s)
}

pub fn snake_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_module_name() {
        assert!(is_valid_module_name("valid_name"));
        assert!(is_valid_module_name("name123"));
        assert!(!is_valid_module_name("123name"));
        assert!(!is_valid_module_name("invalid-name"));
        assert!(!is_valid_module_name(""));
        assert!(!is_valid_module_name("_"));
        
        // Test keywords
        assert!(!is_valid_module_name("union"));
        assert!(!is_valid_module_name("fn"));
        assert!(!is_valid_module_name("let"));
        assert!(!is_valid_module_name("async"));
    }

    #[test]
    fn test_snake_to_pascal() {
        assert_eq!(snake_to_pascal("hello"), "Hello");
        assert_eq!(snake_to_pascal("hello_world"), "HelloWorld");
        assert_eq!(snake_to_pascal("my_art"), "MyArt");
        assert_eq!(snake_to_pascal("a"), "A");
        assert_eq!(snake_to_pascal(""), "");
    }
}
