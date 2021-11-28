use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn missing_arguments() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.assert().failure();
}

#[test]
/// `./grant gen` without any args can generate project in current folder
fn gen_without_any_args() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("gen")
        .assert()
        .success()
        // because run test in current folder
        .stderr(predicate::str::contains("Target already exists"));
}

#[test]
fn gen_with_target_args() {
    // Random folder name in /tmp
    let folder_name = format!("/tmp/{}", rand::random::<u64>());

    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("gen")
        .arg("--target")
        .arg(folder_name.clone())
        .assert()
        .success()
        .stderr(predicate::str::contains("Generated"))
        .stderr(predicate::str::contains(folder_name.clone()));
}

#[test]
/// Test gen-pass
fn gen_pass() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("gen-pass")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated"));
}
