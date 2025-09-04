#![allow(clippy::unwrap_used)] // テストコードではunwrapを許可

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

#[test]
fn test_cli_with_gender_man() {
    use assert_cmd::Command;
    use predicates::prelude::predicate;
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.arg("--name")
        .arg("John")
        .arg("--gender")
        .arg("man")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Mr. John, new world!!"));
}

#[test]
fn test_cli_with_gender_woman() {
    use assert_cmd::Command;
    use predicates::prelude::predicate;
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.arg("--name")
        .arg("Alice")
        .arg("--gender")
        .arg("woman")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Ms. Alice, new world!!"));
}

#[test]
fn test_cli_with_gender_short_flag() {
    use assert_cmd::Command;
    use predicates::prelude::predicate;
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.arg("-n")
        .arg("Bob")
        .arg("-g")
        .arg("man")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Mr. Bob, new world!!"));
}

#[test]
fn test_cli_with_invalid_gender() {
    use assert_cmd::Command;
    use predicates::prelude::predicate;
    let mut cmd = Command::cargo_bin("brust").unwrap();
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
fn test_cli_without_gender() {
    use assert_cmd::Command;
    use predicates::prelude::predicate;
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.arg("--name")
        .arg("Dave")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi, Dave, new world!!"));
}
