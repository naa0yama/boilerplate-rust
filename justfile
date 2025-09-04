[private]
@default: help

# show help message
@help:
	echo "Usage: just <recipe>"
	echo ""
	just --list

# cargo build
cargo-build profile="debug":
	cargo build {{ if profile == "release" { "--release" } else { "" } }}

# cargo build --timings
cargo-build-timings:
	cargo build --timings

# cargo check
cargo-check:
	cargo check --all-targets --all-features

# cargo clippy (configured in Cargo.toml [lints] section)
cargo-clippy:
	cargo clippy --all-targets --all-features

# cargo formatting (supports check mode)
cargo-fmt:
	cargo fmt -- --check

# cargo llvm-cov
cargo-llvm-cov:
	cargo llvm-cov --lcov --output-path target/lcov.info

# cargo test
cargo-test *args="":
	cargo test {{args}}

# dprint formatting (supports check mode)
dprint *args="":
	dprint {{args}}

# cross build for specific target
zigbuild target="x86_64-unknown-linux-gnu":
	cargo zigbuild --release --target {{ target }} --verbose

# build all Tier 1 targets
zigbuild-all:
	cargo zigbuild --release --target aarch64-apple-darwin
	cargo zigbuild --release --target aarch64-unknown-linux-gnu
	cargo zigbuild --release --target x86_64-pc-windows-gnu
	cargo zigbuild --release --target x86_64-unknown-linux-gnu

# ast-grep project rules check
project-rules-check:
	sg scan --error --color never

# comprehensive lint check (combines clippy and project rules)
lint-all:
	just cargo-clippy
	just project-rules-check
