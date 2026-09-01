//! CLI entry point and JSON serialization helpers.

use std::io::{self, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use serde_json::ser::{CharEscape, Formatter, PrettyFormatter, Serializer};
use serde_json::{Map, Value};

use crate::classifier::classify_expression;
use crate::models::{CommandResult, ExpressionResult, InnerCommandResult, Redirect};

const PROGRAM_NAME: &str = "rust-bash-classify";
const PYTHON_PROGRAM_NAME: &str = "bash-classify";
const STDIN_TIMEOUT: Duration = Duration::from_secs(5);

pub fn inner_command_to_value(result: &InnerCommandResult) -> Value {
    let mut d = Map::new();
    d.insert(
        "delegation_mode".to_string(),
        Value::String(result.delegation_mode.clone()),
    );
    d.insert(
        "delegation_source".to_string(),
        Value::String(result.delegation_source.clone()),
    );
    d.insert("command".to_string(), strings_value(&result.command));
    d.insert("argv".to_string(), strings_value(&result.argv));
    d.insert(
        "classification".to_string(),
        Value::String(result.classification.to_string()),
    );
    d.insert("risk".to_string(), Value::String(result.risk.to_string()));
    d.insert(
        "matched_rule".to_string(),
        option_string_value(&result.matched_rule),
    );

    if let Some(ignored_options) = non_empty(&result.ignored_options) {
        d.insert(
            "ignored_options".to_string(),
            strings_value(ignored_options),
        );
    }
    if let Some(remaining_options) = non_empty(&result.remaining_options) {
        d.insert(
            "remaining_options".to_string(),
            strings_value(remaining_options),
        );
    }
    if let Some(overriding_option) = &result.overriding_option {
        d.insert(
            "overriding_option".to_string(),
            Value::String(overriding_option.clone()),
        );
    }

    d.insert(
        "inner_commands".to_string(),
        Value::Array(
            result
                .inner_commands
                .iter()
                .map(inner_command_to_value)
                .collect(),
        ),
    );

    Value::Object(d)
}

pub fn command_to_value(result: &CommandResult) -> Value {
    let mut d = Map::new();
    d.insert("command".to_string(), strings_value(&result.command));
    d.insert("argv".to_string(), strings_value(&result.argv));
    d.insert(
        "classification".to_string(),
        Value::String(result.classification.to_string()),
    );
    d.insert("risk".to_string(), Value::String(result.risk.to_string()));
    d.insert(
        "matched_rule".to_string(),
        option_string_value(&result.matched_rule),
    );

    if let Some(ignored_options) = non_empty(&result.ignored_options) {
        d.insert(
            "ignored_options".to_string(),
            strings_value(ignored_options),
        );
    }
    if let Some(remaining_options) = non_empty(&result.remaining_options) {
        d.insert(
            "remaining_options".to_string(),
            strings_value(remaining_options),
        );
    }
    if let Some(classification_reason) = &result.classification_reason {
        d.insert(
            "classification_reason".to_string(),
            Value::String(classification_reason.clone()),
        );
    }
    if let Some(overriding_option) = &result.overriding_option {
        d.insert(
            "overriding_option".to_string(),
            Value::String(overriding_option.clone()),
        );
    }

    if let Some(write_paths) = non_empty(&result.write_paths) {
        d.insert("write_paths".to_string(), strings_value(write_paths));
    }
    if let Some(read_paths) = non_empty(&result.read_paths) {
        d.insert("read_paths".to_string(), strings_value(read_paths));
    }

    d.insert(
        "inner_commands".to_string(),
        Value::Array(
            result
                .inner_commands
                .iter()
                .map(inner_command_to_value)
                .collect(),
        ),
    );

    Value::Object(d)
}

pub fn redirect_to_value(redirect: &Redirect) -> Value {
    let mut d = Map::new();
    d.insert(
        "operator".to_string(),
        Value::String(redirect.operator.clone()),
    );
    d.insert("target".to_string(), Value::String(redirect.target.clone()));
    d.insert(
        "affects_classification".to_string(),
        Value::Bool(redirect.affects_classification),
    );
    Value::Object(d)
}

pub fn result_to_value(result: &ExpressionResult) -> Value {
    let mut d = Map::new();
    d.insert(
        "expression".to_string(),
        Value::String(result.expression.clone()),
    );
    d.insert(
        "classification".to_string(),
        Value::String(result.classification.to_string()),
    );
    d.insert("risk".to_string(), Value::String(result.risk.to_string()));
    d.insert(
        "directories".to_string(),
        strings_value(&result.directories),
    );
    d.insert(
        "commands".to_string(),
        Value::Array(result.commands.iter().map(command_to_value).collect()),
    );

    if !result.write_paths.is_empty() {
        d.insert(
            "write_paths".to_string(),
            strings_value(&result.write_paths),
        );
    }
    if !result.read_paths.is_empty() {
        d.insert("read_paths".to_string(), strings_value(&result.read_paths));
    }
    if !result.redirects.is_empty() {
        d.insert(
            "redirects".to_string(),
            Value::Array(result.redirects.iter().map(redirect_to_value).collect()),
        );
    }
    if !result.parse_warnings.is_empty() {
        d.insert(
            "parse_warnings".to_string(),
            strings_value(&result.parse_warnings),
        );
    }

    Value::Object(d)
}

fn strings_value(strings: &[String]) -> Value {
    Value::Array(strings.iter().cloned().map(Value::String).collect())
}

fn option_string_value(value: &Option<String>) -> Value {
    value
        .as_ref()
        .map_or(Value::Null, |value| Value::String(value.clone()))
}

fn non_empty(value: &Option<Vec<String>>) -> Option<&[String]> {
    value.as_deref().filter(|items| !items.is_empty())
}

#[derive(Clone, Debug)]
pub struct PyAsciiFormatter<'a> {
    inner: PrettyFormatter<'a>,
}

impl<'a> PyAsciiFormatter<'a> {
    pub fn new() -> Self {
        Self {
            inner: PrettyFormatter::with_indent(b"  "),
        }
    }
}

impl Default for PyAsciiFormatter<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter for PyAsciiFormatter<'_> {
    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let mut buf = [0_u16; 2];
        for ch in fragment.chars() {
            if ch == '\u{007f}' {
                writer.write_all(b"\\u007f")?;
            } else if ch.is_ascii() {
                writer.write_all(&[ch as u8])?;
            } else {
                for unit in ch.encode_utf16(&mut buf) {
                    write!(writer, "\\u{unit:04x}")?;
                }
            }
        }
        Ok(())
    }

    fn write_char_escape<W>(&mut self, writer: &mut W, char_escape: CharEscape) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.write_char_escape(writer, char_escape)
    }

    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.begin_array(writer)
    }

    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.end_array(writer)
    }

    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.begin_array_value(writer, first)
    }

    fn end_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.end_array_value(writer)
    }

    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.begin_object(writer)
    }

    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.end_object(writer)
    }

    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.begin_object_key(writer, first)
    }

    fn end_object_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.end_object_key(writer)
    }

    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.begin_object_value(writer)
    }

    fn end_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.end_object_value(writer)
    }
}

pub fn to_py_ascii_pretty_json<T: ?Sized + Serialize>(value: &T) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let formatter = PyAsciiFormatter::new();
    let mut serializer = Serializer::with_formatter(&mut output, formatter);
    value.serialize(&mut serializer).map_err(io::Error::other)?;
    Ok(output)
}

pub fn py_strip(value: &str) -> &str {
    value.trim_matches(is_python_whitespace)
}

fn is_python_whitespace(ch: char) -> bool {
    matches!(
        ch,
        '\u{0009}'
            | '\u{000a}'
            | '\u{000b}'
            | '\u{000c}'
            | '\u{000d}'
            | '\u{001c}'
            | '\u{001d}'
            | '\u{001e}'
            | '\u{001f}'
            | '\u{0020}'
            | '\u{0085}'
            | '\u{00a0}'
            | '\u{1680}'
            | '\u{2000}'
            | '\u{2001}'
            | '\u{2002}'
            | '\u{2003}'
            | '\u{2004}'
            | '\u{2005}'
            | '\u{2006}'
            | '\u{2007}'
            | '\u{2008}'
            | '\u{2009}'
            | '\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
    )
}

pub fn run() -> i32 {
    let mut args = std::env::args();
    let _program = args.next();
    if let Some(first_arg) = args.next() {
        if first_arg == "-h" || first_arg == "--help" {
            print_help();
            return 0;
        }
        if first_arg == "-v" || first_arg == "--version" {
            println!("{PROGRAM_NAME} {}", env!("CARGO_PKG_VERSION"));
            return 0;
        }
    }

    match run_classification() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{PROGRAM_NAME}: internal error: {error}");
            2
        }
    }
}

fn run_classification() -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut input = String::new();
        let result = io::stdin().read_to_string(&mut input).map(|_| input);
        let _ = sender.send(result);
    });

    let input = match receiver.recv_timeout(STDIN_TIMEOUT) {
        Ok(result) => result?,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            eprintln!(
                "{PROGRAM_NAME}: timed out waiting for input on stdin (use --help for usage)"
            );
            return Ok(1);
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => return Err("stdin reader disconnected".into()),
    };

    let expression = py_strip(&input);
    if expression.is_empty() {
        return Ok(1);
    }

    let result = classify_expression(expression, None)?;
    let output = result_to_value(&result);
    let mut bytes = to_py_ascii_pretty_json(&output)?;
    bytes.push(b'\n');
    io::stdout().write_all(&bytes)?;
    Ok(0)
}

fn print_help() {
    // OQ-2 was resolved for this task as honest Rust binary naming. Keep program-name-bearing
    // strings factored through constants so reverting to Python naming is a one-line switch.
    println!("Usage: {PROGRAM_NAME} < command");
    println!("       echo 'ls -la' | {PROGRAM_NAME}");
    println!();
    println!("Reads a bash expression from stdin, classifies it, and outputs JSON.");
    println!();
    println!("Exit codes:");
    println!("  0  successfully classified");
    println!("  1  empty input");
    println!("  2  internal error");
}

#[allow(dead_code)]
fn python_program_name() -> &'static str {
    PYTHON_PROGRAM_NAME
}

#[cfg(test)]
mod test_cli_serialization {
    use super::*;
    use crate::models::{Classification, Risk};

    fn command_result(command: &[&str]) -> CommandResult {
        CommandResult {
            command: command.iter().map(|s| (*s).to_string()).collect(),
            argv: command.iter().map(|s| (*s).to_string()).collect(),
            classification: Classification::Readonly,
            risk: Risk::Low,
            matched_rule: Some(command[0].to_string()),
            inner_commands: vec![],
            ignored_options: None,
            remaining_options: None,
            classification_reason: None,
            overriding_option: None,
            directories: None,
            write_paths: None,
            read_paths: None,
        }
    }

    fn inner_result(command: &[&str]) -> InnerCommandResult {
        InnerCommandResult {
            delegation_mode: "rest_are_argv".to_string(),
            delegation_source: "sudo".to_string(),
            command: command.iter().map(|s| (*s).to_string()).collect(),
            argv: command.iter().map(|s| (*s).to_string()).collect(),
            classification: Classification::Readonly,
            risk: Risk::Low,
            matched_rule: Some(command[0].to_string()),
            inner_commands: vec![],
            ignored_options: None,
            remaining_options: None,
            overriding_option: None,
        }
    }

    #[test]
    fn test_redirect_affects_classification() {
        let redirect = Redirect {
            operator: ">".to_string(),
            target: "output.txt".to_string(),
            affects_classification: true,
        };
        let d = redirect_to_value(&redirect);
        assert_eq!(d["operator"], ">");
        assert_eq!(d["target"], "output.txt");
        assert_eq!(d["affects_classification"], true);
        assert_eq!(keys(&d), ["operator", "target", "affects_classification"]);
    }

    #[test]
    fn test_redirect_does_not_affect_classification() {
        let redirect = Redirect {
            operator: ">".to_string(),
            target: "/dev/null".to_string(),
            affects_classification: false,
        };
        let d = redirect_to_value(&redirect);
        assert_eq!(d["operator"], ">");
        assert_eq!(d["target"], "/dev/null");
        assert_eq!(d["affects_classification"], false);
    }

    #[test]
    fn test_all_optional_fields_populated() {
        let mut inner = inner_result(&["ls", "-la"]);
        inner.command = vec!["ls".to_string()];
        let mut result = command_result(&["git", "push", "--force"]);
        result.command = vec!["git".to_string(), "push".to_string()];
        result.classification = Classification::Dangerous;
        result.risk = Risk::High;
        result.matched_rule = Some("git.push".to_string());
        result.inner_commands = vec![inner];
        result.ignored_options = Some(vec!["--context=prod".to_string()]);
        result.remaining_options = Some(vec!["--unknown".to_string()]);
        result.classification_reason =
            Some("overridden by option --force to DANGEROUS".to_string());
        result.overriding_option = Some("--force".to_string());

        let d = command_to_value(&result);
        assert_eq!(d["command"], serde_json::json!(["git", "push"]));
        assert_eq!(d["argv"], serde_json::json!(["git", "push", "--force"]));
        assert_eq!(d["classification"], "DANGEROUS");
        assert_eq!(d["risk"], "HIGH");
        assert_eq!(d["matched_rule"], "git.push");
        assert_eq!(d["ignored_options"], serde_json::json!(["--context=prod"]));
        assert_eq!(d["remaining_options"], serde_json::json!(["--unknown"]));
        assert_eq!(
            d["classification_reason"],
            "overridden by option --force to DANGEROUS"
        );
        assert_eq!(d["overriding_option"], "--force");
        assert_eq!(d["inner_commands"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_all_optional_fields_empty_omitted() {
        let result = command_result(&["ls"]);
        let d = command_to_value(&result);
        assert_absent(&d, "ignored_options");
        assert_absent(&d, "remaining_options");
        assert_absent(&d, "classification_reason");
        assert_absent(&d, "overriding_option");
        assert_eq!(d["inner_commands"], serde_json::json!([]));
    }

    #[test]
    fn test_empty_optional_lists_omitted() {
        let mut result = command_result(&["ls"]);
        result.ignored_options = Some(vec![]);
        result.remaining_options = Some(vec![]);
        result.write_paths = Some(vec![]);
        result.read_paths = Some(vec![]);
        let d = command_to_value(&result);
        assert_absent(&d, "ignored_options");
        assert_absent(&d, "remaining_options");
        assert_absent(&d, "write_paths");
        assert_absent(&d, "read_paths");
    }

    #[test]
    fn test_with_delegation_mode_and_source() {
        let mut inner = inner_result(&["rm", "-rf", "/"]);
        inner.command = vec!["rm".to_string()];
        inner.classification = Classification::Dangerous;
        inner.risk = Risk::High;
        inner.matched_rule = Some("rm".to_string());
        let d = inner_command_to_value(&inner);
        assert_eq!(d["delegation_mode"], "rest_are_argv");
        assert_eq!(d["delegation_source"], "sudo");
        assert_eq!(d["command"], serde_json::json!(["rm"]));
        assert_eq!(d["argv"], serde_json::json!(["rm", "-rf", "/"]));
        assert_eq!(d["classification"], "DANGEROUS");
        assert_eq!(d["risk"], "HIGH");
        assert_eq!(d["matched_rule"], "rm");
        assert_eq!(d["inner_commands"], serde_json::json!([]));
    }

    #[test]
    fn test_inner_optional_fields() {
        let mut inner = inner_result(&["x"]);
        inner.matched_rule = None;
        inner.ignored_options = Some(vec!["--ignored".to_string()]);
        inner.remaining_options = Some(vec!["--remaining".to_string()]);
        inner.overriding_option = Some("--danger".to_string());
        let d = inner_command_to_value(&inner);
        assert_eq!(d["matched_rule"], Value::Null);
        assert_eq!(d["ignored_options"], serde_json::json!(["--ignored"]));
        assert_eq!(d["remaining_options"], serde_json::json!(["--remaining"]));
        assert_eq!(d["overriding_option"], "--danger");
    }

    #[test]
    fn test_with_redirects_and_parse_warnings() {
        let redirect = Redirect {
            operator: ">".to_string(),
            target: "out.txt".to_string(),
            affects_classification: true,
        };
        let mut cmd = command_result(&["echo", "hello"]);
        cmd.command = vec!["echo".to_string()];
        cmd.classification = Classification::ExternalEffects;
        cmd.risk = Risk::Medium;
        cmd.classification_reason = Some("elevated by output redirect".to_string());
        let result = ExpressionResult {
            expression: "echo hello > out.txt".to_string(),
            classification: Classification::ExternalEffects,
            risk: Risk::Medium,
            directories: vec![],
            write_paths: vec![],
            read_paths: vec![],
            commands: vec![cmd],
            redirects: vec![redirect],
            parse_warnings: vec!["some warning".to_string()],
        };
        let d = result_to_value(&result);
        assert_eq!(d["expression"], "echo hello > out.txt");
        assert_eq!(d["classification"], "EXTERNAL_EFFECTS");
        assert!(d.get("redirects").is_some());
        assert_eq!(d["redirects"].as_array().unwrap().len(), 1);
        assert_eq!(d["parse_warnings"], serde_json::json!(["some warning"]));
    }

    #[test]
    fn test_empty_redirects_and_warnings_omitted() {
        let result = ExpressionResult {
            expression: "ls".to_string(),
            classification: Classification::Readonly,
            risk: Risk::Low,
            directories: vec![],
            write_paths: vec![],
            read_paths: vec![],
            commands: vec![command_result(&["ls"])],
            redirects: vec![],
            parse_warnings: vec![],
        };
        let d = result_to_value(&result);
        assert_absent(&d, "redirects");
        assert_absent(&d, "parse_warnings");
        assert_eq!(d["directories"], serde_json::json!([]));
    }

    #[test]
    fn test_classification_enum_serializes_as_string() {
        let result = command_result(&["ls"]);
        let d = command_to_value(&result);
        assert_eq!(d["classification"], "READONLY");
        assert!(!d["classification"]
            .as_str()
            .unwrap()
            .contains("Classification."));
    }

    #[test]
    fn test_all_classification_values_serialize_correctly() {
        for cls in [
            Classification::Readonly,
            Classification::LocalEffects,
            Classification::ExternalEffects,
            Classification::Dangerous,
            Classification::Unknown,
        ] {
            let mut result = command_result(&["x"]);
            result.classification = cls;
            result.risk = Risk::High;
            result.matched_rule = None;
            let d = command_to_value(&result);
            assert_eq!(d["classification"], cls.to_string());
            assert!(!d["classification"].as_str().unwrap().contains('.'));
        }
    }

    #[test]
    fn test_risk_field_in_command_dict() {
        let result = command_result(&["ls"]);
        let d = command_to_value(&result);
        assert_eq!(d["risk"], "LOW");
    }

    #[test]
    fn test_risk_field_in_inner_command_dict() {
        let mut inner = inner_result(&["ls"]);
        inner.risk = Risk::Medium;
        let d = inner_command_to_value(&inner);
        assert_eq!(d["risk"], "MEDIUM");
    }

    #[test]
    fn test_risk_field_in_result_dict() {
        let result = ExpressionResult {
            expression: "ls".to_string(),
            classification: Classification::Readonly,
            risk: Risk::Low,
            directories: vec![],
            write_paths: vec![],
            read_paths: vec![],
            commands: vec![command_result(&["ls"])],
            redirects: vec![],
            parse_warnings: vec![],
        };
        let d = result_to_value(&result);
        assert_eq!(d["risk"], "LOW");
    }

    #[test]
    fn test_all_risk_values_serialize_correctly() {
        for risk in [Risk::Low, Risk::Medium, Risk::High] {
            let mut result = command_result(&["x"]);
            result.risk = risk;
            result.matched_rule = None;
            let d = command_to_value(&result);
            assert_eq!(d["risk"], risk.to_string());
        }
    }

    #[test]
    fn test_write_paths_in_command_dict() {
        let mut result = command_result(&["cat"]);
        result.classification = Classification::LocalEffects;
        result.write_paths = Some(vec!["/tmp/foo.txt".to_string()]);
        let d = command_to_value(&result);
        assert_eq!(d["write_paths"], serde_json::json!(["/tmp/foo.txt"]));
        assert_absent(&d, "read_paths");
    }

    #[test]
    fn test_read_paths_in_command_dict() {
        let mut result = command_result(&["cat"]);
        result.read_paths = Some(vec!["/etc/hosts".to_string()]);
        let d = command_to_value(&result);
        assert_eq!(d["read_paths"], serde_json::json!(["/etc/hosts"]));
        assert_absent(&d, "write_paths");
    }

    #[test]
    fn test_expression_write_read_paths() {
        let mut cmd = command_result(&["cat"]);
        cmd.classification = Classification::LocalEffects;
        cmd.write_paths = Some(vec!["/tmp/out.txt".to_string()]);
        cmd.read_paths = Some(vec!["/etc/hosts".to_string()]);
        let result = ExpressionResult {
            expression: "cat < /etc/hosts > /tmp/out.txt".to_string(),
            classification: Classification::LocalEffects,
            risk: Risk::Low,
            directories: vec![],
            write_paths: vec!["/tmp/out.txt".to_string()],
            read_paths: vec!["/etc/hosts".to_string()],
            commands: vec![cmd],
            redirects: vec![],
            parse_warnings: vec![],
        };
        let d = result_to_value(&result);
        assert_eq!(d["write_paths"], serde_json::json!(["/tmp/out.txt"]));
        assert_eq!(d["read_paths"], serde_json::json!(["/etc/hosts"]));
    }

    #[test]
    fn test_expression_paths_omitted_when_empty() {
        let result = ExpressionResult {
            expression: "ls".to_string(),
            classification: Classification::Readonly,
            risk: Risk::Low,
            directories: vec![],
            write_paths: vec![],
            read_paths: vec![],
            commands: vec![command_result(&["ls"])],
            redirects: vec![],
            parse_warnings: vec![],
        };
        let d = result_to_value(&result);
        assert_absent(&d, "write_paths");
        assert_absent(&d, "read_paths");
    }

    #[test]
    fn test_key_order() {
        let result = ExpressionResult {
            expression: "ls".to_string(),
            classification: Classification::Readonly,
            risk: Risk::Low,
            directories: vec![],
            write_paths: vec![],
            read_paths: vec![],
            commands: vec![command_result(&["ls"])],
            redirects: vec![],
            parse_warnings: vec![],
        };
        assert_eq!(
            keys(&result_to_value(&result)),
            [
                "expression",
                "classification",
                "risk",
                "directories",
                "commands"
            ]
        );
        assert_eq!(
            keys(&command_to_value(&result.commands[0])),
            [
                "command",
                "argv",
                "classification",
                "risk",
                "matched_rule",
                "inner_commands"
            ]
        );
    }

    #[test]
    fn test_py_ascii_formatter_matches_python_escape_set() {
        let value = serde_json::json!({
            "s": "nonascii ž DEL \u{007f} astral 😀 controls \t\u{0008}\n\u{000c}\r quote \" backslash \\\\"
        });
        let actual = String::from_utf8(to_py_ascii_pretty_json(&value).unwrap()).unwrap();
        let expected = r#"{
  "s": "nonascii \u017e DEL \u007f astral \ud83d\ude00 controls \t\b\n\f\r quote \" backslash \\\\"
}"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_py_strip_python_whitespace_edges() {
        let whitespace = "\u{0009}\u{000a}\u{000b}\u{000c}\u{000d}\u{001c}\u{001d}\u{001e}\u{001f}\u{0020}\u{0085}\u{00a0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}";
        let input = format!("{whitespace}payload{whitespace}");
        assert_eq!(py_strip(&input), "payload");
        assert_eq!(
            py_strip("\u{001b}payload\u{001b}"),
            "\u{001b}payload\u{001b}"
        );
    }

    fn assert_absent(value: &Value, key: &str) {
        assert!(
            value.as_object().unwrap().get(key).is_none(),
            "{key} was present in {value}"
        );
    }

    fn keys(value: &Value) -> Vec<&str> {
        value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect()
    }
}
