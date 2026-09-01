use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_config_dir(case_name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "rust-bash-classify-golden-{case_name}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir(&dir).unwrap();
    dir
}

fn run_cli(case_name: &str, input: &[u8]) -> std::process::Output {
    let config_dir = temp_config_dir(case_name);
    let mut child = Command::new(env!("CARGO_BIN_EXE_rust-bash-classify"))
        .env("BASH_CLASSIFY_CONFIG_DIR", &config_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("failed to spawn CLI for {case_name}: {err}"));
    child
        .stdin
        .as_mut()
        .unwrap_or_else(|| panic!("stdin unavailable for {case_name}"))
        .write_all(input)
        .unwrap_or_else(|err| panic!("failed to write stdin for {case_name}: {err}"));
    let output = child
        .wait_with_output()
        .unwrap_or_else(|err| panic!("failed to wait for CLI for {case_name}: {err}"));
    let _ = fs::remove_dir(&config_dir);
    output
}

fn expected_exit(manifest: &serde_json::Value, case_name: &str) -> i32 {
    manifest["cases"]
        .as_array()
        .and_then(|cases| {
            cases
                .iter()
                .find(|case| case["name"] == case_name)
                .and_then(|case| case["expected_exit"].as_i64())
        })
        .unwrap_or_else(|| panic!("manifest missing expected_exit for {case_name}")) as i32
}

#[test]
fn golden_corpus_replays_byte_identically() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let golden_dir = root.join("tests/golden");
    let cases_dir = golden_dir.join("cases");
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(golden_dir.join("manifest.json")).expect("manifest.json should exist"),
    )
    .expect("manifest.json should be valid JSON");

    let mut cases = fs::read_dir(&cases_dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", cases_dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    cases.sort();

    assert!(
        !cases.is_empty(),
        "golden corpus must contain at least one case"
    );

    for case_dir in cases {
        let case_name = case_dir
            .file_name()
            .and_then(|name| name.to_str())
            .expect("case directory names should be UTF-8");
        let input = fs::read(case_dir.join("input.txt"))
            .unwrap_or_else(|err| panic!("failed to read input for {case_name}: {err}"));
        let expected_stdout = fs::read(case_dir.join("expected.json"))
            .unwrap_or_else(|err| panic!("failed to read expected stdout for {case_name}: {err}"));
        let expected_exit = expected_exit(&manifest, case_name);

        let output = run_cli(case_name, &input);
        assert_eq!(
            output.status.code(),
            Some(expected_exit),
            "exit code mismatch for {case_name}; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.stdout,
            expected_stdout,
            "stdout mismatch for {case_name}; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
