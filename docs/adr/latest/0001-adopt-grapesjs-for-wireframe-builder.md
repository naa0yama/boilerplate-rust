# 0001. Adopt GrapesJS for Wireframe Builder

- Status: Accepted
- Date: 2026-07-07
- Deciders: naa0yama

## Context

brust-web の初期画面設計を DaisyUI コンポーネントを意識しながら行う
「左のブロック一覧からキャンバスへドラッグ配置」する UI が必要になった。
制約: OSS / 無料 / 自己ホスト / IaC 再現可能。

対象スタック: Axum + Askama + HTMX + DaisyUI v5 (PWA / SSR / JS フレームワーク非依存)。
成果物は「人間が読む参照資料 (モック)」であり、Askama テンプレートへの自動変換は
スコープ外。

## Decision

GrapesJS (BSD-3-Clause) を自己ホストで採用する。
配置: `tools/wireframe/index.html` (単独 HTML)。pnpm workspace member として
`tools/wireframe/` を登録し、`grapesjs` を devDeps 宣言。
キャンバス (iframe) に DaisyUI v5 + `@tailwindcss/browser` (v4) を local
`node_modules` から注入し、DaisyUI のマークアップを Block Manager に
登録してドラッグ配置できるようにする。
MCP 連携は当面不要 — HTML ファイル受け渡しで運用する。

## Consequences

- (+) 自己ホスト・脱ロックイン・素の HTML/CSS 出力で Askama/HTMX に直行できる
- (+) DaisyUI v5 と直接組めるため第三者ラッパーに依存しない
- (+) pnpm workspace 化で本番アプリ (`crates/brust-web`) の依存と
  同一 lockfile で管理でき、バージョン一致が保証される
- (+) ブラウザ内完結でサーバサイドビルド不要
- (-) DaisyUI ブロック定義を自前で書く必要がある (llms.txt から流し込み可)
- (-) ブロックが積み上がると他ツールへの乗り換えコストが増える

## Alternatives Considered

- **Windframe** — DaisyUI v5 ネイティブの SaaS ビルダー。
  自己ホスト不可・クラウド SaaS のため自己ホスト志向と不一致。却下。
- **vocamen/web-mobile-html-builder** — GrapesJS + DaisyUI の既製ビルダー (自己ホスト可)。
  DaisyUI 2.38.1 / Tailwind 2.2 固定で v5/v4 と乖離が大きい。却下。
- **grape-js-daisy-ui (npm, MIT)** — DaisyUI を `dy-` プレフィックスでブロック化。
  対応 DaisyUI 版が不明瞭な小規模個人パッケージ。リスク高。却下。
- **Figma → Penpot** — Penpot 公式 Exporter で変換。変換ロス・プロトタイプ非対応・
  実 HTML 直行より遠回り。却下。
- **Open Design (nexu-io)** — ローカルファースト OSS (Apache-2.0)。
  今回の DaisyUI 意識 wireframe とは別軸のツール。将来再評価。

## History

- 2026-07-07: initial version
