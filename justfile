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

# cargo check
cargo-check:
	cargo check --all-targets --all-features

# cargo clippy (supports strict warnings)
cargo-clippy:
	cargo clippy --all-targets --all-features -- -D warnings -D clippy::all

# cargo formatting (supports check mode)
cargo-fmt:
	cargo fmt -- --check

# cargo llvm-cov
cargo-llvm-cov:
	cargo llvm-cov --lcov --output-path target/lcov.info

# cargo test
cargo-test *args="":
	cargo test {{args}}

# cross build for specific target
cross-build target:
	cargo build --target {{ target }} --release

# dprint formatting (supports check mode)
dprint *args="":
	dprint fmt {{args}}
