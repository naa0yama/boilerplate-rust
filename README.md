# Boilerplate-Rust

![coverage](docs/coverage.svg)
![coverage](docs/time.svg)

Rustプロジェクトのための開発テンプレート

## 概要

このプロジェクトは、Rust 開発を始めるためのボイラープレートです。Dev Containers に対応しており、VS Code での開発環境が簡単に構築できます。

## 必要要件

- Docker
- Visual Studio Code
- VS Code Dev Containers 拡張機能

## セットアップ

1. リポジトリをクローン:

```bash
git clone <repository-url>
cd boilerplate-rust
```

2. VS Codeでプロジェクトを開く:

```bash
code .
```

3. VS Codeのコマンドパレット（`Ctrl+Shift+P` / `Cmd+Shift+P`）から「Dev Containers: Reopen in Container」を選択

## 使い方

### ビルド

```bash
cargo build
```

### 実行

```bash
cargo run
```

### テスト

```bash
cargo test
```

### リリースビルド

```bash
cargo build --release
```

### コードフォーマット

```bash
dprint fmt
```

### Git フックの管理（lefthook）

```bash
# Git フックのインストール
lefthook install

# Git フックの実行
lefthook run pre-commit
```

## プロジェクト構造

```
.
├── .devcontainer/              # Dev Container設定
│   ├── devcontainer.json       # Dev Container設定ファイル
│   └── initializeCommand.sh    # 初期化コマンド
├── .vscode/                    # VS Code設定
│   ├── launch.json             # デバッグ設定
│   └── settings.json           # ワークスペース設定
├── src/
│   ├── main.rs                 # アプリケーションのエントリーポイント
│   ├── libs.rs                 # モジュール定義
│   └── libs/
│       └── hello.rs            # Helloモジュール
├── .editorconfig               # エディター設定
├── .gitignore                  # Git除外設定
├── Cargo.toml                  # プロジェクト設定と依存関係
├── Cargo.lock                  # 依存関係のロックファイル
├── Dockerfile                  # Dockerイメージ定義
├── LICENSE                     # ライセンスファイル
├── README.md                   # このファイル
├── dprint.jsonc                # Dprint フォーマッター設定
├── lefthook.yml                # Git hooks設定
└── renovate.json               # Renovate自動依存関係更新設定
```

## VSCode拡張機能

このプロジェクトの Dev Containers には、Rust開発を効率化する以下の拡張機能が含まれています：

### Rust開発

- **[rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)** - Rust言語サポート（コード補完、エラー検出、リファクタリング）
- **[CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)** - Rustプログラムのデバッグサポート
- **[Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml)** - Cargo.tomlファイルのシンタックスハイライトとバリデーション

### コード品質・フォーマット

- **[Biome](https://marketplace.visualstudio.com/items?itemName=biomejs.biome)** - 高速なフォーマッターとリンター
- **[dprint](https://marketplace.visualstudio.com/items?itemName=dprint.dprint)** - 高速なコードフォーマッター（設定ファイル: `dprint.jsonc`）
- **[EditorConfig for VS Code](https://marketplace.visualstudio.com/items?itemName=EditorConfig.EditorConfig)** - エディター設定の統一
- **[Error Lens](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens)** - エラーと警告をインラインで表示

### 開発支援

- **[Claude Code for VSCode](https://marketplace.visualstudio.com/items?itemName=Anthropic.claude-code)** - AIアシスタントによるコーディング支援
- **[Calculate](https://marketplace.visualstudio.com/items?itemName=acarreiro.calculate)** - 選択したテキストの計算式を評価
- **[indent-rainbow](https://marketplace.visualstudio.com/items?itemName=oderwat.indent-rainbow)** - インデントレベルを色分け表示
- **[Local History](https://marketplace.visualstudio.com/items?itemName=xyz.local-history)** - ファイルの変更履歴をローカルに保存

### テキスト編集

- **[lowercase](https://marketplace.visualstudio.com/items?itemName=ruiquelhas.vscode-lowercase)** - 選択テキストを小文字に変換
- **[uppercase](https://marketplace.visualstudio.com/items?itemName=ruiquelhas.vscode-uppercase)** - 選択テキストを大文字に変換
- **[Markdown All in One](https://marketplace.visualstudio.com/items?itemName=yzhang.markdown-all-in-one)** - Markdownファイルの編集支援

## ライセンス

このプロジェクトは [LICENSE](./LICENSE) ファイルに記載されているライセンスの下で公開されています。

## 参考資料

- [The Rust Programming Language 日本語版](https://doc.rust-jp.rs/book-ja/)
- [Developing inside a Container](https://code.visualstudio.com/docs/devcontainers/containers)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
