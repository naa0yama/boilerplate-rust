//! CLI argument definitions.

use clap::{Parser, Subcommand};

/// Top-level CLI for brust-web.
#[derive(Debug, Parser)]
#[command(name = "brust-web", version, about)]
pub struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run the HTTP server.
    Serve(ServeArgs),
    /// Print the version string.
    Version,
}

/// Arguments for the `serve` subcommand.
#[derive(Debug, clap::Args)]
pub struct ServeArgs {
    /// Socket address to bind (e.g. `0.0.0.0:3000`).
    #[arg(long, default_value = "0.0.0.0:3000", value_name = "ADDR")]
    pub bind: String,
}
