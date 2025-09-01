#[test]
fn test_cli_with_custom_name() {
    use assert_cmd::Command;
    use predicates::prelude::predicate;
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.arg("--name")
        .arg("Alice")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Alice, new world!!"));
}

#[test]
fn test_cli_with_short_flag() {
    use assert_cmd::Command;
    use predicates::prelude::predicate;
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.arg("-n")
        .arg("Bob")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Bob, new world!!"));
}

#[test]
fn test_cli_version_flag() {
    use assert_cmd::Command;
    use predicates::prelude::predicate;
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("brust version"));
}

#[test]
fn test_cli_version_short_flag() {
    use assert_cmd::Command;
    use predicates::prelude::predicate;
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("brust version"));
}
