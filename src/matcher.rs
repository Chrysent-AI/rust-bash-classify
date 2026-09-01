//! Command matching and classification against the database.

use std::collections::HashMap;
use std::{error::Error, fmt};

use crate::database::{CommandDatabase, DbError};
use crate::models::{
    Classification, CommandDef, CommandInvocation, CommandResult, DelegationConfig, DelegationMode,
    InnerCommandResult, Risk, SubcommandMode,
};

const BUILTIN_DIRECTORY_COMMANDS: &[&str] = &["cd", "pushd", "popd"];
const BUILTIN_READONLY_COMMANDS: &[&str] = &["[", "[[", "test"];
const BUILTIN_DANGEROUS_COMMANDS: &[&str] = &["eval", "source", ".", "exec"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchError(String);

impl MatchError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for MatchError {}

impl From<DbError> for MatchError {
    fn from(error: DbError) -> Self {
        Self(error.to_string())
    }
}

impl From<crate::parser::ParseError> for MatchError {
    fn from(error: crate::parser::ParseError) -> Self {
        Self(error.to_string())
    }
}

pub trait CommandLookup {
    fn get_command(&self, key: &str) -> Result<Option<CommandDef>, MatchError>;
    fn resolve_alias(
        &self,
        binary: &str,
        command_def: CommandDef,
    ) -> Result<CommandDef, MatchError>;
}

impl CommandLookup for CommandDatabase {
    fn get_command(&self, key: &str) -> Result<Option<CommandDef>, MatchError> {
        Ok(self.get(key)?)
    }

    fn resolve_alias(
        &self,
        binary: &str,
        command_def: CommandDef,
    ) -> Result<CommandDef, MatchError> {
        Ok(CommandDatabase::resolve_alias(self, binary, command_def)?)
    }
}

impl CommandLookup for HashMap<String, CommandDef> {
    fn get_command(&self, key: &str) -> Result<Option<CommandDef>, MatchError> {
        Ok(self.get(key).cloned())
    }

    fn resolve_alias(
        &self,
        binary: &str,
        command_def: CommandDef,
    ) -> Result<CommandDef, MatchError> {
        resolve_alias_for_test_lookup(self, binary, command_def)
    }
}

impl CommandLookup for std::collections::BTreeMap<String, CommandDef> {
    fn get_command(&self, key: &str) -> Result<Option<CommandDef>, MatchError> {
        Ok(self.get(key).cloned())
    }

    fn resolve_alias(
        &self,
        binary: &str,
        command_def: CommandDef,
    ) -> Result<CommandDef, MatchError> {
        resolve_alias_for_test_lookup(self, binary, command_def)
    }
}

fn resolve_alias_for_test_lookup(
    lookup: &impl CommandLookup,
    binary: &str,
    command_def: CommandDef,
) -> Result<CommandDef, MatchError> {
    let Some(target_name) = command_def.alias_of.clone() else {
        return Ok(command_def);
    };
    let Some(target_def) = lookup.get_command(&target_name)? else {
        return Err(MatchError::new(format!(
            "{binary} aliases '{target_name}' which is not in the database"
        )));
    };
    if let Some(next_target) = &target_def.alias_of {
        return Err(MatchError::new(format!(
            "alias chain too deep: {binary} -> {target_name} -> {next_target}"
        )));
    }
    Ok(target_def)
}

pub fn default_risk(classification: Classification) -> Risk {
    match classification {
        Classification::Dangerous | Classification::Unknown => Risk::High,
        Classification::Readonly => Risk::Low,
        Classification::LocalEffects | Classification::ExternalEffects => Risk::Medium,
    }
}

pub fn clamp_risk(classification: Classification, risk: Risk) -> Risk {
    // Python parity: matcher.py _clamp_risk only clamps upward for Dangerous/Unknown.
    match classification {
        Classification::Dangerous | Classification::Unknown => Risk::High,
        _ => risk,
    }
}

pub fn py_basename(path: &str) -> String {
    // Python parity: os.path.basename returns an empty basename for trailing slash paths.
    match path.rsplit_once('/') {
        Some((_head, tail)) => tail.to_string(),
        None => path.to_string(),
    }
}

/// Python parity: matcher.py match_command, including lookup, overrides, and delegation floors.
pub fn match_command(
    invocation: &CommandInvocation,
    database: &impl CommandLookup,
) -> Result<CommandResult, MatchError> {
    let argv = &invocation.argv;
    if argv.is_empty() {
        return Ok(CommandResult {
            command: vec![],
            argv: vec![],
            classification: Classification::Unknown,
            risk: Risk::High,
            matched_rule: None,
            inner_commands: vec![],
            ignored_options: None,
            remaining_options: None,
            classification_reason: None,
            overriding_option: None,
            directories: None,
            write_paths: None,
            read_paths: None,
        });
    }

    let mut binary = argv[0].clone();

    if BUILTIN_DIRECTORY_COMMANDS.contains(&binary.as_str()) {
        return Ok(handle_directory_builtin(invocation));
    }
    if BUILTIN_READONLY_COMMANDS.contains(&binary.as_str()) {
        return Ok(handle_readonly_builtin(invocation));
    }
    if BUILTIN_DANGEROUS_COMMANDS.contains(&binary.as_str()) {
        return Ok(handle_dangerous_builtin(invocation));
    }

    // Python parity: try the exact binary, then basename fallback only for slash paths.
    let mut command_def = database.get_command(&binary)?;
    if command_def.is_none() && binary.contains('/') {
        let basename = py_basename(&binary);
        command_def = database.get_command(&basename)?;
        if command_def.is_some() {
            binary = basename;
        }
    }

    let Some(mut command_def) = command_def else {
        return Ok(CommandResult {
            command: vec![binary],
            argv: argv.clone(),
            classification: Classification::Unknown,
            risk: Risk::High,
            matched_rule: None,
            inner_commands: vec![],
            ignored_options: None,
            remaining_options: None,
            classification_reason: Some("command not in database".to_string()),
            overriding_option: None,
            directories: None,
            write_paths: None,
            read_paths: None,
        });
    };

    // Python parity: matcher-layer alias resolution preserves raw database get() behavior.
    command_def = database.resolve_alias(&binary, command_def)?;

    let remaining = argv[1..].to_vec();
    let (remaining, ignored_options, global_directories, global_overrides, global_risk_overrides) =
        strip_global_options(&remaining, &command_def);

    let (matched_def, command_chain, remaining, matched_defs) =
        if command_def.subcommand_mode == SubcommandMode::MatchAll {
            let (names, defs, rem) = match_all_subcommands(&remaining, &command_def);
            (command_def.clone(), names, rem, defs)
        } else {
            let (def, chain, rem) = match_subcommand(&remaining, &command_def);
            (def, chain, rem, vec![])
        };

    let ClassifiedOptions {
        unknown_options,
        overrides,
        risk_overrides,
        directories: option_directories,
        delegations: option_delegations,
        positional: remaining_positional,
        ..
    } = classify_options(&remaining, &matched_def);

    let mut unknown_options = unknown_options;
    let mut all_overrides = global_overrides;
    all_overrides.extend(overrides);
    let mut all_risk_overrides = global_risk_overrides;
    all_risk_overrides.extend(risk_overrides);
    let mut all_directories = global_directories;
    all_directories.extend(option_directories);

    let full_command: Vec<String> = std::iter::once(binary.clone())
        .chain(command_chain.iter().cloned())
        .collect();
    let matched_rule = if command_chain.is_empty() {
        binary.clone()
    } else {
        full_command.join(".")
    };

    let base_classification = matched_def
        .classification
        .unwrap_or(Classification::Readonly);
    let base_risk = matched_def
        .risk
        .unwrap_or_else(|| default_risk(base_classification));
    let risk_explicitly_set = matched_def.risk.is_some();

    let mut overriding_option: Option<String> = None;
    let mut classification_reason: Option<String>;
    let mut final_classification;

    if !all_overrides.is_empty() {
        // Python parity: option overrides aggregate by severity, not by last occurrence.
        let (opt, class) = highest_class_override(&all_overrides);
        final_classification = class;
        overriding_option = Some(opt.clone());
        classification_reason = Some(format!(
            "overridden by option {opt} to {final_classification}"
        ));
    } else {
        final_classification = base_classification;
        classification_reason = Some(format!("base classification from rule {matched_rule}"));
    }

    let mut final_risk = compute_risk(
        &all_risk_overrides,
        &all_overrides,
        risk_explicitly_set,
        final_classification,
        base_risk,
    );

    if !command_def.global_options.is_empty() {
        for opt in unknown_options.clone() {
            let opt_key = opt.split_once('=').map_or(opt.as_str(), |(key, _)| key);
            if let Some(global_opt_def) = command_def.global_options.get(opt_key) {
                if let Some(pos) = unknown_options
                    .iter()
                    .position(|candidate| candidate == &opt)
                {
                    unknown_options.remove(pos);
                }
                if let Some(override_class) = global_opt_def.overrides {
                    all_overrides.push((opt_key.to_string(), override_class));
                    let (opt_name, class) = highest_class_override(&all_overrides);
                    final_classification = class;
                    overriding_option = Some(opt_name.clone());
                    classification_reason = Some(format!(
                        "overridden by option {opt_name} to {final_classification}"
                    ));
                }
                if let Some(risk) = global_opt_def.risk {
                    all_risk_overrides.push((opt_key.to_string(), risk));
                }
                final_risk = compute_risk(
                    &all_risk_overrides,
                    &all_overrides,
                    risk_explicitly_set,
                    final_classification,
                    base_risk,
                );
            }
        }
    }

    if command_def.subcommand_mode == SubcommandMode::MatchAll && !matched_defs.is_empty() {
        let mut agg_classification = Classification::Readonly;
        let mut agg_risk = Risk::Low;
        for sub_def in &matched_defs {
            let sub_class = sub_def.classification.unwrap_or(Classification::Readonly);
            let sub_risk = sub_def.risk.unwrap_or_else(|| default_risk(sub_class));
            agg_classification = Classification::max_severity([agg_classification, sub_class]);
            agg_risk = Risk::max_severity([agg_risk, sub_risk]);
        }
        if remaining_positional
            .iter()
            .any(|token| !token.starts_with('-'))
        {
            // Python parity: unmatched positional args reintroduce the wrapper risk to aggregation.
            agg_risk = Risk::max_severity([agg_risk, base_risk]);
        }
        if !all_overrides.is_empty() {
            final_risk = Risk::max_severity([final_risk, agg_risk]);
        } else {
            final_classification =
                Classification::max_severity([final_classification, agg_classification]);
            final_risk = agg_risk;
            classification_reason = Some(format!("base classification from rule {matched_rule}"));
        }
    }

    if matched_def.strict && !unknown_options.is_empty() {
        // Python parity: strict unknown options escalate to Unknown and force High risk.
        final_classification =
            Classification::max_severity([final_classification, Classification::Unknown]);
        if final_classification == Classification::Unknown {
            classification_reason = Some(format!(
                "unrecognized option {} in strict mode",
                unknown_options[0]
            ));
        }
        final_risk = Risk::High;
    }

    let mut inner_commands = Vec::new();
    let mut command_level_inner_count = 0;

    if let Some(delegation) = &matched_def.delegates_to {
        let command_level_inner = handle_delegation(
            &remaining_positional,
            delegation,
            database,
            argv,
            &matched_def,
        )?;
        command_level_inner_count = command_level_inner.len();
        inner_commands.extend(command_level_inner);
    }

    for (opt_name, delegation_config, delegation_tokens) in option_delegations {
        let inner_results =
            handle_option_delegation(&opt_name, &delegation_config, &delegation_tokens, database)?;
        inner_commands.extend(inner_results);
    }

    if command_level_inner_count > 0
        // Python parity: command-level delegation can erase the wrapper unless overrides/strict block it.
        && all_overrides.is_empty()
        && !(matched_def.strict && !unknown_options.is_empty())
    {
        final_classification = Classification::Readonly;
        final_risk = Risk::Low;
        classification_reason = Some(format!("delegated to inner via {matched_rule}"));
    }

    for inner in &inner_commands {
        if inner.classification.severity() > final_classification.severity() {
            final_classification =
                Classification::max_severity([final_classification, inner.classification]);
            classification_reason = Some("elevated by inner command".to_string());
        }
        final_risk = Risk::max_severity([final_risk, inner.risk]);
    }

    final_risk = clamp_risk(final_classification, final_risk);

    Ok(CommandResult {
        command: full_command,
        argv: argv.clone(),
        classification: final_classification,
        risk: final_risk,
        matched_rule: Some(matched_rule),
        inner_commands,
        ignored_options: nonempty(ignored_options),
        remaining_options: nonempty(unknown_options),
        classification_reason,
        overriding_option,
        directories: nonempty(all_directories),
        write_paths: None,
        read_paths: None,
    })
}

fn nonempty<T>(values: Vec<T>) -> Option<Vec<T>> {
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

fn base_result(
    invocation: &CommandInvocation,
    classification: Classification,
    reason: String,
) -> CommandResult {
    CommandResult {
        command: vec![invocation.argv[0].clone()],
        argv: invocation.argv.clone(),
        classification,
        risk: default_risk(classification),
        matched_rule: None,
        inner_commands: vec![],
        ignored_options: None,
        remaining_options: None,
        classification_reason: Some(reason),
        overriding_option: None,
        directories: None,
        write_paths: None,
        read_paths: None,
    }
}

// Python parity: shell builtin handlers return synthetic matcher.py builtin results.
pub fn handle_directory_builtin(invocation: &CommandInvocation) -> CommandResult {
    base_result(
        invocation,
        Classification::Readonly,
        format!("shell builtin (always {})", Classification::Readonly),
    )
}

pub fn handle_readonly_builtin(invocation: &CommandInvocation) -> CommandResult {
    base_result(
        invocation,
        Classification::Readonly,
        format!("shell builtin (always {})", Classification::Readonly),
    )
}

pub fn handle_dangerous_builtin(invocation: &CommandInvocation) -> CommandResult {
    base_result(
        invocation,
        Classification::Dangerous,
        format!("shell builtin (always {})", Classification::Dangerous),
    )
}

pub type Overrides = Vec<(String, Classification)>;
pub type RiskOverrides = Vec<(String, Risk)>;

/// Python parity: matcher.py _strip_global_options stops at the first non-option token.
pub fn strip_global_options(
    argv: &[String],
    command_def: &CommandDef,
) -> (
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Overrides,
    RiskOverrides,
) {
    if command_def.global_options.is_empty() {
        return (argv.to_vec(), vec![], vec![], vec![], vec![]);
    }

    let mut remaining = Vec::new();
    let mut ignored = Vec::new();
    let mut directories = Vec::new();
    let mut overrides = Vec::new();
    let mut risk_overrides = Vec::new();
    let mut i = 0;

    while i < argv.len() {
        let token = &argv[i];
        if !token.starts_with('-') {
            remaining.extend_from_slice(&argv[i..]);
            break;
        }

        if let Some((key, value)) = token.split_once('=') {
            if let Some(opt_def) = command_def.global_options.get(key) {
                ignored.push(token.clone());
                if opt_def.captures_directory {
                    directories.push(value.to_string());
                }
                if let Some(classification) = opt_def.overrides {
                    overrides.push((key.to_string(), classification));
                }
                if let Some(risk) = opt_def.risk {
                    risk_overrides.push((key.to_string(), risk));
                }
                i += 1;
                continue;
            }
        }

        if let Some(opt_def) = command_def.global_options.get(token) {
            ignored.push(token.clone());
            if let Some(classification) = opt_def.overrides {
                overrides.push((token.clone(), classification));
            }
            if let Some(risk) = opt_def.risk {
                risk_overrides.push((token.clone(), risk));
            }
            if opt_def.takes_value && i + 1 < argv.len() {
                i += 1;
                ignored.push(argv[i].clone());
                if opt_def.captures_directory {
                    directories.push(argv[i].clone());
                }
            }
            i += 1;
            continue;
        }

        remaining.push(token.clone());
        i += 1;
    }

    (remaining, ignored, directories, overrides, risk_overrides)
}

/// Python parity: matcher.py _match_subcommand consumes contiguous non-option subcommands.
pub fn match_subcommand(
    argv: &[String],
    command_def: &CommandDef,
) -> (CommandDef, Vec<String>, Vec<String>) {
    let mut matched_def = command_def.clone();
    let mut command_chain = Vec::new();
    let mut remaining = argv.to_vec();
    while !remaining.is_empty() {
        let token = &remaining[0];
        if token.starts_with('-') {
            break;
        }
        if let Some(sub_def) = matched_def.subcommands.get(token) {
            command_chain.push(token.clone());
            matched_def = sub_def.clone();
            remaining.remove(0);
        } else {
            break;
        }
    }
    (matched_def, command_chain, remaining)
}

/// Python parity: matcher.py _match_all_subcommands collects every known subcommand for later max aggregation.
pub fn match_all_subcommands(
    argv: &[String],
    command_def: &CommandDef,
) -> (Vec<String>, Vec<CommandDef>, Vec<String>) {
    let mut matched_names = Vec::new();
    let mut matched_defs = Vec::new();
    let mut remaining = Vec::new();
    for token in argv {
        if token.starts_with('-') {
            remaining.push(token.clone());
        } else if let Some(sub_def) = command_def.subcommands.get(token) {
            matched_names.push(token.clone());
            matched_defs.push(sub_def.clone());
        } else {
            remaining.push(token.clone());
        }
    }
    (matched_names, matched_defs, remaining)
}

#[derive(Debug, Default, Clone)]
pub struct ClassifiedOptions {
    pub known_options: Vec<String>,
    pub unknown_options: Vec<String>,
    pub overrides: Overrides,
    pub risk_overrides: RiskOverrides,
    pub directories: Vec<String>,
    pub delegations: Vec<(String, DelegationConfig, Vec<String>)>,
    pub positional: Vec<String>,
}

/// Python parity: matcher.py _classify_options recognizes all seven option forms before strict handling.
pub fn classify_options(argv: &[String], command_def: &CommandDef) -> ClassifiedOptions {
    let mut out = ClassifiedOptions::default();
    let options = &command_def.options;
    let stop_at_first_positional = command_def
        .delegates_to
        .as_ref()
        .is_some_and(|delegation| delegation.mode == DelegationMode::RestAreArgv);
    let mut i = 0;

    while i < argv.len() {
        let token = &argv[i];

        if token == "--" {
            out.positional.push(token.clone());
            i += 1;
            while i < argv.len() {
                out.positional.push(argv[i].clone());
                i += 1;
            }
            break;
        }

        if !token.starts_with('-') {
            if stop_at_first_positional {
                i += command_def
                    .delegates_to
                    .as_ref()
                    .map_or(0, |delegation| delegation.skip_leading_positionals);
                out.positional.extend_from_slice(&argv[i.min(argv.len())..]);
                break;
            }
            out.positional.push(token.clone());
            i += 1;
            continue;
        }

        if token.starts_with("--") && token.contains('=') {
            let (key, value) = token.split_once('=').expect("contains '='");
            if let Some(opt_def) = options.get(key) {
                out.known_options.push(token.clone());
                apply_option_effects(key, opt_def, value, &mut out, false);
            } else {
                out.unknown_options.push(token.clone());
            }
            i += 1;
            continue;
        }

        if token.starts_with("--") {
            if let Some(opt_def) = options.get(token) {
                out.known_options.push(token.clone());
                apply_option_effects(token, opt_def, "", &mut out, false);
                if let Some(delegation) = &opt_def.delegates_to {
                    let delegation_tokens = extract_delegation_tokens(argv, i, delegation);
                    out.delegations
                        .push((token.clone(), delegation.clone(), delegation_tokens));
                    i = skip_delegation_tokens(argv, i, delegation);
                    continue;
                }
                if opt_def.takes_value && i + 1 < argv.len() {
                    i += 1;
                    let value = &argv[i];
                    out.known_options.push(value.clone());
                    if opt_def.captures_directory {
                        out.directories.push(value.clone());
                    }
                }
            } else {
                out.unknown_options.push(token.clone());
            }
            i += 1;
            continue;
        }

        if let Some(opt_def) = options.get(token) {
            out.known_options.push(token.clone());
            apply_option_effects(token, opt_def, "", &mut out, false);
            if let Some(delegation) = &opt_def.delegates_to {
                let delegation_tokens = extract_delegation_tokens(argv, i, delegation);
                out.delegations
                    .push((token.clone(), delegation.clone(), delegation_tokens));
                i = skip_delegation_tokens(argv, i, delegation);
                continue;
            }
            if opt_def.takes_value && i + 1 < argv.len() {
                i += 1;
                let value = &argv[i];
                out.known_options.push(value.clone());
                if opt_def.captures_directory {
                    out.directories.push(value.clone());
                }
            }
            i += 1;
            continue;
        }

        if token.len() >= 2 && token.starts_with('-') && !token.starts_with("--") {
            let mut chars = token.chars();
            let _dash = chars.next();
            let second = chars.next().expect("len >= 2");
            let short_flag = format!("-{second}");
            let opt_def = options.get(&short_flag);

            if let Some(opt_def) = opt_def.filter(|opt| opt.takes_value && token.len() > 2) {
                out.known_options.push(token.clone());
                let value = token[2..].to_string();
                apply_option_effects(&short_flag, opt_def, &value, &mut out, true);
                i += 1;
                continue;
            }

            if let Some(first_def) = opt_def.filter(|opt| !opt.takes_value && token.len() > 2) {
                let _ = first_def;
                let token_chars: Vec<char> = token.chars().collect();
                let mut all_known = true;
                let mut pending_overrides = Vec::new();
                let mut pending_risk_overrides = Vec::new();
                let mut pending_directories = Vec::new();
                for j in 1..token_chars.len() {
                    let char_flag = format!("-{}", token_chars[j]);
                    let Some(char_def) = options.get(&char_flag) else {
                        all_known = false;
                        break;
                    };
                    if char_def.takes_value {
                        if let Some(classification) = char_def.overrides {
                            pending_overrides.push((char_flag.clone(), classification));
                        }
                        if let Some(risk) = char_def.risk {
                            pending_risk_overrides.push((char_flag.clone(), risk));
                        }
                        let remaining_chars: String = token_chars[j + 1..].iter().collect();
                        if !remaining_chars.is_empty() {
                            if char_def.captures_directory {
                                pending_directories.push(remaining_chars);
                            }
                        } else if i + 1 < argv.len() {
                            i += 1;
                            out.known_options.push(argv[i].clone());
                            if char_def.captures_directory {
                                pending_directories.push(argv[i].clone());
                            }
                        }
                        break;
                    }
                    if let Some(classification) = char_def.overrides {
                        pending_overrides.push((char_flag.clone(), classification));
                    }
                    if let Some(risk) = char_def.risk {
                        pending_risk_overrides.push((char_flag, risk));
                    }
                }
                if all_known {
                    out.known_options.push(token.clone());
                    out.overrides.extend(pending_overrides);
                    out.risk_overrides.extend(pending_risk_overrides);
                    out.directories.extend(pending_directories);
                    i += 1;
                    continue;
                }
            }

            if let Some(opt_def) = opt_def {
                out.known_options.push(token.clone());
                apply_option_effects(&short_flag, opt_def, "", &mut out, false);
                if opt_def.takes_value && i + 1 < argv.len() {
                    i += 1;
                    let value = &argv[i];
                    out.known_options.push(value.clone());
                    if opt_def.captures_directory {
                        out.directories.push(value.clone());
                    }
                }
                i += 1;
                continue;
            }
        }

        out.unknown_options.push(token.clone());
        i += 1;
    }

    out
}

fn apply_option_effects(
    opt_name: &str,
    opt_def: &crate::models::OptionDef,
    value: &str,
    out: &mut ClassifiedOptions,
    value_from_joined_form: bool,
) {
    if let Some(classification) = opt_def.overrides {
        out.overrides.push((opt_name.to_string(), classification));
    }
    if let Some(risk) = opt_def.risk {
        out.risk_overrides.push((opt_name.to_string(), risk));
    }
    if value_from_joined_form && opt_def.captures_directory {
        out.directories.push(value.to_string());
    }
    if !value_from_joined_form && !value.is_empty() && opt_def.captures_directory {
        out.directories.push(value.to_string());
    }
}

/// Python parity: matcher.py _is_terminator accepts the raw or backslash-escaped terminator.
pub fn is_terminator(token: &str, terminator: Option<&str>) -> bool {
    let Some(terminator) = terminator else {
        return false;
    };
    token == terminator || token == format!("\\{terminator}")
}

pub fn extract_delegation_tokens(
    argv: &[String],
    option_index: usize,
    delegation: &DelegationConfig,
) -> Vec<String> {
    if delegation.mode == DelegationMode::TerminatedArgv {
        let mut tokens = Vec::new();
        let mut i = option_index + 1;
        while i < argv.len() {
            if is_terminator(&argv[i], delegation.terminator.as_deref()) {
                break;
            }
            tokens.push(argv[i].clone());
            i += 1;
        }
        tokens
    } else {
        vec![]
    }
}

pub fn skip_delegation_tokens(
    argv: &[String],
    option_index: usize,
    delegation: &DelegationConfig,
) -> usize {
    if delegation.mode == DelegationMode::TerminatedArgv {
        let mut i = option_index + 1;
        while i < argv.len() {
            if is_terminator(&argv[i], delegation.terminator.as_deref()) {
                return i + 1;
            }
            i += 1;
        }
        argv.len()
    } else {
        option_index + 1
    }
}

/// Python parity: matcher.py _handle_delegation supports rest-argv, separator, expression, and option-owned modes.
pub fn handle_delegation(
    remaining_positional: &[String],
    delegation: &DelegationConfig,
    database: &impl CommandLookup,
    full_argv: &[String],
    command_def: &CommandDef,
) -> Result<Vec<InnerCommandResult>, MatchError> {
    let mut results = Vec::new();
    match delegation.mode {
        // Python parity: per-mode knobs are local here: assignment stripping, separators, flags, floors.
        DelegationMode::RestAreArgv => {
            let mut inner_argv = remaining_positional.to_vec();
            if delegation.strip_assignments {
                while inner_argv.first().is_some_and(|token| is_assignment(token)) {
                    inner_argv.remove(0);
                }
            }
            if !inner_argv.is_empty() {
                results.push(match_inner_command(
                    &inner_argv,
                    database,
                    "rest_are_argv",
                    &command_def.command,
                    delegation.min_classification,
                )?);
            }
        }
        DelegationMode::AfterSeparator => {
            let separator = delegation.separator.as_deref().unwrap_or("--");
            if let Some(sep_index) = remaining_positional
                .iter()
                .position(|token| token == separator)
            {
                let inner_argv = remaining_positional[sep_index + 1..].to_vec();
                if !inner_argv.is_empty() {
                    results.push(match_inner_command(
                        &inner_argv,
                        database,
                        "after_separator",
                        separator,
                        delegation.min_classification,
                    )?);
                }
            }
        }
        DelegationMode::FlagValueIsExpression => {
            let Some(flag) = &delegation.flag else {
                return Ok(results);
            };
            if let Some(expression_value) = find_flag_value(full_argv, flag) {
                let (inner_invocations, _warnings) =
                    crate::parser::parse_expression(&expression_value)?;
                for inv in inner_invocations {
                    results.push(match_inner_command(
                        &inv.argv,
                        database,
                        "flag_value_is_expression",
                        flag,
                        delegation.min_classification,
                    )?);
                }
            }
        }
        DelegationMode::TerminatedArgv => {}
    }
    Ok(results)
}

/// Python parity: matcher.py _handle_option_delegation filters placeholder tokens before recursion.
pub fn handle_option_delegation(
    option_name: &str,
    delegation: &DelegationConfig,
    delegation_tokens: &[String],
    database: &impl CommandLookup,
) -> Result<Vec<InnerCommandResult>, MatchError> {
    if delegation.mode == DelegationMode::TerminatedArgv {
        let inner_argv: Vec<String> = delegation_tokens
            .iter()
            .filter(|token| token.as_str() != "{}")
            .cloned()
            .collect();
        if !inner_argv.is_empty() {
            return Ok(vec![match_inner_command(
                &inner_argv,
                database,
                "terminated_argv",
                option_name,
                delegation.min_classification,
            )?]);
        }
    }
    Ok(vec![])
}

/// Python parity: matcher.py _match_inner_command applies delegation min_classification as a floor.
pub fn match_inner_command(
    argv: &[String],
    database: &impl CommandLookup,
    delegation_mode: &str,
    delegation_source: &str,
    min_classification: Option<Classification>,
) -> Result<InnerCommandResult, MatchError> {
    let inner_invocation = CommandInvocation {
        argv: argv.to_vec(),
        redirects: vec![],
        position_in_pipeline: 0,
        pipeline_length: 1,
        context: "toplevel".to_string(),
        operator_before: None,
        is_background: false,
    };
    let inner_result = match_command(&inner_invocation, database)?;
    let mut classification = inner_result.classification;
    let mut risk = inner_result.risk;
    if let Some(min_classification) = min_classification {
        // Python parity: the floor can only raise classification/risk, never lower the inner result.
        classification = Classification::max_severity([classification, min_classification]);
        risk = Risk::max_severity([risk, default_risk(min_classification)]);
    }
    risk = clamp_risk(classification, risk);
    Ok(InnerCommandResult {
        delegation_mode: delegation_mode.to_string(),
        delegation_source: delegation_source.to_string(),
        command: inner_result.command,
        argv: argv.to_vec(),
        classification,
        risk,
        matched_rule: inner_result.matched_rule,
        inner_commands: inner_result.inner_commands,
        ignored_options: inner_result.ignored_options,
        remaining_options: inner_result.remaining_options,
        overriding_option: inner_result.overriding_option,
    })
}

/// Python parity: matcher.py _find_flag_value checks separated then flag=value forms and strips shell quotes.
pub fn find_flag_value(argv: &[String], flag: &str) -> Option<String> {
    let mut value = None;
    for (i, token) in argv.iter().enumerate() {
        if token == flag && i + 1 < argv.len() {
            value = Some(argv[i + 1].clone());
            break;
        }
        let prefix = format!("{flag}=");
        if token.starts_with(&prefix) {
            value = Some(token[prefix.len()..].to_string());
            break;
        }
    }
    value.map(|raw| strip_quotes(&raw))
}

/// Python parity: matcher.py _strip_quotes removes one matching quote pair only.
pub fn strip_quotes(s: &str) -> String {
    if s.len() >= 2 {
        let bytes = s.as_bytes();
        if (bytes[0] == b'\"' && bytes[s.len() - 1] == b'\"')
            || (bytes[0] == b'\'' && bytes[s.len() - 1] == b'\'')
        {
            return s[1..s.len() - 1].to_string();
        }
    }
    s.to_string()
}

fn highest_class_override(overrides: &[(String, Classification)]) -> (&String, Classification) {
    let (mut opt, mut class) = (&overrides[0].0, overrides[0].1);
    for (candidate_opt, candidate_class) in &overrides[1..] {
        if candidate_class.severity() > class.severity() {
            opt = candidate_opt;
            class = *candidate_class;
        }
    }
    (opt, class)
}

fn compute_risk(
    risk_overrides: &[(String, Risk)],
    overrides: &[(String, Classification)],
    risk_explicitly_set: bool,
    final_classification: Classification,
    base_risk: Risk,
) -> Risk {
    if !risk_overrides.is_empty() {
        risk_overrides
            .iter()
            .map(|(_, risk)| *risk)
            .max_by_key(|risk| risk.severity())
            .unwrap_or(Risk::Low)
    } else if !overrides.is_empty() && !risk_explicitly_set {
        default_risk(final_classification)
    } else {
        base_risk
    }
}

fn is_assignment(token: &str) -> bool {
    let Some(eq_index) = token.find('=') else {
        return false;
    };
    let name = &token[..eq_index];
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::{is_terminator, py_basename, strip_quotes};

    #[test]
    fn test_py_basename() {
        assert_eq!(py_basename("/usr/bin/ls"), "ls");
        assert_eq!(py_basename("a/b/"), "");
        assert_eq!(py_basename(""), "");
        assert_eq!(py_basename("ls"), "ls");
    }

    #[test]
    fn test_is_terminator() {
        assert!(is_terminator(";", Some(";")));
        assert!(is_terminator("\\;", Some(";")));
        assert!(!is_terminator("+", Some(";")));
        assert!(!is_terminator(";", None));
    }

    #[test]
    fn test_strip_quotes() {
        assert_eq!(strip_quotes("'hello world'"), "hello world");
        assert_eq!(strip_quotes("\"hello world\""), "hello world");
        assert_eq!(strip_quotes("hello"), "hello");
    }
}
