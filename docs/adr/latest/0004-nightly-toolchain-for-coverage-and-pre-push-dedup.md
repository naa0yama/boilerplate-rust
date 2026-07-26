# 0004. Nightly toolchain for coverage measurement, and pre-push test dedup

- Status: Accepted
- Date: 2026-07-26
- Deciders: naa0yama

## Context

`cargo-llvm-cov` を使ったカバレッジ計測で、I/O 専用関数や TUI 関数を計測対象から
除外するには `#[coverage(off)]` 属性(`coverage_attribute` feature)が必要。この
属性は 2026-07 時点でも stable Rust に未 stabilize:

- stabilize PR [rust-lang/rust#130766](https://github.com/rust-lang/rust/pull/130766)
  が一度 merge されたが、プロセス不備を理由に
  [#134672](https://github.com/rust-lang/rust/pull/134672) で revert。
- 再 stabilize PR [#134942](https://github.com/rust-lang/rust/pull/134942) と
  tracking issue [#134749](https://github.com/rust-lang/rust/issues/134749) で
  議論継続中。
- 最新 nightly unstable-book にも `coverage_attribute` が unstable feature として
  掲載され続けている。

このため本リポジトリでは nightly toolchain を固定 pin(`nightly-2026-03-15`、
`rustup component add llvm-tools-preview`)し、`cargo +nightly-2026-03-15 llvm-cov
nextest` でカバレッジを計測、対象外関数は
`#[cfg_attr(coverage_nightly, coverage(off))]` で除外している
(`cargo-llvm-cov` が nightly 使用時に自動設定する `--cfg coverage_nightly` に連動)。

別件として、`pre-push` task が `test`(stable)と `coverage`(nightly、
instrumented)の両方に依存しており、フルテストスイートを毎回二重実行、
`target/debug` + `target/llvm-cov-target` の両方が肥大化する問題があった
(`coverage` は既に nextest フルスイートを内包しているため `test` は
シグナルとして冗長)。

## Decision

1. **nightly 依存を維持する**: `coverage_attribute` が stable 化されるまで、
   カバレッジ計測は `nightly-2026-03-15` 固定 toolchain を使用し続ける。
2. **`pre-push` から `test` 依存を除外し、`pre-commit` に移す**: `pre-push` は
   `coverage` のみを実行し二重実行を解消する。stable-toolchain でのテスト実行
   自体は `pre-commit`(`.mise/tasks.toml` `[pre-commit].depends` に `test` 追加)
   へ移動し、コミットごとの高速フィードバックとして維持する。
3. **`coverage` 実行後に `target/llvm-cov-target` を自動削除**: EXIT trap で
   `cargo +nightly-2026-03-15 llvm-cov clean`(`--workspace` フラグなし)を実行。
   CI(`GITHUB_ACTIONS=true`)および `COVERAGE_KEEP=1` 指定時は削除をスキップ。
4. テスト実行を伴う 3 タスク(`test` / `coverage` / `test:trace`)で重複していた
   スレッド数計算・quiet フラグ判定ロジックを `.mise/lib/nextest-env.sh` に
   集約し、`source` して共有する。`coverage:html` は Decision 5 により
   テスト再実行なしでレポートを開くだけのタスクに簡素化したため対象外。
5. **`coverage` タスクで lcov.info と html を同一実行内で両方生成する**:
   `cargo llvm-cov nextest` は `--lcov` と `--html` を同時指定できない
   (CLI 側で相互排他、実測確認済み)ため、`--lcov` でテストを実行した直後に
   同一 profraw キャッシュから `cargo llvm-cov report --html
   --output-dir target/coverage-html` を追加実行し、テスト再実行なしで
   両形式を得る。`--output-dir` はデフォルトの `target/llvm-cov/html` ではなく
   明示パスを指定する — デフォルト位置は `cargo llvm-cov clean`
   (`--workspace` なし)の削除対象に含まれることを実測で確認したため、
   EXIT trap 実行時に生成直後の html が消える事故を避けるための対応。
   `coverage:html` タスクは `coverage` に依存せず、生成済みレポートの
   存在確認のみ行い開くだけのタスクに簡素化する(依存させるとレポートを
   開くだけの操作で毎回フルテストスイートが再実行されてしまうため)。
   `--fail-under-lines 90` は `nextest` 実行時でも html 生成の `report`
   呼び出しでもなく、フォーマット指定なしの 3 回目の `report` 呼び出しに
   分離して評価させる。理由は 2 つ:
   (a) `set -euo pipefail` 下で `nextest` 側に閾値判定を置くと、閾値未達時に
   html 生成前でスクリプトが終了し、最も html が見たい(未カバー箇所を
   確認したい)場面で html が出力されない事故になる。
   (b) `--html` 指定時、`report` は `llvm-cov show -format=html` へ直接
   処理を委譲し、cargo-llvm-cov 自身によるカバレッジサマリ表の stdout
   出力を行わない(upstream 実測確認済み)。`--html` 呼び出しに
   `--fail-under-lines` を同居させると、CI/ローカルログからサマリ表が
   消える副作用が生じるため、フォーマット指定なしの呼び出しを分離して
   サマリ表出力と閾値判定を両立させる。同一 profraw データの再利用のため
   3 回目もテスト再実行は発生せず低コスト。

## Consequences

**Positive**

- `pre-push` 1 回あたりのディスク使用量倍増を解消。
- `coverage` 実行のたびに `target/llvm-cov-target` を自動的に手放すため、
  ローカル開発環境のディスク圧迫が恒常化しない。
- スレッド数・quiet フラグ判定ロジックの重複が解消され、変更箇所が 1 箇所に
  集約された。
- `coverage` 実行 1 回で lcov.info と html レポートの両方が揃うため、
  `coverage:html` 単独実行(テスト再実行なし)でも常に最新レポートを
  参照できる。

**Negative**

- stable-toolchain でのテスト実行(`test`)が `pre-commit` の一部になったため、
  コミットのたびに実行コストがかかる(実測 ~5 秒、77 テスト)。ただし
  `coverage`(instrumented、より低速)を `pre-push` に留めることで、頻度の
  高い `pre-commit` は軽量な検証、低頻度の `pre-push` は網羅的な検証という
  役割分担は維持される。
- CI の `test` job は引き続き `mise run coverage`(nightly)のみを実行し、
  `cross-check` job は `cargo check`(コンパイルのみ)。CI 上で
  stable-toolchain のテスト実行を独立して確認したい場合は別途 job 追加が
  必要(本 ADR の範囲外)。
- nightly toolchain 依存が継続するため、nightly のブレーキングチェンジや
  toolchain pin の定期更新コストが残る。

## Alternatives Considered

- **`cargo llvm-cov clean --workspace` で cleanup する**: 採用せず。
  `du` で実測したところ、`--workspace` はワークスペースメンバーの
  ビルド成果物のみを削除し、依存クレートのビルドキャッシュ(ディレクトリの
  大部分を占める)は残存する。ディレクトリ全体を削除する目的には
  フラグなしの `cargo llvm-cov clean` が必要。
- **`#[coverage(off)]` を諦めて計測対象から除外しない**: 採用せず。
  I/O 専用・TUI 関数はテスト困難で `--fail-under-lines 90` の閾値達成を
  不必要に阻害するため、除外の仕組み自体は維持する必要がある。
- **stable toolchain のみで運用し nightly を廃止**: 採用せず。
  `coverage_attribute` が stabilize されていない現状では、除外属性を
  失うことになり `--fail-under-lines` 閾値運用が破綻する。

## History

- 2026-07-26: initial version
