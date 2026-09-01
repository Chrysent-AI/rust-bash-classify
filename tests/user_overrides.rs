use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run_cli(expression: &str, config_root: Option<&PathBuf>) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rust-bash-classify"));
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(config_root) = config_root {
        command.env("BASH_CLASSIFY_CONFIG_DIR", config_root);
    } else {
        command.env_remove("BASH_CLASSIFY_CONFIG_DIR");
    }
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(expression.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn temp_config_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "rust-bash-classify-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(root.join("commands")).unwrap();
    root
}

#[test]
fn user_yaml_overrides_builtin_by_filename_stem() {
    let root = temp_config_root("override-ls");
    fs::write(
        root.join("commands/ls.yaml"),
        "command: ls\nclassification: DANGEROUS\nstrict: false\n",
    )
    .unwrap();

    let baseline = run_cli("ls", None);
    assert_eq!(
        baseline.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&baseline.stderr)
    );
    let baseline_json: serde_json::Value = serde_json::from_slice(&baseline.stdout).unwrap();
    assert_eq!(baseline_json["classification"], "READONLY");

    let overridden = run_cli("ls", Some(&root));
    assert_eq!(
        overridden.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&overridden.stderr)
    );
    let overridden_json: serde_json::Value = serde_json::from_slice(&overridden.stdout).unwrap();
    assert_eq!(overridden_json["classification"], "DANGEROUS");
    assert_eq!(overridden_json["commands"][0]["matched_rule"], "ls");
}

#[test]
fn user_yaml_adds_new_command_from_custom_config_root() {
    let root = temp_config_root("new-command");
    fs::write(
        root.join("commands/acme-safe.yaml"),
        "command: acme-safe\nclassification: READONLY\nstrict: false\n",
    )
    .unwrap();

    let baseline = run_cli("acme-safe --flag", None);
    assert_eq!(
        baseline.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&baseline.stderr)
    );
    let baseline_json: serde_json::Value = serde_json::from_slice(&baseline.stdout).unwrap();
    assert_eq!(baseline_json["classification"], "UNKNOWN");
    assert_eq!(
        baseline_json["commands"][0]["matched_rule"],
        serde_json::Value::Null
    );

    let configured = run_cli("acme-safe --flag", Some(&root));
    assert_eq!(
        configured.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&configured.stderr)
    );
    let configured_json: serde_json::Value = serde_json::from_slice(&configured.stdout).unwrap();
    assert_eq!(configured_json["classification"], "READONLY");
    assert_eq!(configured_json["commands"][0]["matched_rule"], "acme-safe");
}
