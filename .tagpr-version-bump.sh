#!/usr/bin/env bash
# Bump [workspace.package] version in Cargo.toml.
# tagpr v1.19.0 added Cargo.toml support but only handles [package], not
# [workspace.package], so we manage the version bump manually via this script.
# Ref: https://github.com/Songmu/tagpr/pull/350
set -euo pipefail

NEXT_VER="${TAGPR_NEXT_VERSION#v}"
sed -i "s/^version = \"[^\"]*\"/version = \"${NEXT_VER}\"/" Cargo.toml
cargo generate-lockfile
git add Cargo.toml Cargo.lock
