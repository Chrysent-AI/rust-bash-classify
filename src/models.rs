//! Data models for bash-classify.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Classification {
    Readonly,
    LocalEffects,
    ExternalEffects,
    Dangerous,
    Unknown,
}

impl Classification {
    pub fn severity(self) -> u8 {
        match self {
            Self::Readonly => 0,
            Self::LocalEffects => 1,
            Self::ExternalEffects => 2,
            Self::Unknown => 3,
            Self::Dangerous => 4,
        }
    }

    pub fn max_severity<I>(classifications: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        classifications
            .into_iter()
            .max_by_key(|classification| classification.severity())
            .unwrap_or(Self::Readonly)
    }
}

impl std::fmt::Display for Classification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Readonly => "READONLY",
            Self::LocalEffects => "LOCAL_EFFECTS",
            Self::ExternalEffects => "EXTERNAL_EFFECTS",
            Self::Dangerous => "DANGEROUS",
            Self::Unknown => "UNKNOWN",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Risk {
    Low,
    Medium,
    High,
}

impl Risk {
    pub fn severity(self) -> u8 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
        }
    }

    pub fn max_severity<I>(risks: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        risks
            .into_iter()
            .max_by_key(|risk| risk.severity())
            .unwrap_or(Self::Low)
    }
}

impl std::fmt::Display for Risk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubcommandMode {
    Hierarchical,
    MatchAll,
}

impl Default for SubcommandMode {
    fn default() -> Self {
        Self::Hierarchical
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationMode {
    RestAreArgv,
    AfterSeparator,
    TerminatedArgv,
    FlagValueIsExpression,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Redirect {
    pub operator: String,
    pub target: String,
    pub affects_classification: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandInvocation {
    pub argv: Vec<String>,
    pub redirects: Vec<Redirect>,
    pub position_in_pipeline: usize,
    pub pipeline_length: usize,
    pub context: String,
    pub operator_before: Option<String>,
    pub is_background: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationConfig {
    pub mode: DelegationMode,
    pub separator: Option<String>,
    pub terminator: Option<String>,
    pub flag: Option<String>,
    pub strip_assignments: bool,
    pub skip_leading_positionals: usize,
    pub min_classification: Option<Classification>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionDef {
    pub takes_value: bool,
    pub aliases: Vec<String>,
    pub overrides: Option<Classification>,
    pub risk: Option<Risk>,
    pub captures_directory: bool,
    pub delegates_to: Option<DelegationConfig>,
}

impl Default for OptionDef {
    fn default() -> Self {
        Self {
            takes_value: false,
            aliases: Vec::new(),
            overrides: None,
            risk: None,
            captures_directory: false,
            delegates_to: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandDef {
    pub command: String,
    pub alias_of: Option<String>,
    pub classification: Option<Classification>,
    pub risk: Option<Risk>,
    pub global_options: HashMap<String, OptionDef>,
    pub subcommands: HashMap<String, CommandDef>,
    pub options: HashMap<String, OptionDef>,
    pub strict: bool,
    pub subcommand_mode: SubcommandMode,
    pub delegates_to: Option<DelegationConfig>,
}

impl CommandDef {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            alias_of: None,
            classification: None,
            risk: None,
            global_options: HashMap::new(),
            subcommands: HashMap::new(),
            options: HashMap::new(),
            strict: true,
            subcommand_mode: SubcommandMode::Hierarchical,
            delegates_to: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InnerCommandResult {
    pub delegation_mode: String,
    pub delegation_source: String,
    pub command: Vec<String>,
    pub argv: Vec<String>,
    pub classification: Classification,
    pub risk: Risk,
    pub matched_rule: Option<String>,
    pub inner_commands: Vec<InnerCommandResult>,
    pub ignored_options: Option<Vec<String>>,
    pub remaining_options: Option<Vec<String>>,
    pub overriding_option: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: Vec<String>,
    pub argv: Vec<String>,
    pub classification: Classification,
    pub risk: Risk,
    pub matched_rule: Option<String>,
    pub inner_commands: Vec<InnerCommandResult>,
    pub ignored_options: Option<Vec<String>>,
    pub remaining_options: Option<Vec<String>>,
    pub classification_reason: Option<String>,
    pub overriding_option: Option<String>,
    pub directories: Option<Vec<String>>,
    pub write_paths: Option<Vec<String>>,
    pub read_paths: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpressionResult {
    pub expression: String,
    pub classification: Classification,
    pub risk: Risk,
    pub directories: Vec<String>,
    pub write_paths: Vec<String>,
    pub read_paths: Vec<String>,
    pub commands: Vec<CommandResult>,
    pub redirects: Vec<Redirect>,
    pub parse_warnings: Vec<String>,
}

#[cfg(test)]
mod test_models {
    use super::{Classification, Risk, SubcommandMode};

    mod test_classification_severity {
        use super::Classification;

        #[test]
        fn test_severity_ordering() {
            assert!(
                Classification::Readonly.severity() < Classification::ExternalEffects.severity()
            );
            assert!(
                Classification::ExternalEffects.severity() < Classification::Unknown.severity()
            );
            assert!(Classification::Unknown.severity() < Classification::Dangerous.severity());
        }

        #[test]
        fn test_max_severity_empty() {
            assert_eq!(Classification::max_severity([]), Classification::Readonly);
        }

        #[test]
        fn test_max_severity_single() {
            assert_eq!(
                Classification::max_severity([Classification::ExternalEffects]),
                Classification::ExternalEffects
            );
        }

        #[test]
        fn test_max_severity_multiple() {
            let result = Classification::max_severity([
                Classification::Readonly,
                Classification::Dangerous,
                Classification::ExternalEffects,
            ]);
            assert_eq!(result, Classification::Dangerous);
        }

        #[test]
        fn test_max_severity_ties() {
            let result = Classification::max_severity([
                Classification::ExternalEffects,
                Classification::ExternalEffects,
            ]);
            assert_eq!(result, Classification::ExternalEffects);
        }

        #[test]
        fn test_max_severity_all_four() {
            let result = Classification::max_severity([
                Classification::Readonly,
                Classification::ExternalEffects,
                Classification::Unknown,
                Classification::Dangerous,
            ]);
            assert_eq!(result, Classification::Dangerous);
        }
    }

    mod test_risk_severity {
        use super::Risk;

        #[test]
        fn test_severity_ordering() {
            assert!(Risk::Low.severity() < Risk::Medium.severity());
            assert!(Risk::Medium.severity() < Risk::High.severity());
        }

        #[test]
        fn test_max_severity_empty() {
            assert_eq!(Risk::max_severity([]), Risk::Low);
        }

        #[test]
        fn test_max_severity_single() {
            assert_eq!(Risk::max_severity([Risk::Medium]), Risk::Medium);
        }

        #[test]
        fn test_max_severity_multiple() {
            let result = Risk::max_severity([Risk::Low, Risk::High, Risk::Medium]);
            assert_eq!(result, Risk::High);
        }

        #[test]
        fn test_max_severity_ties() {
            let result = Risk::max_severity([Risk::Medium, Risk::Medium]);
            assert_eq!(result, Risk::Medium);
        }

        #[test]
        fn test_max_severity_all_three() {
            let result = Risk::max_severity([Risk::Low, Risk::Medium, Risk::High]);
            assert_eq!(result, Risk::High);
        }
    }

    mod test_subcommand_mode {
        use super::SubcommandMode;

        #[test]
        fn test_subcommand_mode_values() {
            assert_eq!(
                serde_yaml::to_string(&SubcommandMode::Hierarchical)
                    .unwrap()
                    .trim(),
                "hierarchical"
            );
            assert_eq!(
                serde_yaml::to_string(&SubcommandMode::MatchAll)
                    .unwrap()
                    .trim(),
                "match_all"
            );
        }
    }
}
