use assert_cmd::Command;
use predicates::prelude::*;

fn mock_cmd() -> Command {
    let mut c = Command::cargo_bin("scale-bridge").unwrap();
    c.env("SCALE_BRIDGE_MOCK", "1");
    c
}

// --- weight subcommand ---

#[test]
fn weight_text_exits_zero_and_prints_value() {
    mock_cmd()
        .args(["weight"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1234.56"))
        .stdout(predicate::str::contains("lb"));
}

#[test]
fn weight_json_output_is_valid_json_with_weight_key() {
    let output = mock_cmd()
        .args(["weight", "--output", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(output).unwrap();
    let v: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON");
    assert!(
        v.get("Weight").is_some(),
        "JSON should have 'Weight' key, got: {s}"
    );
}

#[test]
fn weight_csv_output_contains_timestamp_value_unit() {
    mock_cmd()
        .args(["weight", "--output", "csv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1234.56"))
        .stdout(predicate::str::contains("lb"))
        .stdout(predicate::str::contains("stable"));
}

// --- control commands ---

#[test]
fn zero_command_exits_zero_and_prints_ok() {
    mock_cmd()
        .args(["zero"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));
}

#[test]
fn tare_command_exits_zero_and_prints_ok() {
    mock_cmd()
        .args(["tare"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));
}

// --- error cases ---

#[test]
fn missing_port_and_host_exits_with_code_2() {
    Command::cargo_bin("scale-bridge")
        .unwrap()
        .args(["weight"])
        .assert()
        .failure()
        .code(2);
}

// --- help ---

#[test]
fn help_exits_zero() {
    Command::cargo_bin("scale-bridge")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn weight_subcommand_help_exits_zero() {
    Command::cargo_bin("scale-bridge")
        .unwrap()
        .args(["weight", "--help"])
        .assert()
        .success();
}

// --- systemd flag ---

#[test]
fn systemd_flag_is_accepted() {
    mock_cmd().args(["--systemd", "weight"]).assert().success();
}

#[test]
fn parity_flag_is_accepted() {
    mock_cmd()
        .args(["--serial-port", "/dev/ttyUSB0", "--parity", "odd", "weight"])
        .assert()
        .success();
}

#[test]
fn legacy_port_alias_is_accepted() {
    mock_cmd()
        .args(["--port", "/dev/ttyUSB0", "weight"])
        .assert()
        .success();
}

#[test]
fn global_serial_port_flag_is_accepted_after_subcommand() {
    mock_cmd()
        .args(["weight", "--serial-port", "/dev/ttyUSB0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1234.56"));
}

#[test]
fn verbose_level_emits_raw_frame_logs() {
    mock_cmd()
        .args(["--verbose", "1", "weight"])
        .assert()
        .success()
        .stderr(predicate::str::contains("tx: 57 0D"))
        .stderr(predicate::str::contains(
            "rx: 0A 20 20 31 32 33 34 2E 35 36 6C 62 0D 0A B0 B0 0D 03",
        ));
}
