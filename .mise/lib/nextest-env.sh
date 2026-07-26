# Shared nextest thread-count and quiet-flag resolution.
# Source-only — not meant to be executed directly. Sourced by test/coverage
# tasks in .mise/tasks.toml to avoid duplicating the same cpu/thread math in
# four separate `run` scripts.
_cpus=$(nproc 2>/dev/null || sysctl -n hw.logicalcpu 2>/dev/null || echo 4)
_threads=$(( _cpus * 7 / 10 < 1 ? 1 : _cpus * 7 / 10 ))
[[ "${GITHUB_ACTIONS:-}" == "true" ]] && _threads="$_cpus"

_quiet=""
[[ "${CLAUDECODE:-}" == "1" ]] && _quiet="--cargo-quiet --cargo-quiet --status-level fail"
