pub fn sayhello(name: String) -> String {
    format!("Hi, {}", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sayhello() {
        assert_eq!(sayhello("Alice".to_string()), "Hi, Alice");
        assert_eq!(sayhello("Bob".to_string()), "Hi, Bob");
        assert_eq!(sayhello("世界".to_string()), "Hi, 世界");
        assert_eq!(sayhello("".to_string()), "Hi, ");
    }
}
