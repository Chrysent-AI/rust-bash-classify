use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

fn run_with_stdin(expression: &str) -> std::process::Output {
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

#[test]
fn matrix_success_piped_compound_subshell_redirect_outputs_match_python_fixtures() {
    // Expected stdout generated once from the Python source of truth with:
    //   cd ../bash-classify && printf '%s' "$expr" | uv run bash-classify
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
fn matrix_timeout_without_stdin_data() {
    let child = Command::new(env!("CARGO_BIN_EXE_rust-bash-classify"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(5500));
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "rust-bash-classify: timed out waiting for input on stdin (use --help for usage)\n"
    );
}
