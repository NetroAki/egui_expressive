pub const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "union", "unsized", "virtual", "yield", "try",
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
}
