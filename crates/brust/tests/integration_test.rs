#![allow(clippy::unwrap_used)] // テストコードではunwrapを許可
#![allow(missing_docs)] // テストコードではdocコメント不要

use std::net::TcpListener;
use std::time::Duration;

use assert_cmd::cargo_bin_cmd;
use predicates::prelude::{PredicateBooleanExt, predicate};

/// Spawn a minimal HTTP server that accepts connections and returns 200 for any request.
/// Reused for both plain HTTP fetch tests and OTLP receiver stubbing.
fn start_fake_http_server() -> u16 {
    use std::io::{Read as _, Write as _};

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    std::thread::spawn(move || {
        while let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
        }
    });

    port
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_with_custom_name() {
    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("--name")
        .arg("Alice")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Alice, new world!!"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_with_short_flag() {
    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("-n")
        .arg("Bob")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Bob, new world!!"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_version_flag() {
    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("brust"))
        .stdout(predicate::str::contains("(rev:"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_version_short_flag() {
    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("brust"))
        .stdout(predicate::str::contains("(rev:"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_with_gender_man() {
    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("--name")
        .arg("John")
        .arg("--gender")
        .arg("man")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Mr. John, new world!!"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_with_gender_woman() {
    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("--name")
        .arg("Alice")
        .arg("--gender")
        .arg("woman")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Ms. Alice, new world!!"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_with_gender_short_flag() {
    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("-n")
        .arg("Bob")
        .arg("-g")
        .arg("man")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Mr. Bob, new world!!"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_with_invalid_gender() {
    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("--name")
        .arg("Charlie")
        .arg("--gender")
        .arg("other")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Hi, Charlie (invalid gender: other), new world!!",
        ));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_without_gender() {
    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("--name")
        .arg("Dave")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Dave, new world!!"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_count_basic() {
    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("-c")
        .arg("1")
        .timeout(Duration::from_secs(10))
        .assert()
        .success()
        .stdout(predicate::str::contains("starting iteration"))
        .stdout(predicate::str::contains("finished iteration"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_count_zero() {
    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("--count")
        .arg("0")
        .assert()
        .success()
        .stdout(predicate::str::contains("starting iteration").not());
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_count_with_name() {
    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("-n")
        .arg("Alice")
        .arg("-c")
        .arg("1")
        .timeout(Duration::from_secs(10))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Alice, new world!!"))
        .stdout(predicate::str::contains("finished iteration"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_url_fetch_success() {
    let port = start_fake_http_server();

    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("--url")
        .arg(format!("http://127.0.0.1:{port}/"))
        .timeout(Duration::from_secs(15))
        .assert()
        .success();
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_url_fetch_connection_refused() {
    // Bind to get a free port then drop listener → connection refused
    let port = {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };

    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("--url")
        .arg(format!("http://127.0.0.1:{port}/"))
        .timeout(Duration::from_secs(15))
        .assert()
        .success(); // exits 0 even on HTTP error (logs error, continues)
}

#[cfg(feature = "otel")]
#[test]
#[cfg_attr(miri, ignore)]
fn test_cli_otel_init_and_shutdown() {
    let port = start_fake_http_server();

    let mut cmd = cargo_bin_cmd!("brust");
    cmd.arg("--name")
        .arg("OTel")
        .env(
            "OTEL_EXPORTER_OTLP_ENDPOINT",
            format!("http://127.0.0.1:{port}"),
        )
        .timeout(Duration::from_secs(30))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, OTel, new world!!"));
}
