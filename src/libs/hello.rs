/// 挨拶生成時のエラー種別
#[derive(Debug, PartialEq, Eq)]
pub enum GreetingError {
    /// 性別が未指定
    UnknownGender,
    /// 無効な性別が指定された
    InvalidGender(String),
}

impl std::fmt::Display for GreetingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownGender => write!(f, "gender not specified"),
            Self::InvalidGender(gender) => write!(f, "invalid gender: {gender}"),
        }
    }
}

impl std::error::Error for GreetingError {}

/// 性別を考慮した挨拶生成の結果型
///
/// - `Ok(Ok(T))` - 成功
/// - `Ok(Err(GreetingError))` - 回復可能なエラー
/// - `Err(anyhow::Error)` - 回復不可能なエラー
pub type GreetingResult<T> =
    std::result::Result<std::result::Result<T, GreetingError>, anyhow::Error>;

/// 性別を考慮した挨拶メッセージを生成
///
/// # Arguments
/// * `name` - 挨拶対象の名前
/// * `gender` - 性別（None, Some("man"), Some("woman"), その他）
///
/// # Returns
/// * `Ok(Ok(String))` - 正常な挨拶文字列
/// * `Ok(Err(GreetingError::UnknownGender))` - 性別未指定（回復可能）
/// * `Ok(Err(GreetingError::InvalidGender))` - 無効な性別（回復可能）
/// * `Err(anyhow::Error)` - システムエラー（回復不可能）
///
/// # Errors
/// この関数はネストした Result パターンを使用します：
/// - 外側の `Result` は `anyhow::Error` でシステムレベルのエラー
/// - 内側の `Result` は `GreetingError` で回復可能なビジネスロジックエラー
///
/// # Examples
/// ```
/// use hello::sayhello;
///
/// // 成功例
/// let result = sayhello("Alice", Some("woman"));
/// assert!(matches!(result, Ok(Ok(_))));
///
/// // 回復可能エラー例
/// let result = sayhello("Bob", None);
/// assert!(matches!(result, Ok(Err(_))));
/// ```
pub fn sayhello(name: &str, gender: Option<&str>) -> GreetingResult<String> {
    use anyhow::Context;

    let result = match gender {
        Some("man") => Ok(format!("Hi, Mr. {name}")),
        Some("woman") => Ok(format!("Hi, Ms. {name}")),
        None => Ok(format!("Hi, {name}")),
        Some(invalid) => Err(GreetingError::InvalidGender(String::from(invalid))),
    };

    Ok::<std::result::Result<String, GreetingError>, anyhow::Error>(result)
        .context("Failed to generate greeting with gender")
}

#[cfg(test)]
mod tests {
    use super::{GreetingError, sayhello};

    #[test]
    fn test_sayhello_with_gender_man() {
        let result = sayhello("John", Some("man"));
        assert!(matches!(result, Ok(Ok(_))));
        if let Ok(Ok(greeting)) = result {
            assert_eq!(greeting, "Hi, Mr. John");
        }
    }

    #[test]
    fn test_sayhello_with_gender_woman() {
        let result = sayhello("Alice", Some("woman"));
        assert!(matches!(result, Ok(Ok(_))));
        if let Ok(Ok(greeting)) = result {
            assert_eq!(greeting, "Hi, Ms. Alice");
        }
    }

    #[test]
    fn test_sayhello_with_gender_none() {
        let result = sayhello("Bob", None);
        assert!(matches!(result, Ok(Ok(_))));
        if let Ok(Ok(greeting)) = result {
            assert_eq!(greeting, "Hi, Bob");
        }
    }

    #[test]
    fn test_sayhello_with_gender_invalid() {
        let result = sayhello("Charlie", Some("other"));
        assert!(matches!(result, Ok(Err(GreetingError::InvalidGender(_)))));
        if let Ok(Err(GreetingError::InvalidGender(gender))) = result {
            assert_eq!(gender, "other");
        }
    }

    #[test]
    fn test_sayhello_with_gender_empty_string() {
        let result = sayhello("Dave", Some(""));
        assert!(matches!(result, Ok(Err(GreetingError::InvalidGender(_)))));
        if let Ok(Err(GreetingError::InvalidGender(gender))) = result {
            assert_eq!(gender, "");
        }
    }

    #[test]
    fn test_greeting_error_display() {
        assert_eq!(
            GreetingError::UnknownGender.to_string(),
            "gender not specified"
        );
        assert_eq!(
            GreetingError::InvalidGender(String::from("test")).to_string(),
            "invalid gender: test"
        );
    }
}
