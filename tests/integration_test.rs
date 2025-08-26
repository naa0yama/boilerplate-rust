use assert_cmd::Command;

#[test]
fn test_cli_with_default_name() {
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.assert().success().stdout("Hi, Youre, new world!!\n");
}

#[test]
fn test_cli_with_custom_name() {
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.arg("--name")
        .arg("Alice")
        .assert()
        .success()
        .stdout("Hi, Alice, new world!!\n");
}

#[test]
fn test_cli_with_short_flag() {
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.arg("-n")
        .arg("Bob")
        .assert()
        .success()
        .stdout("Hi, Bob, new world!!\n");
}

#[test]
fn test_cli_with_japanese_name() {
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.arg("--name")
        .arg("世界")
        .assert()
        .success()
        .stdout("Hi, 世界, new world!!\n");
}

#[test]
fn test_cli_with_empty_name() {
    let mut cmd = Command::cargo_bin("brust").unwrap();
    cmd.arg("--name")
        .arg("")
        .assert()
        .success()
        .stdout("Hi, , new world!!\n");
}

#[test]
fn test_cli_version_flag() {
    let mut cmd = Command::cargo_bin("brust").unwrap();
    let output = cmd.arg("--version").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("brust version"));
}

#[test]
fn test_cli_version_short_flag() {
    let mut cmd = Command::cargo_bin("brust").unwrap();
    let output = cmd.arg("-V").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("brust version"));
}
