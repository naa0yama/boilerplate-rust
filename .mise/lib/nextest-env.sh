# Shared nextest thread-count and quiet-flag resolution.
# Source-only — not meant to be executed directly. Sourced by test/coverage
# tasks in .mise/tasks.toml to avoid duplicating the same cpu/thread math in
# four separate `run` scripts.
_cpus=$(nproc 2>/dev/null || sysctl -n hw.logicalcpu 2>/dev/null || echo 4)
_threads=$(( _cpus * 7 / 10 < 1 ? 1 : _cpus * 7 / 10 ))
# `if` (not `[[ ]] && ...`) — under `set -e` in the sourcing caller, a false
# `&&` expression as this script's last-executed statement would make
# `source` itself return 1, silently aborting the caller before any test
# ever runs. `if` always returns 0 regardless of the condition's outcome.
if [[ "${GITHUB_ACTIONS:-}" == "true" ]]; then
  _threads="$_cpus"
fi

_quiet=""
# Same set -e/source gotcha as above — keep as `if`, not `[[ ]] && ...`.
if [[ "${CLAUDECODE:-}" == "1" ]]; then
  _quiet="--cargo-quiet --cargo-quiet --status-level fail"
fi
