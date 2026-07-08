# 0002. Separate Canvas CSS from Production Build

- Status: Accepted
- Date: 2026-07-07
- Deciders: naa0yama

## Context

GrapesJS は編集対象を iframe に描画する。キャンバスに DaisyUI / Tailwind の
スタイルが当たらないとブロックが素の見た目になる。

本番 brust-web は pnpm + `@tailwindcss/cli` で Tailwind v4 をビルドし、
`@source` ディレクティブで使用クラスのみをパージした `output.css` を生成する。
GrapesJS のブロック定義 (`blocks/daisy-blocks.js`) に含まれるクラスは
Askama テンプレートには存在しないため、本番ビルドにキャンバス CSS を統合すると
パージでブロックのスタイルが失われる。

## Decision

エディタキャンバス (iframe) の CSS と本番アプリの CSS を分離する。

- **キャンバス**: `daisyui` パッケージ同梱の未パージ CSS +
  `@tailwindcss/browser` (ブラウザ内 JIT、全クラス利用可) を
  `node_modules` から local 参照。両パッケージとも
  `tools/wireframe/package.json` の devDeps に固定バージョンで宣言し
  (`daisyui` は本番側 `crates/brust-web/package.json` にも同版で宣言)、
  root の `pnpm-lock.yaml` (workspace 単一 lockfile) で一致を保証。
- **本番アプリ**: 従来通り `pnpm run build` → `output.css` (パージ済み)。

## Consequences

- (+) ブロック定義が本番ビルドから独立し、ブロック追加時に `@source` 設定変更不要
- (+) ブラウザ JIT でどのユーティリティも即反映、設計体験が良い
- (+) 本番 CSS のパージ精度を損なわない
- (+) local file 参照のため CDN 侵害・オフライン制約・SRI ハッシュ管理が不要
- (+) バージョン一致を lockfile が担保するため本番との差が最小
- (-) キャンバス用と本番用で 2 系統の CSS が併存する
  (責務が異なるため許容)
- (-) `daisyui` / `@tailwindcss/browser` のパッケージ内部レイアウトが
  minor 版で変わった場合、`index.html` の相対パスを更新する必要がある

## Alternatives Considered

- **CDN 版 (`cdn.jsdelivr.net/npm/daisyui@5` 等) を流用** —
  当初案。オフライン不可、CDN 侵害面、SRI ハッシュを毎バージョン管理する必要。
  local 参照で全て解消できるため却下。
- **自前 `output.css` をキャンバスに流す** — ブロック定義のクラスが
  Tailwind v4 パージで消え、キャンバスのスタイルが崩れる。却下。
- **ブロック定義ファイルを `@source` に追加してパージ対象にする** — 本番ビルドに
  wireframe 専用クラスが混入し、CSS サイズが増加する。責務分離の観点から却下。

## History

- 2026-07-07: initial version (local `node_modules` 参照で決着)
