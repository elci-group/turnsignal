use std::process::{Command, Stdio};

#[test]
fn cli_emits_canonical_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_turnsignal"))
        .args([
            "--json",
            "--mode",
            "routing",
            "Compliance tests are missing",
        ])
        .output()
        .expect("turnsignal should execute");

    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(value["mode"], "routing");
    assert_eq!(
        value["signal"]["next"]["action"],
        "inspect_test_infrastructure"
    );
    assert_eq!(value["signal"]["route"]["target"], "isopod");
}

#[test]
fn cli_reads_agent_output_from_stdin() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_turnsignal"))
        .args(["--mode", "injective"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("turnsignal should start");
    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"Implementation complete")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("[TURNSIGNAL]"));
}
