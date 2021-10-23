use assert_cmd::Command;
use assert_cmd::prelude::*;
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
fn type_cmd() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.arg("type").arg("1");
    cmd.assert().stdout(predicate::eq("number\n")).success();
}


#[test]
fn integration_test() {
    let mut cmd = Command::new("baret");

    cmd.assert().success();
}