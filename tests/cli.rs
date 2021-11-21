use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn init() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.arg("init");
    cmd.assert().stdout(predicate::eq("{}\n")).success();
}

#[test]
fn set() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.arg("set").arg("{}").arg("/test").arg("1");
    cmd.assert()
        .stdout(predicate::eq("{\"test\":1}\n"))
        .success();
}

#[test]
fn get() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.arg("get").arg("{\"test\":1}").arg("/test");
    cmd.assert().stdout(predicate::eq("1\n")).success();
}

#[test]
fn pretty() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.arg("--pretty").arg("get").arg("{\"test\":1}").arg("''");
    cmd.assert()
        .stdout(predicate::eq("{\n  \"test\": 1\n}\n"))
        .success();
}

#[test]
fn escaped() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.arg("--escaped")
        .arg("get")
        .arg("{\"test\":1}")
        .arg("''");
    cmd.assert()
        .stdout(predicate::eq("\"{\\\"test\\\":1}\"\n"))
        .success();
}

#[test]
fn escaped_pretty() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.arg("--pretty")
        .arg("--escaped")
        .arg("get")
        .arg("{\"test\":1}")
        .arg("''");
    cmd.assert()
        .stdout(predicate::eq("\"{\\n  \\\"test\\\": 1\\n}\"\n"))
        .success();
}

#[test]
fn type_cmd() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.arg("type").arg("1");
    cmd.assert().stdout(predicate::eq("number\n")).success();
}

#[test]
fn compare_empty_cmd() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.arg("compare").arg("{}").arg("{}");
    cmd.assert().stdout(predicate::eq("true\n")).success();
}

#[test]
fn compare_cmd_equal_different_order() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.arg("compare").arg(r#"{"testing": 5, "test": 1}"#).arg(r#"{"test": 1, "testing": 5}"#);
    cmd.assert().stdout(predicate::eq("true\n")).success();
}

#[test]
fn compare_cmd_not_equal() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.arg("compare").arg(r#"{"test": 1}"#).arg("{}");
    cmd.assert().stderr(predicate::eq("Error: \"false\"\n")).failure();
}

#[test]
fn integration_test() {
    let mut cmd = Command::new("baret");

    cmd.assert().success();
}
