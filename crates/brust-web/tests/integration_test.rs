#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]
#![allow(missing_docs)]
use assert_cmd::cargo::cargo_bin_cmd;
use axum::body::Body;
use http::{Request, StatusCode};
use predicates::prelude::*;
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_200_with_json() {
    let app = brust_web::create_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn index_returns_200_html() {
    let app = brust_web::create_router();
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert!(
        headers["content-type"]
            .to_str()
            .unwrap()
            .contains("text/html")
    );
}

#[tokio::test]
async fn static_css_returns_200() {
    let app = brust_web::create_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/static/app.css")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn static_htmx_returns_200() {
    let app = brust_web::create_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/static/htmx.min.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
#[cfg_attr(miri, ignore)]
fn version_subcommand_prints_version() {
    let mut cmd = cargo_bin_cmd!("brust-web");
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
#[cfg_attr(miri, ignore)]
fn serve_subcommand_binds_and_responds() {
    use std::io::{BufRead as _, BufReader};
    use std::net::SocketAddr;
    use std::process::{Command, Stdio};
    use std::sync::mpsc;
    use std::time::Duration;

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("brust-web"))
        .args(["serve", "--bind", "127.0.0.1:0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .env("NO_COLOR", "1")
        .env_remove("OTEL_EXPORTER_OTLP_ENDPOINT")
        .spawn()
        .expect("failed to spawn brust-web serve");

    let stdout = child.stdout.take().expect("no stdout");
    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let Ok(line) = line else { return };
            let done = line.contains("server started");
            tx.send(line).ok();
            if done {
                return;
            }
        }
    });

    let port: u16 = loop {
        let line = rx
            .recv_timeout(Duration::from_secs(30))
            .expect("timeout waiting for server to start");
        if line.contains("server started") {
            let addr: SocketAddr = line
                .split("port=")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse().ok())
                .expect("failed to parse bound address from server log");
            break addr.port();
        }
    };

    {
        use std::io::{Read as _, Write as _};
        use std::net::TcpStream;
        let mut stream =
            TcpStream::connect(format!("127.0.0.1:{port}")).expect("TCP connect failed");
        write!(
            stream,
            "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
        )
        .expect("TCP write failed");
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("TCP read failed");
        assert!(
            response.starts_with("HTTP/1.1 200"),
            "unexpected response: {response}"
        );
    }

    Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .expect("failed to send SIGINT");

    let status = child.wait().expect("failed to wait for child");
    assert!(status.success(), "brust-web exited with {status}");
}
