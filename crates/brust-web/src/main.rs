//! Entry point for the brust-web binary.

fn main() {
    // Phase 7 will replace this with tokio::main + Axum serve.
    // eprintln used to prevent clippy::missing_const_for_fn on the stub.
    #[allow(clippy::print_stderr)]
    {
        eprintln!("{} (stub)", brust_web::app_version());
    }
}
