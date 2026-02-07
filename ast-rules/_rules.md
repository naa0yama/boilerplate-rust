# ast-grep ルール解説

プロジェクトで使用しているast-grepルールの詳細解説です。各ルールについてダメな例、良い例、理由を示します。

> **Note**: 以下のルールは clippy に移行済みのため、ast-grep からは削除されました:
> - `unsafe-needs-safety-comment` -> `clippy::undocumented_unsafe_blocks`
> - `no-unwrap-in-production` -> `clippy::unwrap_used` + `.clippy.toml` の `allow-unwrap-in-tests`
> - `no-println-debug` -> `clippy::print_stdout` + `print_stderr` + `dbg_macro`
> - `no-wildcard-import` -> `clippy::wildcard_imports`
> - `no-ignored-result` -> rustc `unused_must_use`
> - `require-pub-doc-comment` -> rustc `missing_docs`
> - `avoid-nested-matches` -> `clippy::collapsible_match`

---

## パフォーマンス最適化

### prefer-vec-with-capacity

**目的**: ループ内 `push` の際は `Vec::with_capacity()` 使用を推奨

#### ダメな例

```rust
fn process_items(items: &[Item]) -> Vec<ProcessedItem> {
    let mut result = Vec::new(); // 初期容量0 - 何度も再割り当て発生
    for item in items {
        result.push(process_item(item));
    }
    result
}
```

#### 良い例

```rust
fn process_items(items: &[Item]) -> Vec<ProcessedItem> {
    // 事前に容量確保 - 再割り当て回数を削減
    let mut result = Vec::with_capacity(items.len());
    for item in items {
        result.push(process_item(item));
    }
    result
}

// さらに良い例: イテレータ使用
fn process_items(items: &[Item]) -> Vec<ProcessedItem> {
    items.iter().map(process_item).collect()
}
```

#### 理由

- メモリ再割り当ての回数削減
- パフォーマンス向上(2-3倍高速化の場合も)
- メモリフラグメンテーション軽減

---

### optimize-string-concat

**目的**: 非効率な文字列結合の最適化

#### ダメな例

```rust
fn build_message(name: &str, status: &str) -> String {
    // to_string() + &str は非効率
    name.to_string() + " is " + status
}

fn build_long_string(parts: &[&str]) -> String {
    let mut result = String::new();
    for part in parts {
        // ループ内push_str - 容量不足で何度も再割り当て
        result.push_str(part);
    }
    result
}
```

#### 良い例

```rust
fn build_message(name: &str, status: &str) -> String {
    // format!マクロで効率的
    format!("{} is {}", name, status)
}

fn build_long_string(parts: &[&str]) -> String {
    // 事前容量確保
    let capacity = parts.iter().map(|s| s.len()).sum();
    let mut result = String::with_capacity(capacity);
    for part in parts {
        result.push_str(part);
    }
    result
}
```

#### 理由

- 不要なメモリ割り当て削減
- 文字列操作のパフォーマンス向上
- メモリ使用量の最適化

---

### no-blocking-in-async

**目的**: `async` 関数内での同期 I/O 操作を禁止

#### ダメな例

```rust
async fn load_config() -> Result<Config, Error> {
    // async関数内で同期I/O - スレッドブロッキング
    let content = std::fs::read_to_string("config.toml")?;

    // 同期sleep - 他のタスクもブロック
    std::thread::sleep(Duration::from_secs(1));

    parse_config(&content)
}
```

#### 良い例

```rust
async fn load_config() -> Result<Config, Error> {
    // async版I/O - 他のタスクをブロックしない
    let content = tokio::fs::read_to_string("config.toml").await?;

    // async版sleep
    tokio::time::sleep(Duration::from_secs(1)).await;

    parse_config(&content)
}
```

#### 理由

- 非同期実行環境でのスレッドブロッキング回避
- 並行性の維持
- スケーラビリティの確保

---

### prefer-iterator-over-loop

**目的**: 単純な `for` 文よりイテレータチェーン推奨

#### ダメな例

```rust
fn process_numbers(numbers: &[i32]) -> Vec<String> {
    let mut result = Vec::new();
    for &num in numbers {
        if num > 0 {
            result.push(format!("positive: {}", num));
        }
    }
    result
}
```

#### 良い例

```rust
fn process_numbers(numbers: &[i32]) -> Vec<String> {
    numbers
        .iter()
        .filter(|&&num| num > 0)
        .map(|&num| format!("positive: {}", num))
        .collect()
}
```

#### 理由

- 関数型プログラミングの利点
- 意図がより明確
- チェイン可能で拡張性が高い

---

## コード組織

### module-size-limit

**目的**: 大きすぎるモジュールの警告

#### ダメな例

```rust
// 1つのファイルに10個以上の関数 - 責務が不明確
mod user_management {
    pub fn create_user() { /* ... */ }
    pub fn update_user() { /* ... */ }
    pub fn delete_user() { /* ... */ }
    pub fn validate_email() { /* ... */ }
    pub fn hash_password() { /* ... */ }
    pub fn send_welcome_email() { /* ... */ }
    pub fn log_user_action() { /* ... */ }
    pub fn calculate_permissions() { /* ... */ }
    pub fn format_username() { /* ... */ }
    pub fn cleanup_old_sessions() { /* ... */ }
    pub fn generate_api_key() { /* ... */ }
    // さらに多数の関数...
}
```

#### 良い例

```rust
// 責務で分割
mod user {
    pub fn create() { /* ... */ }
    pub fn update() { /* ... */ }
    pub fn delete() { /* ... */ }
}

mod validation {
    pub fn validate_email() { /* ... */ }
    pub fn validate_password() { /* ... */ }
}

mod auth {
    pub fn hash_password() { /* ... */ }
    pub fn generate_api_key() { /* ... */ }
}

mod notification {
    pub fn send_welcome_email() { /* ... */ }
}
```

#### 理由

- 単一責務原則の遵守
- コードの可読性・保守性向上
- テストのしやすさ向上

---

### error-context-required

**目的**: エラーにコンテキスト情報を追加

#### ダメな例

```rust
fn load_user_config(user_id: u32) -> Result<Config, Error> {
    let path = format!("/users/{}/config.toml", user_id);
    // エラー情報不足 - どのファイルで何が失敗したか不明
    let content = std::fs::read_to_string(&path)?;
    let config = toml::from_str(&content)?;
    Ok(config)
}
```

#### 良い例

```rust
use anyhow::{Context, Result};

fn load_user_config(user_id: u32) -> Result<Config> {
    let path = format!("/users/{}/config.toml", user_id);

    // 詳細なエラーコンテキスト
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("ユーザー{}の設定ファイル読み込み失敗: {}", user_id, path))?;

    let config = toml::from_str(&content)
        .with_context(|| format!("ユーザー{}の設定ファイル解析失敗", user_id))?;

    Ok(config)
}
```

#### 理由

- デバッグ時の問題特定が容易
- エラーログの品質向上
- 運用時のトラブルシューティング効率化

---

## セキュリティ

### no-hardcoded-credentials

**目的**: ハードコードされた認証情報を検出

#### ダメな例

```rust
fn connect_to_database() -> Connection {
    // ハードコードされた認証情報 - セキュリティリスク
    let password = "super_secret_password_123";
    let api_key = "sk-1234567890abcdef1234567890abcdef";

    Database::connect("localhost", "admin", password)
}
```

#### 良い例

```rust
use std::env;

fn connect_to_database() -> Result<Connection, DatabaseError> {
    // 環境変数から取得 - セキュア
    let password = env::var("DB_PASSWORD")
        .context("DB_PASSWORD環境変数が設定されていません")?;
    let api_key = env::var("API_KEY")
        .context("API_KEY環境変数が設定されていません")?;

    Database::connect("localhost", "admin", &password)
}
```

#### 理由

- 認証情報の漏洩防止
- 環境ごとの設定分離
- セキュリティベストプラクティス遵守

---

### secure-random-required

**目的**: セキュリティ用途にセキュアな乱数生成を要求

#### ダメな例

```rust
fn generate_session_token() -> String {
    use rand::Rng;

    // 非セキュアな乱数 - 予測可能性のリスク
    let mut rng = rand::thread_rng();
    (0..32).map(|_| rng.gen::<u8>()).collect()
}
```

#### 良い例

```rust
fn generate_session_token() -> Result<String, CryptoError> {
    use rand::{rngs::OsRng, RngCore};

    // セキュアな乱数生成器
    let mut rng = OsRng;
    let mut token = vec![0u8; 32];
    rng.fill_bytes(&mut token);

    Ok(hex::encode(token))
}
```

#### 理由

- 暗号学的に安全な乱数生成
- セキュリティトークンの品質保証
- 攻撃耐性の向上

---

## プロジェクト固有ルール

### no-get-prefix

**目的**: Rust の `getter` 命名規則遵守

#### ダメな例

```rust
impl User {
    pub fn get_name(&self) -> &str { &self.name } // get_プレフィックス不要
}
```

#### 良い例

```rust
impl User {
    pub fn name(&self) -> &str { &self.name } // Rustの慣例
}
```

---

### no-use-alias

**目的**: use文でのエイリアス禁止

#### ダメな例

```rust
use std::collections::HashMap as Map; // エイリアス使用
```

#### 良い例

```rust
use std::collections::HashMap; // 直接使用
```

---

### no-type-result-override

**目的**: `Result` 型の上書き禁止

#### ダメな例

```rust
type Result<T> = std::result::Result<T, MyError>; // 標準Result型を隠蔽
```

#### 良い例

```rust
type MyResult<T> = std::result::Result<T, MyError>; // 独自の型名使用
```

---

### no-file-level-external-use

**目的**: ファイルトップレベルでの外部 `use` 禁止

#### ダメな例

```rust
use external_crate::SomeType; // ファイル先頭での外部use

fn main() {
    let instance = SomeType::new();
}
```

#### 良い例

```rust
fn main() {
    use external_crate::SomeType; // 関数内でのuse
    let instance = SomeType::new();
}
```

---

### prefer-nested-result

**目的**: ネストされた `Result` 型の使用を推奨

---

## ルール無効化

特定の箇所でルールを無効にする場合:

```rust
// 単一ルール無効化
// ast-grep-ignore: error-context-required
let value = some_operation()?;

// 複数ルール無効化
// ast-grep-ignore: no-hardcoded-credentials, no-file-level-external-use
use some_crate::SomeType;

// 全ルール無効化
// ast-grep-ignore
dangerous_code_here();
```

---

## 参考資料

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [ast-grep Documentation](https://ast-grep.github.io/)
- [Clippy Lints Reference](https://rust-lang.github.io/rust-clippy/master/)
- プロジェクト `project_rules.md`
