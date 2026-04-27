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
mod naming_tests {
    use super::*;

    #[test]
    fn test_snake_to_pascal() {
        assert_eq!(snake_to_pascal("hello"), "Hello");
        assert_eq!(snake_to_pascal("hello_world"), "HelloWorld");
        assert_eq!(snake_to_pascal("my_art"), "MyArt");
        assert_eq!(snake_to_pascal("a"), "A");
        assert_eq!(snake_to_pascal(""), "");
    }
}
