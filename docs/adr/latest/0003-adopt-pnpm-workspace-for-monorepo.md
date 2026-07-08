# 0003. Adopt pnpm Workspace for Monorepo Dep Management

- Status: Accepted
- Date: 2026-07-07
- Deciders: naa0yama

## Context

brust-web リポジトリはこれまで `crates/brust-web/package.json` /
`crates/brust-web/pnpm-workspace.yaml` / `crates/brust-web/pnpm-lock.yaml`
を単一 pnpm プロジェクトとして持ち、Rust `build.rs` から `pnpm install`
を呼び出して Tailwind v4 CLI で本番用 CSS をビルドしていた。

wireframe ビルダー (ADR-0001) を導入するにあたり、開発ツール依存
(grapesjs / @tailwindcss/browser / serve) を本番 frontend 依存
(daisyui / tailwindcss / htmx など) と同じ `package.json` に混在させると
「アプリの実行時依存」と「作業用ツール」の責務が曖昧になる。同時に
`daisyui` は本番と wireframe canvas の双方で参照するため、バージョン一致
を lockfile で保証したい。

## Decision

pnpm workspace 化する。

- root `/app/package.json` (最小、`private: true` + `packageManager`)
- root `/app/pnpm-workspace.yaml` に member 登録
  (`crates/brust-web`, `tools/wireframe`) と supply-chain 設定
  (`allowBuilds` / `minimumReleaseAge` / `minimumReleaseAgeExclude`) を集約
- root `/app/pnpm-lock.yaml` (workspace 単一 lockfile)
- `crates/brust-web/package.json` は残置 (本番 frontend 依存の宣言)
- `tools/wireframe/package.json` を新規追加 (wireframe 専用 devDeps)
- `crates/brust-web/pnpm-workspace.yaml` と `pnpm-lock.yaml` は削除
- `crates/brust-web/build.rs` の `cargo:rerun-if-changed=pnpm-lock.yaml` を
  `../../pnpm-lock.yaml` に修正 (1 行)。`pnpm install` の cwd は
  crates/brust-web のまま、pnpm が workspace root を自動検出

## Consequences

- (+) 本番アプリ依存と作業ツール依存が package.json 単位で分離される
- (+) 共有パッケージ (daisyui など) のバージョン一致を単一 lockfile で保証
- (+) `pnpm install` を root で 1 回実行するだけで全 member の依存が揃う
- (+) 将来 `tools/` 配下にツールを追加する際、member 登録だけで済む
- (+) supply-chain 設定 (minimumReleaseAge) を集中管理できる
- (-) pnpm hoisting の挙動を理解する必要 (symlink node_modules)
- (-) unwind コスト大 (lockfile 分割・member ごとの独立化・
  build.rs paths 復元) — 事実上 accept 後は戻さない前提

## Alternatives Considered

- **単一 `crates/brust-web/package.json` に全部集約** — 開発ツールと
  本番依存の責務混在。wireframe が独自 devDeps を大量に持つため
  `crates/brust-web/node_modules` に無関係アセットが増える。却下。
- **`tools/wireframe/` を workspace 外の独立 pnpm プロジェクトに** —
  daisyui バージョン一致を lockfile で保証できない。人手で pin する
  運用は将来 daisyui minor 更新時に事故る。却下。
- **`.devcontainer/grapesjs/` に置く** — export した wireframe HTML の
  受け渡し先として `tools/` 配下の方が自然。開発ツールカテゴリの
  一般化にも向く。却下。

## History

- 2026-07-07: initial version
