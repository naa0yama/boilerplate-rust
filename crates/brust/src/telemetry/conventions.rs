//! brust-prefixed semantic conventions for app-specific telemetry.
//!
//! Mirrors the layout of `opentelemetry_semantic_conventions::{metric,
//! attribute}` to provide a single source of truth for `brust.*` names
//! across all signals (metrics today, tracing/logs in the future).
//! Use these constants instead of string literals to avoid typos and drift.

pub mod metric {
    pub const RUN_DURATION: &str = "brust.run.duration";
    pub const GREETING_COUNT: &str = "brust.greeting.count";
    pub const GREETING_ERRORS: &str = "brust.greeting.errors";
    pub const ITERATION_COUNT: &str = "brust.iteration.count";
    pub const ITERATION_DURATION: &str = "brust.iteration.duration";
    pub const ITERATION_IN_FLIGHT: &str = "brust.iteration.in_flight";
}

pub mod attribute {
    pub const COMMAND: &str = "brust.command";
    pub const GENDER: &str = "brust.gender";
}
