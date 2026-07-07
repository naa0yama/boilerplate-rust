//! Entry point for the brust-web binary.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context as _;
use axum::{Router, middleware::from_fn_with_state, routing::get};
use clap::Parser as _;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use brust_web::{
    assets,
    cli::{Cli, Commands},
    routes,
    telemetry::{self, metrics::Meters},
    trace::{OtelHttpServerMakeSpan, OtelOnResponse, server_metrics_mw},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let cli = Cli::parse();
    match cli.command {
        Commands::Version => {
            #[allow(clippy::print_stdout)]
            {
                println!("{}", brust_web::app_version());
            }
        }
        Commands::Serve(args) => {
            let telemetry = telemetry::init_telemetry(
                env!("CARGO_PKG_NAME"),
                option_env!("GIT_HASH").unwrap_or("unknown"),
            )
            .context("failed to initialise telemetry")?;

            let meters = Arc::new(Meters::new());

            let router = Router::new()
                .route("/", get(routes::index::handler))
                .route("/health", get(routes::health::handler))
                .merge(assets::router())
                .layer(from_fn_with_state(Arc::clone(&meters), server_metrics_mw))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(OtelHttpServerMakeSpan)
                        .on_response(OtelOnResponse),
                );

            let listener = TcpListener::bind(&args.bind)
                .await
                .with_context(|| format!("failed to bind to {}", args.bind))?;
            tracing::info!(port = %args.bind, "server started");

            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c().await.ok();
            })
            .await
            .context("server error")?;

            telemetry.shutdown();
        }
    }
    Ok(())
}
