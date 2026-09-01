//! Top-level classifier orchestrating parsing, matching, and classification.

use std::{error::Error, fmt};

use crate::database::{load_database, CommandDatabase, DbError};
use crate::matcher::{match_command, CommandLookup, MatchError};
use crate::models::{Classification, CommandResult, ExpressionResult, InnerCommandResult, Risk};
use crate::parser::{parse_expression, ParseError};

const SYSTEM_DIRS: &[&str] = &[
    "/etc", "/lib", "/lib64", "/usr", "/bin", "/sbin", "/boot", "/sys", "/proc", "/run", "/srv",
    "/root", "/opt", "/var", "/dev",
];
const SAFE_PREFIXES: &[&str] = &[
    "/tmp",
    "/var/tmp",
    "/home",
    "/dev/null",
    "/dev/stdin",
    "/dev/stdout",
    "/dev/stderr",
    "/dev/fd",
    "/dev/tcp",
    "/dev/udp",
];
const WRITE_OPERATORS: &[&str] = &[">", ">>", "2>", "&>", ">&"];
const READ_OPERATORS: &[&str] = &["<"];
const DEV_PATHS: &[&str] = &[
    "/dev/null",
    "/dev/stdin",
    "/dev/stdout",
    "/dev/stderr",
    "/dev/fd/",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifyError(String);

impl ClassifyError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ClassifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for ClassifyError {}

impl From<DbError> for ClassifyError {
    fn from(error: DbError) -> Self {
        Self(error.to_string())
    }
}

impl From<ParseError> for ClassifyError {
    fn from(error: ParseError) -> Self {
        Self(error.to_string())
    }
}

impl From<MatchError> for ClassifyError {
    fn from(error: MatchError) -> Self {
        Self(error.to_string())
    }
}

pub fn is_temp_path(path: &str) -> bool {
    path.starts_with("/tmp/")
        || path == "/tmp"
        || path.starts_with("/var/tmp/")
        || path == "/var/tmp"
}

pub fn is_real_file_path(path: &str) -> bool {
    DEV_PATHS
        .iter()
        .all(|dev| !(path == *dev || path.starts_with(dev)))
}

pub fn is_system_path(path: &str) -> bool {
    if !path.starts_with('/') {
        return false;
    }
    for safe in SAFE_PREFIXES {
        if path == *safe || path.starts_with(&format!("{safe}/")) {
            return false;
        }
    }
    SYSTEM_DIRS
        .iter()
        .any(|sysdir| path == *sysdir || path.starts_with(&format!("{sysdir}/")))
}

pub fn py_dirname(path: &str) -> String {
    if path.is_empty() || !path.contains('/') {
        return String::new();
    }
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return "/".to_string();
    }
    match trimmed.rfind('/') {
        Some(0) => "/".to_string(),
        Some(index) => trimmed[..index].to_string(),
        None => String::new(),
    }
}

pub fn classify_expression(
    expression: &str,
    database: Option<&CommandDatabase>,
) -> Result<ExpressionResult, ClassifyError> {
    match database {
        Some(database) => classify_expression_with_database(expression, database),
        None => {
            let database = load_database(None)?;
            classify_expression_with_database(expression, &database)
        }
    }
}

pub fn classify_expression_with_database(
    expression: &str,
    database: &impl CommandLookup,
) -> Result<ExpressionResult, ClassifyError> {
    let (invocations, parse_warnings) = parse_expression(expression)?;
    let mut command_results = Vec::new();
    let mut all_redirects = Vec::new();

    for invocation in invocations {
        let mut result = if invocation
            .argv
            .first()
            .is_some_and(|arg| arg.starts_with('$'))
        {
            CommandResult {
                command: vec![invocation.argv[0].clone()],
                argv: invocation.argv.clone(),
                classification: Classification::Dangerous,
                risk: Risk::High,
                matched_rule: None,
                inner_commands: vec![],
                ignored_options: None,
                remaining_options: None,
                classification_reason: Some("variable expansion in command position".to_string()),
                overriding_option: None,
                directories: None,
                write_paths: None,
                read_paths: None,
            }
        } else {
            match_command(&invocation, database)?
        };

        let mut write_paths = Vec::new();
        let mut read_paths = Vec::new();
        let mut all_write_targets_are_temp = true;

        for redirect in &invocation.redirects {
            if WRITE_OPERATORS.contains(&redirect.operator.as_str())
                && is_real_file_path(&redirect.target)
            {
                write_paths.push(redirect.target.clone());
                if !is_temp_path(&redirect.target) {
                    all_write_targets_are_temp = false;
                }
            } else if READ_OPERATORS.contains(&redirect.operator.as_str())
                && is_real_file_path(&redirect.target)
            {
                read_paths.push(redirect.target.clone());
            }
        }

        for redirect in &invocation.redirects {
            if redirect.affects_classification {
                let elevated = Classification::max_severity([
                    result.classification,
                    Classification::LocalEffects,
                ]);
                if elevated != result.classification {
                    result.classification = elevated;
                    result.classification_reason =
                        Some(match result.classification_reason.take() {
                            Some(reason) => format!("{reason}; elevated by output redirect"),
                            None => "elevated by output redirect".to_string(),
                        });
                }
                if !(!write_paths.is_empty() && all_write_targets_are_temp) {
                    result.risk = Risk::max_severity([result.risk, Risk::Medium]);
                }
            }
            if redirect.target.starts_with("/dev/tcp/") || redirect.target.starts_with("/dev/udp/")
            {
                let elevated = Classification::max_severity([
                    result.classification,
                    Classification::Dangerous,
                ]);
                if elevated != result.classification {
                    result.classification = elevated;
                    result.classification_reason = Some(
                        "elevated to DANGEROUS: /dev/tcp or /dev/udp access detected".to_string(),
                    );
                }
                result.risk = Risk::High;
            }
        }

        for arg in &invocation.argv {
            if arg.starts_with("/dev/tcp/") || arg.starts_with("/dev/udp/") {
                let elevated = Classification::max_severity([
                    result.classification,
                    Classification::Dangerous,
                ]);
                if elevated != result.classification {
                    result.classification = elevated;
                    result.classification_reason = Some(
                        "elevated to DANGEROUS: /dev/tcp or /dev/udp access detected".to_string(),
                    );
                }
                result.risk = Risk::High;
            }
        }

        if invocation.is_background {
            let elevated =
                Classification::max_severity([result.classification, Classification::LocalEffects]);
            if elevated != result.classification {
                result.classification = elevated;
                result.classification_reason = Some(match result.classification_reason.take() {
                    Some(reason) => format!("{reason}; elevated by backgrounding"),
                    None => "elevated by backgrounding".to_string(),
                });
            }
            result.risk = Risk::max_severity([result.risk, Risk::Medium]);
        }

        if result.classification.severity() >= Classification::LocalEffects.severity() {
            let mut system_paths_found = Vec::new();
            for token in &invocation.argv {
                if is_system_path(token) {
                    system_paths_found.push(token.clone());
                }
            }
            for redirect in &invocation.redirects {
                if is_system_path(&redirect.target) {
                    system_paths_found.push(redirect.target.clone());
                }
            }
            if !system_paths_found.is_empty() && result.classification != Classification::Dangerous
            {
                result.classification = Classification::Dangerous;
                let prefix = result.classification_reason.clone().unwrap_or_default();
                result.classification_reason = Some(format!(
                    "{prefix}; elevated to DANGEROUS: system path {}",
                    system_paths_found[0]
                ));
                result.risk = Risk::High;
            }
        }

        result.write_paths = if write_paths.is_empty() {
            None
        } else {
            Some(write_paths)
        };
        result.read_paths = if read_paths.is_empty() {
            None
        } else {
            Some(read_paths)
        };

        command_results.push(result);
        all_redirects.extend(invocation.redirects);
    }

    let directories = collect_directories(&command_results);

    let mut all_write_paths = Vec::new();
    let mut all_read_paths = Vec::new();
    for cmd_result in &command_results {
        if let Some(write_paths) = &cmd_result.write_paths {
            all_write_paths.extend(write_paths.clone());
        }
        if let Some(read_paths) = &cmd_result.read_paths {
            all_read_paths.extend(read_paths.clone());
        }
    }

    let (overall, overall_risk) = if command_results.is_empty() && !parse_warnings.is_empty() {
        (Classification::Unknown, Risk::High)
    } else if command_results.is_empty() {
        (Classification::Readonly, Risk::Low)
    } else {
        (
            Classification::max_severity(command_results.iter().map(|r| r.classification)),
            Risk::max_severity(command_results.iter().map(|r| r.risk)),
        )
    };

    Ok(ExpressionResult {
        expression: expression.to_string(),
        classification: overall,
        risk: overall_risk,
        directories,
        write_paths: all_write_paths,
        read_paths: all_read_paths,
        commands: command_results,
        redirects: all_redirects,
        parse_warnings,
    })
}

pub fn extract_directories_from_argv(command: &[String], argv: &[String]) -> Vec<String> {
    if command.is_empty() || argv.len() <= 1 {
        return vec![];
    }

    let binary = command[0].as_str();
    if binary == "find" || binary == "ls" {
        for arg in &argv[1..] {
            if !arg.starts_with('-') {
                return vec![arg.clone()];
            }
        }
        return vec![];
    }

    if ["cat", "head", "tail", "less", "more"].contains(&binary) {
        for arg in &argv[1..] {
            if !arg.starts_with('-') && arg.contains('/') {
                let dirname = py_dirname(arg);
                if !dirname.is_empty() {
                    return vec![dirname];
                }
                return vec![];
            }
        }
    }

    vec![]
}

pub fn collect_directories(results: &[CommandResult]) -> Vec<String> {
    let mut directories = Vec::new();
    for result in results {
        if result
            .command
            .first()
            .is_some_and(|cmd| cmd == "cd" || cmd == "pushd")
            && result.argv.len() > 1
        {
            directories.push(result.argv[1].clone());
        }
        if let Some(captured) = &result.directories {
            directories.extend(captured.clone());
        }
        directories.extend(extract_directories_from_argv(&result.command, &result.argv));
        directories.extend(collect_inner_directories(&result.inner_commands));
    }
    directories
}

pub fn collect_inner_directories(results: &[InnerCommandResult]) -> Vec<String> {
    let mut directories = Vec::new();
    for result in results {
        directories.extend(extract_directories_from_argv(&result.command, &result.argv));
        directories.extend(collect_inner_directories(&result.inner_commands));
    }
    directories
}
