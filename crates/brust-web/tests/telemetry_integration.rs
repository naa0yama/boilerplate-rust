//! Integration tests for `brust_web::telemetry` — each test runs in its own
//! process (nextest default), so `tracing_subscriber::try_init` succeeds once.
#![allow(clippy::unwrap_used)]
#![allow(missing_docs)]

use brust_web::telemetry::{TelemetryGuard, init_telemetry};

#[test]
fn init_telemetry_disabled_when_no_endpoint() {
    // SAFETY: single-threaded test process (nextest isolates each test)
    unsafe { std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT") };
    let guard = init_telemetry("test-svc", "deadbeef").expect("init_telemetry failed");
    assert!(
        matches!(guard, TelemetryGuard::Disabled),
        "expected Disabled variant"
    );
    guard.shutdown();
}

/// Spin up a minimal HTTP server that accepts OTLP POST requests and returns 200.
#[cfg(feature = "otel")]
fn start_fake_otlp() -> u16 {
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    std::thread::spawn(move || {
        while let Ok((mut stream, _)) = listener.accept() {
            use std::io::{Read as _, Write as _};
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            // Respond with minimal HTTP 200
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
        }
    });

    port
}

#[cfg(feature = "otel")]
#[test]
#[cfg_attr(miri, ignore)] // gethostname -> rustix::uname triggers Miri UB on uninit sysname bytes
fn init_telemetry_otlp_variant_with_fake_receiver() {
    // reqwest uses rustls-no-provider; install ring before building any client
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();
    let port = start_fake_otlp();
    // SAFETY: single-threaded test process (nextest isolates each test)
    unsafe {
        std::env::set_var(
            "OTEL_EXPORTER_OTLP_ENDPOINT",
            format!("http://127.0.0.1:{port}"),
        );
    }
    let guard = init_telemetry("test-svc", "deadbeef").expect("init_telemetry failed");
    assert!(
        matches!(guard, TelemetryGuard::Otlp { .. }),
        "expected Otlp variant"
    );
    guard.shutdown();
    // SAFETY: single-threaded test process
    unsafe { std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT") };
}
