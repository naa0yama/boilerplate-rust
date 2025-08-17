#!/usr/bin/env bash
set -euxo pipefail

# dirs from mounts
mkdir -p ~/.claude/

# files from mounts
touch \
	~/.claude.json \
	~/.gitconfig \
	~/.gitignore_global

lefthook install
