/// 指定された名前で挨拶メッセージを生成
///
/// # Arguments
/// * `name` - 挨拶対象の名前
///
/// # Returns
/// "Hi, {name}"形式の挨拶文字列
///
/// # Examples
/// ```
/// let greeting = sayhello("Alice");
/// assert_eq!(greeting, "Hi, Alice");
/// ```
#[must_use]
pub fn sayhello(name: &str) -> String {
    format!("Hi, {name}")
}

#[cfg(test)]
mod tests {
    use super::sayhello;

    #[test]
    fn test_sayhello() {
        assert_eq!(sayhello("Alice"), "Hi, Alice");
        assert_eq!(sayhello("Bob"), "Hi, Bob");
        assert_eq!(sayhello("世界"), "Hi, 世界");
        assert_eq!(sayhello(""), "Hi, ");
    }
}
