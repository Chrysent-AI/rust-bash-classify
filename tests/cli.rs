// Python pytest suite inventory used for the port running total:
// test_classifier.py 114 + test_cli.py 7 + test_cli_serialization.py 18
// + test_database.py 36 + test_matcher.py 134 + test_models.py 13
// + test_parser.py 49 + test_real_world.py 90 + test_schema.py 1
// + test_smoke.py 1 = 463 Python test functions.
use std::io::Write;
use std::process::{Command, Stdio};

fn run_cli(expression: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rust-bash-classify"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(expression.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn json_output(expression: &str) -> serde_json::Value {
    let output = run_cli(expression);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn test_ls_readonly() {
    let output = json_output("ls -la");
    assert_eq!(output["classification"], "READONLY");
    assert!(output.get("commands").is_some());
    assert!(output.get("directories").is_some());
    assert!(output.get("expression").is_some());
}

#[test]
fn test_git_push_force_dangerous() {
    let output = json_output("git push --force");
    assert_eq!(output["classification"], "DANGEROUS");
}

#[test]
fn test_empty_string_exits_with_1() {
    let output = run_cli("");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_json_output_has_required_fields() {
    let output = json_output("echo hello");
    assert!(output.get("expression").is_some());
    assert!(output.get("classification").is_some());
    assert!(output.get("commands").is_some());
    assert!(output.get("directories").is_some());
}

#[test]
fn test_sudo_ls_has_inner_commands() {
    let output = json_output("sudo ls");
    let inner = &output["commands"][0]["inner_commands"][0];
    assert_eq!(inner["delegation_mode"], "rest_are_argv");
    assert_eq!(inner["command"], serde_json::json!(["ls"]));
}

#[test]
fn test_redirect_appears_in_json() {
    let output = json_output("echo hello > out.txt");
    assert!(output["redirects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["operator"] == ">"));
    assert_eq!(output["classification"], "LOCAL_EFFECTS");
}

#[test]
fn test_classification_reason_in_output() {
    let output = json_output("ls -la");
    assert!(output["commands"][0].get("classification_reason").is_some());
}
