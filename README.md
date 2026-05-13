# Boilerplate-Rust

![coverage](https://raw.githubusercontent.com/naa0yama/boilerplate-rust/badges/coverage.svg)
![test execution time](https://raw.githubusercontent.com/naa0yama/boilerplate-rust/badges/time.svg)

Rust プロジェクトのための開発テンプレート

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

## devcontainer CLI + traefik による開発環境 (WSL2)

VS Code を使わず tmux + yazi + claude code などで複数 worktree / 複数プロジェクトを
並行開発する場合は、devcontainer CLI と traefik を組み合わせる方法を使います。
ポート衝突なしに各 devcontainer へ `p<port>.<branch>.<project>.localhost:8080` で
アクセスできます。

### ホスト前提条件

WSL2 ホストに以下が必要です。いずれもユーザーグローバルの mise で管理します
(プロジェクトの `mise.toml` には含まれていません)。

```bash
# Node.js (devcontainer-cli の実行に必要)
mise use --global node@lts

# devcontainer CLI
mise use --global devcontainer-cli@0.86.0
```

`dev:up` 実行時に、ホストで `ssh-agent` と `gpg-agent` が起動していれば
SSH エージェント転送と GPG 署名がコンテナ内で自動的に有効になります。
未起動の場合は起動時に警告が出力されますが、開発は続行できます。

### 初回セットアップ (WSL2 ホストで1回だけ実行)

前提条件のインストール後、以下を実行します。
traefik バイナリの取得・設定・systemd user service への登録を一括で行います。

```bash
mise run traefik:setup
```

traefik の状態確認:

```bash
systemctl --user status traefik
```

### devcontainer の起動と停止

```bash
mise run dev:up      # 現在の worktree の devcontainer を起動
mise run dev:down    # 現在の worktree の devcontainer を停止・削除
mise run dev:exec    # 稼働中の devcontainer に bash で接続
mise run dev:status  # 稼働中の devcontainer 一覧を表示
```

`dev:up` は `devcontainer.json` の `portsAttributes` に定義された全ポートに対して
traefik ルーティングを自動設定します。起動後に以下の形式の URL が表示されます。

```
http://p<port>.<branch>.<project>.localhost:8080
```

例 (ポート `5080`、ブランチ `feature/add-auth`、プロジェクト `boilerplate-rust`):

```
http://p5080.feature-add-auth.boilerplate-rust.localhost:8080
```

### 複数 worktree での利用

```bash
# 1つ目の worktree
cd /path/to/boilerplate-rust
mise run dev:up
# -> http://p5080.main.boilerplate-rust.localhost:8080

# 2つ目の worktree (別ブランチ)
cd /path/to/boilerplate-rust-feat
mise run dev:up
# -> http://p5080.feature-x.boilerplate-rust.localhost:8080
```

ブランチ名は DNS ラベル形式 (小文字英数字とハイフン、63文字以内) に自動変換されます。

## 使い方

すべてのタスクは `mise run <task>` で実行します。

### 基本操作

```bash
mise run build            # デバッグビルド
mise run build:release    # リリースビルド
mise run test             # テスト実行
mise run test:watch       # TDD ウォッチモード
mise run test:doc         # ドキュメントテスト
```

### コード品質

```bash
mise run fmt              # フォーマット (cargo fmt + dprint)
mise run fmt:check        # フォーマットチェック
mise run clippy           # Lint
mise run clippy:strict    # Lint (warnings をエラー扱い)
mise run ast-grep         # ast-grep カスタムルールチェック
```

### コミット前チェック

```bash
mise run pre-commit       # clean:sweep + fmt:check + clippy:strict + ast-grep + lint:gh + check:no-plans
```

## プロジェクト構造

```
.
├── .cargo/                     # Cargo設定
│   └── config.toml
├── .devcontainer/              # Dev Container設定
│   ├── devcontainer.json       # Dev Container設定ファイル
│   ├── initializeCommand.sh    # 初期化コマンド
│   └── postStartCommand.sh     # 起動後コマンド
├── .githooks/                  # Git hooks (mise run 連携)
│   ├── commit-msg              # Conventional Commits 検証
│   ├── post-checkout           # tmux ペイン名を repo:branch に設定
│   ├── pre-commit              # コミット前チェック
│   └── pre-push                # プッシュ前チェック
├── .github/                    # GitHub Actions & 設定
│   ├── actions/                # カスタムアクション
│   ├── graft/                  # graft マニフェスト (テンプレートリポジトリからのファイル同期設定)
│   ├── workflows/              # CI/CD ワークフロー
│   ├── labeler.yml
│   ├── project-config.json         # CI/リリース設定 (ビルドターゲット・タイムアウト・apt パッケージ等)
│   └── release.yml
├── .mise/                      # mise タスク定義
│   ├── tasks.toml              # 共通タスク定義 (boilerplate から管理)
│   └── overrides.toml          # プロジェクト固有のタスク上書き
├── .vscode/                    # VS Code設定
│   ├── launch.json             # デバッグ設定
│   └── settings.json           # ワークスペース設定
├── ast-rules/                  # ast-grep プロジェクトルール
├── crates/                     # ワークスペースクレート
│   └── brust/                  # CLI バイナリクレート
│       ├── src/
│       │   ├── main.rs         # アプリケーションのエントリーポイント
│       │   ├── libs.rs         # モジュール定義
│       │   ├── metrics.rs      # OTel メトリクス instruments
│       │   └── libs/
│       │       ├── count.rs    # イテレーションカウンターモジュール
│       │       ├── hello.rs    # Hello モジュール
│       │       └── http.rs     # HTTP クライアント (OTel メトリクス付き)
│       ├── tests/
│       │   └── integration_test.rs  # 統合テスト
│       ├── build.rs            # ビルドスクリプト
│       └── Cargo.toml          # クレート設定
├── docs/                       # ドキュメント
├── .editorconfig               # エディター設定
├── .gitignore                  # Git除外設定
├── .octocov.yml                # カバレッジレポート設定
├── .tagpr                      # タグ&リリース設定
├── Cargo.lock                  # 依存関係のロックファイル
├── Cargo.toml                  # ワークスペース設定と共有依存関係
├── deny.toml                   # cargo-deny 設定
├── Dockerfile                  # Dockerイメージ定義
├── dprint.jsonc                # Dprint フォーマッター設定
├── LICENSE                     # ライセンスファイル
├── mise.toml                   # ツール管理 (タスクは .mise/ を参照)
├── README.md                   # このファイル
├── renovate.json               # Renovate自動依存関係更新設定
├── rust-toolchain.toml         # Rust toolchain バージョン固定
└── sgconfig.yml                # ast-grep 設定ファイル
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

### サードパーティライセンスについて

Dev Container の起動時に [OpenObserve Enterprise Edition](https://openobserve.ai/) が自動的にダウンロード・インストールされます。Enterprise 版は MCP (Model Context Protocol) サーバー機能など OSS 版にはない付加機能を備えているため採用しています。Enterprise 版は 200GB/Day のインジェストクォータ内であれば無料で利用できます。

OpenObserve Enterprise Edition は [EULA (End User License Agreement)](https://openobserve.ai/enterprise-license/) の下で提供されており、OSS 版 (AGPL-3.0) とはライセンスが異なります。Enterprise 版の機能一覧は [OpenObserve Enterprise](https://openobserve.ai/docs/features/enterprise/) を参照してください。

## 参考資料

- [The Rust Programming Language 日本語版](https://doc.rust-jp.rs/book-ja/)
- [Developing inside a Container](https://code.visualstudio.com/docs/devcontainers/containers)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)

## Troubleshooting

### Rust debug

```bash
RUST_LOG=trace RUST_BACKTRACE=1 cargo run -- help
```
