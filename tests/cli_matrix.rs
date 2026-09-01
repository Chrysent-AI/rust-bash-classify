use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

fn run_with_stdin(expression: &str) -> std::process::Output {
    let mut child = command_with_piped_stdio().spawn().unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(expression.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn command_with_piped_stdio() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rust-bash-classify"));
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

fn temp_config_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "rust-bash-classify-cli-matrix-{name}-{}-{}",
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
fn matrix_success_piped_compound_subshell_redirect_outputs_match_python_fixtures() {
    // Expected stdout generated once from the Python source of truth with:
    //   cd /home/chrysent/devel/bash-classify && printf '%s' "$expr" | uv run bash-classify
    let cases = [
        (
            "ls | grep x",
            r#"{
  "expression": "ls | grep x",
  "classification": "READONLY",
  "risk": "LOW",
  "directories": [],
  "commands": [
    {
      "command": [
        "ls"
      ],
      "argv": [
        "ls"
      ],
      "classification": "READONLY",
      "risk": "LOW",
      "matched_rule": "ls",
      "classification_reason": "base classification from rule ls",
      "inner_commands": []
    },
    {
      "command": [
        "grep"
      ],
      "argv": [
        "grep",
        "x"
      ],
      "classification": "READONLY",
      "risk": "LOW",
      "matched_rule": "grep",
      "classification_reason": "base classification from rule grep",
      "inner_commands": []
    }
  ]
}
"#,
        ),
        (
            "cd /tmp && pwd; echo hi",
            r#"{
  "expression": "cd /tmp && pwd; echo hi",
  "classification": "READONLY",
  "risk": "LOW",
  "directories": [
    "/tmp"
  ],
  "commands": [
    {
      "command": [
        "cd"
      ],
      "argv": [
        "cd",
        "/tmp"
      ],
      "classification": "READONLY",
      "risk": "LOW",
      "matched_rule": null,
      "classification_reason": "shell builtin (always READONLY)",
      "inner_commands": []
    },
    {
      "command": [
        "pwd"
      ],
      "argv": [
        "pwd"
      ],
      "classification": "READONLY",
      "risk": "LOW",
      "matched_rule": "pwd",
      "classification_reason": "base classification from rule pwd",
      "inner_commands": []
    },
    {
      "command": [
        "echo"
      ],
      "argv": [
        "echo",
        "hi"
      ],
      "classification": "READONLY",
      "risk": "LOW",
      "matched_rule": "echo",
      "classification_reason": "base classification from rule echo",
      "inner_commands": []
    }
  ]
}
"#,
        ),
        (
            "(cd /tmp; ls)",
            r#"{
  "expression": "(cd /tmp; ls)",
  "classification": "READONLY",
  "risk": "LOW",
  "directories": [
    "/tmp"
  ],
  "commands": [
    {
      "command": [
        "cd"
      ],
      "argv": [
        "cd",
        "/tmp"
      ],
      "classification": "READONLY",
      "risk": "LOW",
      "matched_rule": null,
      "classification_reason": "shell builtin (always READONLY)",
      "inner_commands": []
    },
    {
      "command": [
        "ls"
      ],
      "argv": [
        "ls"
      ],
      "classification": "READONLY",
      "risk": "LOW",
      "matched_rule": "ls",
      "classification_reason": "base classification from rule ls",
      "inner_commands": []
    }
  ]
}
"#,
        ),
        (
            "echo hi > /tmp/f",
            r#"{
  "expression": "echo hi > /tmp/f",
  "classification": "LOCAL_EFFECTS",
  "risk": "LOW",
  "directories": [],
  "commands": [
    {
      "command": [
        "echo"
      ],
      "argv": [
        "echo",
        "hi"
      ],
      "classification": "LOCAL_EFFECTS",
      "risk": "LOW",
      "matched_rule": "echo",
      "classification_reason": "base classification from rule echo; elevated by output redirect",
      "write_paths": [
        "/tmp/f"
      ],
      "inner_commands": []
    }
  ],
  "write_paths": [
    "/tmp/f"
  ],
  "redirects": [
    {
      "operator": ">",
      "target": "/tmp/f",
      "affects_classification": true
    }
  ]
}
"#,
        ),
    ];

    for (expression, expected_stdout) in cases {
        let output = run_with_stdin(expression);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{expression}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            expected_stdout,
            "{expression}"
        );
        assert!(
            output.stderr.is_empty(),
            "{expression}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn matrix_help_and_version() {
    let help = Command::new(env!("CARGO_BIN_EXE_rust-bash-classify"))
        .arg("--help")
        .output()
        .unwrap();
    assert_eq!(help.status.code(), Some(0));
    assert_eq!(String::from_utf8(help.stdout).unwrap(), "Usage: rust-bash-classify < command\n       echo 'ls -la' | rust-bash-classify\n\nReads a bash expression from stdin, classifies it, and outputs JSON.\n\nExit codes:\n  0  successfully classified\n  1  empty input\n  2  internal error\n");
    assert!(help.stderr.is_empty());

    let version = Command::new(env!("CARGO_BIN_EXE_rust-bash-classify"))
        .arg("--version")
        .output()
        .unwrap();
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "rust-bash-classify 0.1.0\n"
    );
    assert!(version.stderr.is_empty());
}

#[test]
fn matrix_empty_input() {
    let output = run_with_stdin("  \t\n\u{001c}\u{001f}");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn matrix_internal_error_invalid_user_yaml() {
    let root = temp_config_root("internal-error");
    fs::write(root.join("commands/badcmd.yaml"), "rules: [\nnot-yaml\n").unwrap();

    let mut command = command_with_piped_stdio();
    command.env("BASH_CLASSIFY_CONFIG_DIR", &root);
    let mut child = command.spawn().unwrap();
    child.stdin.as_mut().unwrap().write_all(b"badcmd").unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("rust-bash-classify: internal error: Error loading "),
        "{stderr}"
    );
    assert!(stderr.contains("commands/badcmd.yaml"), "{stderr}");
}

#[test]
fn matrix_timeout_without_stdin_data() {
    let child = command_with_piped_stdio().spawn().unwrap();
    std::thread::sleep(Duration::from_millis(5500));
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "rust-bash-classify: timed out waiting for input on stdin (use --help for usage)\n"
    );
}

#[test]
fn matrix_slow_stdin_after_first_byte_waits_for_eof() {
    let mut child = command_with_piped_stdio().spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1000));
        stdin.write_all(b"ls").unwrap();
        std::thread::sleep(Duration::from_millis(7000));
        stdin.write_all(b" -la").unwrap();
    });

    let output = child.wait_with_output().unwrap();
    writer.join().unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"expression\": \"ls -la\""), "{stdout}");
    assert!(
        stdout.contains("\"classification\": \"READONLY\""),
        "{stdout}"
    );
}
