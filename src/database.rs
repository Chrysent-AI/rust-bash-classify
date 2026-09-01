//! Command database loading from YAML files.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[cfg(unix)]
use std::ffi::CStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

use serde_yaml::{Mapping, Value};

use crate::commands_data;
use crate::models::{
    Classification, CommandDef, DelegationConfig, DelegationMode, OptionDef, Risk, SubcommandMode,
};

#[derive(Debug)]
pub struct DbError(String);

impl DbError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DbError {}

#[derive(Clone, Debug)]
enum CommandSource {
    Builtin {
        display_path: String,
        yaml_text: &'static str,
    },
    User {
        path: PathBuf,
    },
}

/// Lazy-loading command database keyed by command name.
pub struct CommandDatabase {
    files: HashMap<String, CommandSource>,
    cache: Mutex<HashMap<String, CommandDef>>,
}

impl CommandDatabase {
    pub fn new(builtin_dir: Option<&Path>, user_dir: Option<&Path>) -> Result<Self, DbError> {
        let mut files = HashMap::new();

        if let Some(builtin_dir) = builtin_dir {
            index_dir(&mut files, builtin_dir)?;
        } else {
            for (name, display_path, yaml_text) in commands_data::COMMANDS {
                files.insert(
                    (*name).to_string(),
                    CommandSource::Builtin {
                        display_path: (*display_path).to_string(),
                        yaml_text,
                    },
                );
            }
        }

        if let Some(user_dir) = user_dir {
            if user_dir.is_dir() {
                // Mirrors database.py CommandDatabase.__init__: user filename stems replace built-ins; no merging.
                index_dir(&mut files, user_dir)?;
            }
        }

        Ok(Self {
            files,
            cache: Mutex::new(HashMap::new()),
        })
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.cache
            .lock()
            .expect("database cache poisoned")
            .contains_key(key)
            || self.files.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.files.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn get(&self, key: &str) -> Result<Option<CommandDef>, DbError> {
        // Mirrors database.py CommandDatabase.__getitem__: parse on first access so unrelated invalid YAML does not poison the database.
        if let Some(command_def) = self
            .cache
            .lock()
            .expect("database cache poisoned")
            .get(key)
            .cloned()
        {
            return Ok(Some(command_def));
        }

        let Some(source) = self.files.get(key).cloned() else {
            return Ok(None);
        };

        let command_def = load_command_source(&source)?;
        self.cache
            .lock()
            .expect("database cache poisoned")
            .insert(key.to_string(), command_def.clone());
        Ok(Some(command_def))
    }

    pub fn get_required(&self, key: &str) -> Result<CommandDef, DbError> {
        self.get(key)?
            .ok_or_else(|| DbError::new(format!("{key} is not in the database")))
    }

    /// Resolve an `alias_of` pointer with Python matcher's message text.
    pub fn resolve_alias(
        &self,
        binary: &str,
        command_def: CommandDef,
    ) -> Result<CommandDef, DbError> {
        crate::matcher::resolve_alias_def(self, binary, command_def)
            .map_err(|error| DbError::new(error.to_string()))
    }
}

pub fn get_default_commands_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vendor")
        .join("commands")
}

pub fn get_user_commands_dir() -> PathBuf {
    // Mirrors database.py get_user_commands_dir: an empty BASH_CLASSIFY_CONFIG_DIR is falsey.
    if let Some(config_dir) =
        std::env::var_os("BASH_CLASSIFY_CONFIG_DIR").filter(|value| !value.is_empty())
    {
        return PathBuf::from(config_dir).join("commands");
    }
    home_dir()
        .join(".config")
        .join("bash-classify")
        .join("commands")
}

pub fn load_database(commands_dir: Option<&Path>) -> Result<CommandDatabase, DbError> {
    if let Some(commands_dir) = commands_dir {
        return CommandDatabase::new(Some(commands_dir), None);
    }

    let user_dir = get_user_commands_dir();
    let user_dir = user_dir.is_dir().then_some(user_dir);
    CommandDatabase::new(None, user_dir.as_deref())
}

fn home_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME").filter(|value| !value.is_empty()) {
        return PathBuf::from(home);
    }
    home_dir_from_passwd().unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(unix)]
fn home_dir_from_passwd() -> Option<PathBuf> {
    #[repr(C)]
    struct Passwd {
        pw_name: *mut std::os::raw::c_char,
        pw_passwd: *mut std::os::raw::c_char,
        pw_uid: u32,
        pw_gid: u32,
        pw_gecos: *mut std::os::raw::c_char,
        pw_dir: *mut std::os::raw::c_char,
        pw_shell: *mut std::os::raw::c_char,
    }

    extern "C" {
        fn getuid() -> u32;
        fn getpwuid_r(
            uid: u32,
            pwd: *mut Passwd,
            buf: *mut std::os::raw::c_char,
            buflen: usize,
            result: *mut *mut Passwd,
        ) -> std::os::raw::c_int;
    }

    let mut pwd = Passwd {
        pw_name: std::ptr::null_mut(),
        pw_passwd: std::ptr::null_mut(),
        pw_uid: 0,
        pw_gid: 0,
        pw_gecos: std::ptr::null_mut(),
        pw_dir: std::ptr::null_mut(),
        pw_shell: std::ptr::null_mut(),
    };
    let mut result = std::ptr::null_mut();
    let mut buffer = vec![0; 16_384];
    let status = unsafe {
        getpwuid_r(
            getuid(),
            &mut pwd,
            buffer.as_mut_ptr(),
            buffer.len(),
            &mut result,
        )
    };
    if status != 0 || result.is_null() || pwd.pw_dir.is_null() {
        return None;
    }
    let bytes = unsafe { CStr::from_ptr(pwd.pw_dir) }.to_bytes().to_vec();
    Some(PathBuf::from(std::ffi::OsString::from_vec(bytes)))
}

#[cfg(not(unix))]
fn home_dir_from_passwd() -> Option<PathBuf> {
    None
}

fn index_dir(
    files: &mut HashMap<String, CommandSource>,
    commands_dir: &Path,
) -> Result<(), DbError> {
    if !commands_dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(commands_dir)
        .map_err(|err| DbError::new(format!("Error scanning {}: {err}", commands_dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|err| {
            DbError::new(format!("Error scanning {}: {err}", commands_dir.display()))
        })?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
            let name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default()
                .to_string();
            files.insert(name, CommandSource::User { path });
        }
    }
    Ok(())
}

fn load_command_source(source: &CommandSource) -> Result<CommandDef, DbError> {
    match source {
        CommandSource::Builtin {
            display_path,
            yaml_text,
        } => load_command_yaml(display_path, yaml_text),
        CommandSource::User { path } => {
            let yaml_text = fs::read_to_string(path)
                .map_err(|err| DbError::new(format!("Error loading {}: {err}", path.display())))?;
            load_command_yaml(&path.display().to_string(), &yaml_text)
        }
    }
}

fn load_command_yaml(display_path: &str, yaml_text: &str) -> Result<CommandDef, DbError> {
    parse_command_yaml(yaml_text)
        .map_err(|err| DbError::new(format!("Error loading {display_path}: {err}")))
}

fn parse_command_yaml(yaml_text: &str) -> Result<CommandDef, String> {
    let data: Value = serde_yaml::from_str(yaml_text).map_err(|err| err.to_string())?;
    if data.is_null() {
        return Err("Empty YAML file".to_string());
    }
    let mapping = data
        .as_mapping()
        .ok_or_else(|| format!("Expected a YAML mapping, got {}", yaml_type_name(&data)))?;
    let command_name = yaml_str(required_value(mapping, "command")?);
    parse_command_def(mapping, &command_name)
}

fn required_value<'a>(mapping: &'a Mapping, key: &str) -> Result<&'a Value, String> {
    mapping
        .get(Value::String(key.to_string()))
        .ok_or_else(|| format!("'{key}'"))
}

fn optional_value<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Value> {
    mapping.get(Value::String(key.to_string()))
}

fn yaml_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "NoneType",
        Value::Bool(_) => "bool",
        Value::Number(_) => "int",
        Value::String(_) => "str",
        Value::Sequence(_) => "list",
        Value::Mapping(_) => "dict",
        Value::Tagged(_) => "tagged",
    }
}

fn yaml_str(value: &Value) -> String {
    // Mirrors database.py _yaml_str: YAML booleans become lowercase command names such as true/false.
    match value {
        Value::Bool(value) => {
            if *value {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Null => "None".to_string(),
        _ => serde_yaml::to_string(value)
            .unwrap_or_else(|_| format!("{value:?}"))
            .trim()
            .to_string(),
    }
}

const ALIAS_FORBIDDEN_KEYS: &[&str] = &[
    "classification",
    "risk",
    "subcommands",
    "options",
    "global_options",
    "subcommand_mode",
    "delegates_to",
    "strict",
];

fn parse_command_def(data: &Mapping, command_name: &str) -> Result<CommandDef, String> {
    let alias_of = optional_value(data, "alias_of");
    if let Some(alias_of) = alias_of {
        let mut conflicting: Vec<_> = ALIAS_FORBIDDEN_KEYS
            .iter()
            .filter(|key| optional_value(data, key).is_some())
            .copied()
            .collect();
        conflicting.sort();
        if !conflicting.is_empty() {
            return Err(format!(
                "alias file for '{command_name}' has alias_of='{}' combined with forbidden field(s): {}",
                yaml_str(alias_of),
                conflicting.join(", ")
            ));
        }
        let mut command_def = CommandDef::new(command_name);
        command_def.alias_of = Some(yaml_str(alias_of));
        return Ok(command_def);
    }

    let mut command_def = CommandDef::new(command_name);
    command_def.classification = parse_classification(optional_value(data, "classification"))?;
    command_def.risk = parse_risk(optional_value(data, "risk"))?;
    command_def.global_options = parse_options(optional_value(data, "global_options"))?;
    command_def.options = parse_options(optional_value(data, "options"))?;
    command_def.subcommands = parse_subcommands(optional_value(data, "subcommands"))?;
    command_def.delegates_to = parse_delegation_config(optional_value(data, "delegates_to"))?;
    command_def.strict = optional_value(data, "strict")
        .map(yaml_bool)
        .unwrap_or(true);
    command_def.subcommand_mode = parse_subcommand_mode(optional_value(data, "subcommand_mode"))?;
    Ok(command_def)
}

fn yaml_bool(value: &Value) -> bool {
    match value {
        Value::Bool(value) => *value,
        Value::Null => false,
        _ => true,
    }
}

fn yaml_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(number) => {
            if let Some(number) = number.as_i64() {
                number != 0
            } else if let Some(number) = number.as_u64() {
                number != 0
            } else if let Some(number) = number.as_f64() {
                number != 0.0
            } else {
                true
            }
        }
        Value::String(value) => !value.is_empty(),
        Value::Sequence(values) => !values.is_empty(),
        Value::Mapping(values) => !values.is_empty(),
        Value::Tagged(tagged) => yaml_truthy(&tagged.value),
    }
}

fn parse_classification(value: Option<&Value>) -> Result<Option<Classification>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = yaml_str(value);
    let classification = match value.as_str() {
        "READONLY" => Classification::Readonly,
        "LOCAL_EFFECTS" => Classification::LocalEffects,
        "EXTERNAL_EFFECTS" => Classification::ExternalEffects,
        "DANGEROUS" => Classification::Dangerous,
        "UNKNOWN" => Classification::Unknown,
        _ => return Err(format!("'{value}' is not a valid Classification")),
    };
    Ok(Some(classification))
}

fn parse_risk(value: Option<&Value>) -> Result<Option<Risk>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = yaml_str(value);
    let risk = match value.as_str() {
        "LOW" => Risk::Low,
        "MEDIUM" => Risk::Medium,
        "HIGH" => Risk::High,
        _ => return Err(format!("'{value}' is not a valid Risk")),
    };
    Ok(Some(risk))
}

fn parse_subcommand_mode(value: Option<&Value>) -> Result<SubcommandMode, String> {
    let Some(value) = value else {
        return Ok(SubcommandMode::Hierarchical);
    };
    match yaml_str(value).as_str() {
        "hierarchical" => Ok(SubcommandMode::Hierarchical),
        "match_all" => Ok(SubcommandMode::MatchAll),
        other => Err(format!("'{other}' is not a valid SubcommandMode")),
    }
}

fn parse_delegation_mode(value: &Value) -> Result<DelegationMode, String> {
    match yaml_str(value).as_str() {
        "rest_are_argv" => Ok(DelegationMode::RestAreArgv),
        "after_separator" => Ok(DelegationMode::AfterSeparator),
        "terminated_argv" => Ok(DelegationMode::TerminatedArgv),
        "flag_value_is_expression" => Ok(DelegationMode::FlagValueIsExpression),
        other => Err(format!("'{other}' is not a valid DelegationMode")),
    }
}

fn parse_options(raw: Option<&Value>) -> Result<HashMap<String, OptionDef>, String> {
    let Some(raw) = raw else {
        return Ok(HashMap::new());
    };
    if !yaml_truthy(raw) {
        return Ok(HashMap::new());
    }
    let mapping = raw
        .as_mapping()
        .ok_or_else(|| format!("Expected a YAML mapping, got {}", yaml_type_name(raw)))?;

    let mut options = HashMap::new();
    for (name, props) in mapping {
        let name = yaml_str(name);
        let option_def = if props.is_null() {
            OptionDef::default()
        } else {
            let props = props
                .as_mapping()
                .ok_or_else(|| format!("Expected a YAML mapping, got {}", yaml_type_name(props)))?;
            OptionDef {
                takes_value: optional_value(props, "takes_value")
                    .map(yaml_bool)
                    .unwrap_or(false),
                aliases: parse_string_list(optional_value(props, "aliases")),
                overrides: parse_classification(optional_value(props, "overrides"))?,
                risk: parse_risk(optional_value(props, "risk"))?,
                captures_directory: optional_value(props, "captures_directory")
                    .map(yaml_bool)
                    .unwrap_or(false),
                delegates_to: parse_delegation_config(optional_value(props, "delegates_to"))?,
            }
        };

        options.insert(name, option_def.clone());
        for alias in &option_def.aliases {
            // Mirrors database.py _parse_options: aliases are inserted as equivalent lookup keys.
            options.insert(alias.clone(), option_def.clone());
        }
    }
    Ok(options)
}

fn parse_string_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Sequence(values)) => values.iter().map(yaml_str).collect(),
        Some(value) if !value.is_null() => vec![yaml_str(value)],
        _ => Vec::new(),
    }
}

fn parse_subcommands(raw: Option<&Value>) -> Result<HashMap<String, CommandDef>, String> {
    let Some(raw) = raw else {
        return Ok(HashMap::new());
    };
    if !yaml_truthy(raw) {
        return Ok(HashMap::new());
    }
    let mapping = raw
        .as_mapping()
        .ok_or_else(|| format!("Expected a YAML mapping, got {}", yaml_type_name(raw)))?;

    let mut subcommands = HashMap::new();
    for (name, props) in mapping {
        let name = yaml_str(name);
        let props = props
            .as_mapping()
            .ok_or_else(|| format!("Expected a YAML mapping, got {}", yaml_type_name(props)))?;
        subcommands.insert(name.clone(), parse_command_def(props, &name)?);
    }
    Ok(subcommands)
}

fn parse_delegation_config(raw: Option<&Value>) -> Result<Option<DelegationConfig>, String> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    if raw.is_null() || matches!(raw, Value::Bool(false)) {
        return Ok(None);
    }
    let mapping = raw
        .as_mapping()
        .ok_or_else(|| format!("Expected a YAML mapping, got {}", yaml_type_name(raw)))?;
    let mode = parse_delegation_mode(required_value(mapping, "mode")?)?;
    Ok(Some(DelegationConfig {
        mode,
        separator: optional_value(mapping, "separator").map(yaml_str),
        terminator: optional_value(mapping, "terminator").map(yaml_str),
        flag: optional_value(mapping, "flag").map(yaml_str),
        strip_assignments: optional_value(mapping, "strip_assignments")
            .map(yaml_bool)
            .unwrap_or(false),
        skip_leading_positionals: optional_value(mapping, "skip_leading_positionals")
            .and_then(|value| yaml_str(value).parse::<usize>().ok())
            .unwrap_or(0),
        min_classification: parse_classification(optional_value(mapping, "min_classification"))?,
    }))
}

#[cfg(test)]
mod test_database {
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{load_database, CommandDatabase};
    use crate::models::{Classification, CommandDef, DelegationMode};

    static DATABASE: OnceLock<CommandDatabase> = OnceLock::new();
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn database() -> &'static CommandDatabase {
        DATABASE.get_or_init(|| load_database(None).unwrap())
    }

    fn tmp_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rust-bash-classify-test-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    mod test_database_loading {
        use super::{database, CommandDef};

        #[test]
        fn test_load_all_yaml_files_succeeds() {
            assert!(!database().is_empty());
            for name in database().names() {
                database().get_required(&name).unwrap();
            }
        }

        #[test]
        fn test_all_entries_are_command_defs() {
            for name in database().names() {
                let cmd_def: CommandDef = database().get_required(&name).unwrap();
                assert_eq!(cmd_def.command, name);
            }
        }
    }

    mod test_kubectl_yaml {
        use super::{database, Classification};

        #[test]
        fn test_kubectl_exists() {
            assert!(database().contains_key("kubectl"));
        }

        #[test]
        fn test_kubectl_global_options_context() {
            let kubectl = database().get_required("kubectl").unwrap();
            assert!(kubectl.global_options.contains_key("--context"));
            assert!(kubectl.global_options["--context"].takes_value);
        }

        #[test]
        fn test_kubectl_global_options_namespace() {
            let kubectl = database().get_required("kubectl").unwrap();
            assert!(kubectl.global_options["--namespace"].takes_value);
            assert!(kubectl.global_options["--namespace"]
                .aliases
                .contains(&"-n".to_string()));
        }

        #[test]
        fn test_kubectl_alias_resolution_n() {
            let kubectl = database().get_required("kubectl").unwrap();
            assert!(kubectl.global_options.contains_key("-n"));
            assert_eq!(
                kubectl.global_options["-n"],
                kubectl.global_options["--namespace"]
            );
        }

        #[test]
        fn test_kubectl_subcommand_get() {
            let kubectl = database().get_required("kubectl").unwrap();
            assert_eq!(
                kubectl.subcommands["get"].classification,
                Some(Classification::Readonly)
            );
        }

        #[test]
        fn test_kubectl_subcommand_delete() {
            let kubectl = database().get_required("kubectl").unwrap();
            assert_eq!(
                kubectl.subcommands["delete"].classification,
                Some(Classification::Dangerous)
            );
        }

        #[test]
        fn test_kubectl_subcommand_exec() {
            let kubectl = database().get_required("kubectl").unwrap();
            assert_eq!(
                kubectl.subcommands["exec"].classification,
                Some(Classification::Dangerous)
            );
        }
    }

    mod test_git_yaml {
        use super::{database, Classification};

        #[test]
        fn test_git_exists() {
            assert!(database().contains_key("git"));
        }

        #[test]
        fn test_git_global_option_c_captures_directory() {
            let git = database().get_required("git").unwrap();
            assert!(git.global_options["-C"].takes_value);
            assert!(git.global_options["-C"].captures_directory);
        }

        #[test]
        fn test_git_subcommand_status() {
            let git = database().get_required("git").unwrap();
            assert_eq!(
                git.subcommands["status"].classification,
                Some(Classification::Readonly)
            );
        }

        #[test]
        fn test_git_subcommand_push() {
            let git = database().get_required("git").unwrap();
            let push = &git.subcommands["push"];
            assert_eq!(push.classification, Some(Classification::ExternalEffects));
            assert_eq!(
                push.options["--force"].overrides,
                Some(Classification::Dangerous)
            );
        }
    }

    mod test_find_yaml {
        use super::{database, Classification, DelegationMode};

        #[test]
        fn test_find_base_classification() {
            assert_eq!(
                database().get_required("find").unwrap().classification,
                Some(Classification::Readonly)
            );
        }

        #[test]
        fn test_find_strict_false() {
            assert!(!database().get_required("find").unwrap().strict);
        }

        #[test]
        fn test_find_exec_delegates_to_terminated_argv() {
            let find = database().get_required("find").unwrap();
            let exec_opt = &find.options["-exec"];
            assert_eq!(
                exec_opt.delegates_to.as_ref().unwrap().mode,
                DelegationMode::TerminatedArgv
            );
            assert_eq!(
                exec_opt
                    .delegates_to
                    .as_ref()
                    .unwrap()
                    .terminator
                    .as_deref(),
                Some(";")
            );
        }

        #[test]
        fn test_find_delete_overrides_dangerous() {
            let find = database().get_required("find").unwrap();
            assert_eq!(
                find.options["-delete"].overrides,
                Some(Classification::Dangerous)
            );
        }
    }

    mod test_xargs_yaml {
        use super::{database, DelegationMode};

        #[test]
        fn test_xargs_delegates_to_rest_are_argv() {
            let xargs = database().get_required("xargs").unwrap();
            assert_eq!(
                xargs.delegates_to.unwrap().mode,
                DelegationMode::RestAreArgv
            );
        }
    }

    mod test_sh_yaml {
        use super::{database, DelegationMode};

        #[test]
        fn test_sh_delegates_to_flag_value_is_expression() {
            let sh = database().get_required("sh").unwrap();
            let delegates_to = sh.delegates_to.unwrap();
            assert_eq!(delegates_to.mode, DelegationMode::FlagValueIsExpression);
            assert_eq!(delegates_to.flag.as_deref(), Some("-c"));
        }
    }

    mod test_sudo_yaml {
        use super::{database, Classification, DelegationMode};

        #[test]
        fn test_sudo_delegates_to_rest_are_argv_with_min_classification() {
            let sudo = database().get_required("sudo").unwrap();
            let delegates_to = sudo.delegates_to.unwrap();
            assert_eq!(delegates_to.mode, DelegationMode::RestAreArgv);
            assert_eq!(
                delegates_to.min_classification,
                Some(Classification::Dangerous)
            );
        }
    }

    mod test_env_yaml {
        use super::{database, DelegationMode};

        #[test]
        fn test_env_delegates_to_rest_are_argv_with_strip_assignments() {
            let env = database().get_required("env").unwrap();
            let delegates_to = env.delegates_to.unwrap();
            assert_eq!(delegates_to.mode, DelegationMode::RestAreArgv);
            assert!(delegates_to.strip_assignments);
        }
    }

    mod test_database_error_handling {
        use super::{load_database, tmp_path, write};

        #[test]
        fn test_invalid_yaml_includes_filename() {
            let tmp_path = tmp_path();
            write(
                &tmp_path.join("bad.yaml"),
                "command: test\nclassification: INVALID_VALUE\n",
            );
            let db = load_database(Some(&tmp_path)).unwrap();
            let error = db.get_required("bad").unwrap_err().to_string();
            assert!(error.contains("bad.yaml"));
        }

        #[test]
        fn test_missing_command_key_includes_filename() {
            let tmp_path = tmp_path();
            write(
                &tmp_path.join("nocommand.yaml"),
                "classification: READONLY\n",
            );
            let db = load_database(Some(&tmp_path)).unwrap();
            let error = db.get_required("nocommand").unwrap_err().to_string();
            assert!(error.contains("nocommand.yaml"));
        }
    }

    mod test_empty_and_non_dict_yaml_files {
        use super::{load_database, tmp_path, write, Classification};

        #[test]
        fn test_empty_yaml_file_raises_on_access() {
            let tmp_path = tmp_path();
            write(&tmp_path.join("empty.yaml"), "");
            write(
                &tmp_path.join("echo.yaml"),
                "command: echo\nclassification: READONLY\n",
            );
            let db = load_database(Some(&tmp_path)).unwrap();
            assert!(db.contains_key("echo"));
            assert!(db.contains_key("empty"));
            assert_eq!(db.len(), 2);
            assert_eq!(
                db.get_required("echo").unwrap().classification,
                Some(Classification::Readonly)
            );
            assert!(db
                .get_required("empty")
                .unwrap_err()
                .to_string()
                .contains("empty.yaml"));
        }

        #[test]
        fn test_list_yaml_raises_valueerror_on_access() {
            let tmp_path = tmp_path();
            write(&tmp_path.join("badlist.yaml"), "- item1\n- item2\n");
            let db = load_database(Some(&tmp_path)).unwrap();
            assert!(db
                .get_required("badlist")
                .unwrap_err()
                .to_string()
                .contains("badlist.yaml"));
        }

        #[test]
        fn test_truthy_non_mapping_options_raises_on_access() {
            let tmp_path = tmp_path();
            write(
                &tmp_path.join("badopts.yaml"),
                "command: badopts\nclassification: READONLY\noptions:\n  - --bad\n",
            );
            let db = load_database(Some(&tmp_path)).unwrap();
            let error = db.get_required("badopts").unwrap_err().to_string();
            assert!(error.contains("badopts.yaml"));
            assert!(error.contains("Expected a YAML mapping"));
        }

        #[test]
        fn test_non_mapping_option_body_raises_on_access() {
            let tmp_path = tmp_path();
            write(
                &tmp_path.join("badoptbody.yaml"),
                "command: badoptbody\nclassification: READONLY\noptions:\n  --bad: false\n",
            );
            let db = load_database(Some(&tmp_path)).unwrap();
            let error = db.get_required("badoptbody").unwrap_err().to_string();
            assert!(error.contains("badoptbody.yaml"));
            assert!(error.contains("Expected a YAML mapping"));
        }

        #[test]
        fn test_truthy_non_mapping_subcommands_raises_on_access() {
            let tmp_path = tmp_path();
            write(
                &tmp_path.join("badsubs.yaml"),
                "command: badsubs\nclassification: READONLY\nsubcommands:\n  - status\n",
            );
            let db = load_database(Some(&tmp_path)).unwrap();
            let error = db.get_required("badsubs").unwrap_err().to_string();
            assert!(error.contains("badsubs.yaml"));
            assert!(error.contains("Expected a YAML mapping"));
        }

        #[test]
        fn test_non_mapping_subcommand_body_raises_on_access() {
            let tmp_path = tmp_path();
            write(
                &tmp_path.join("badsubbody.yaml"),
                "command: badsubbody\nclassification: READONLY\nsubcommands:\n  status: false\n",
            );
            let db = load_database(Some(&tmp_path)).unwrap();
            let error = db.get_required("badsubbody").unwrap_err().to_string();
            assert!(error.contains("badsubbody.yaml"));
            assert!(error.contains("Expected a YAML mapping"));
        }
    }

    mod test_strict_default {
        use super::database;

        #[test]
        fn test_strict_defaults_to_true() {
            let kubectl = database().get_required("kubectl").unwrap();
            assert!(!kubectl.strict);
            assert!(kubectl.subcommands["top"].strict);
        }

        #[test]
        fn test_strict_false_on_find() {
            assert!(!database().get_required("find").unwrap().strict);
        }

        #[test]
        fn test_strict_false_on_grep() {
            assert!(!database().get_required("grep").unwrap().strict);
        }
    }

    mod test_alias_of {
        use super::{load_database, tmp_path, write};

        #[test]
        fn test_alias_file_parses() {
            let tmp_path = tmp_path();
            write(
                &tmp_path.join("target.yaml"),
                "command: target\nclassification: READONLY\n",
            );
            write(
                &tmp_path.join("myalias.yaml"),
                "command: myalias\ndescription: pointer\nalias_of: target\n",
            );
            let db = load_database(Some(&tmp_path)).unwrap();
            let alias_def = db.get_required("myalias").unwrap();
            assert_eq!(alias_def.alias_of.as_deref(), Some("target"));
            assert_eq!(alias_def.command, "myalias");
            assert!(alias_def.classification.is_none());
            assert!(alias_def.subcommands.is_empty());
        }

        #[test]
        fn test_alias_file_with_subcommands_raises() {
            let tmp_path = tmp_path();
            write(
                &tmp_path.join("bad.yaml"),
                "command: bad\nalias_of: target\nsubcommands:\n  sub: {classification: READONLY}\n",
            );
            let db = load_database(Some(&tmp_path)).unwrap();
            assert!(db
                .get_required("bad")
                .unwrap_err()
                .to_string()
                .contains("subcommands"));
        }

        #[test]
        fn test_alias_file_with_classification_raises() {
            let tmp_path = tmp_path();
            write(
                &tmp_path.join("bad.yaml"),
                "command: bad\nalias_of: target\nclassification: READONLY\n",
            );
            let db = load_database(Some(&tmp_path)).unwrap();
            assert!(db
                .get_required("bad")
                .unwrap_err()
                .to_string()
                .contains("classification"));
        }

        #[test]
        fn test_alias_file_only_command_description_alias_of_is_valid() {
            let tmp_path = tmp_path();
            write(
                &tmp_path.join("target.yaml"),
                "command: target\nclassification: READONLY\n",
            );
            write(
                &tmp_path.join("myalias.yaml"),
                "command: myalias\ndescription: \"some desc\"\nalias_of: target\n",
            );
            let db = load_database(Some(&tmp_path)).unwrap();
            assert_eq!(
                db.get_required("myalias").unwrap().alias_of.as_deref(),
                Some("target")
            );
        }

        #[test]
        fn test_alias_with_missing_target_loads_as_raw_alias() {
            let tmp_path = tmp_path();
            write(
                &tmp_path.join("myalias.yaml"),
                "command: myalias\nalias_of: missing\n",
            );
            let db = load_database(Some(&tmp_path)).unwrap();
            let alias_def = db.get_required("myalias").unwrap();
            assert_eq!(alias_def.alias_of.as_deref(), Some("missing"));
        }
    }

    mod test_user_commands_dir {
        use super::{load_database, tmp_path, write, Classification, ENV_LOCK};
        use crate::database::{get_default_commands_dir, get_user_commands_dir};

        #[test]
        fn test_user_override_replaces_builtin() {
            let _guard = ENV_LOCK.lock().unwrap();
            let tmp_path = tmp_path();
            let config = tmp_path.join("config");
            write(
                &config.join("commands/grep.yaml"),
                "command: grep\nclassification: EXTERNAL_EFFECTS\nstrict: false\n",
            );
            let old = std::env::var_os("BASH_CLASSIFY_CONFIG_DIR");
            std::env::set_var("BASH_CLASSIFY_CONFIG_DIR", &config);
            let db = load_database(None).unwrap();
            restore_env(old);
            assert_eq!(
                db.get_required("grep").unwrap().classification,
                Some(Classification::ExternalEffects)
            );
        }

        #[test]
        fn test_user_adds_new_command() {
            let _guard = ENV_LOCK.lock().unwrap();
            let tmp_path = tmp_path();
            let config = tmp_path.join("config");
            write(
                &config.join("commands/mycustomtool.yaml"),
                "command: mycustomtool\nclassification: READONLY\nstrict: false\n",
            );
            let old = std::env::var_os("BASH_CLASSIFY_CONFIG_DIR");
            std::env::set_var("BASH_CLASSIFY_CONFIG_DIR", &config);
            let db = load_database(None).unwrap();
            restore_env(old);
            assert!(db.contains_key("mycustomtool"));
            assert_eq!(
                db.get_required("mycustomtool").unwrap().classification,
                Some(Classification::Readonly)
            );
        }

        #[test]
        fn test_missing_user_dir_is_silently_skipped() {
            let _guard = ENV_LOCK.lock().unwrap();
            let tmp_path = tmp_path();
            let old = std::env::var_os("BASH_CLASSIFY_CONFIG_DIR");
            std::env::set_var("BASH_CLASSIFY_CONFIG_DIR", tmp_path.join("nonexistent"));
            let db = load_database(None).unwrap();
            restore_env(old);
            assert!(!db.is_empty());
        }

        #[test]
        fn test_explicit_commands_dir_skips_user_overrides() {
            let _guard = ENV_LOCK.lock().unwrap();
            let tmp_path = tmp_path();
            let config = tmp_path.join("config");
            write(
                &config.join("commands/grep.yaml"),
                "command: grep\nclassification: DANGEROUS\nstrict: false\n",
            );
            let old = std::env::var_os("BASH_CLASSIFY_CONFIG_DIR");
            std::env::set_var("BASH_CLASSIFY_CONFIG_DIR", &config);
            let db = load_database(Some(&get_default_commands_dir())).unwrap();
            restore_env(old);
            assert_eq!(
                db.get_required("grep").unwrap().classification,
                Some(Classification::Readonly)
            );
        }

        #[test]
        fn test_empty_config_dir_falls_back_to_home_config() {
            let _guard = ENV_LOCK.lock().unwrap();
            let tmp_path = tmp_path();
            let old_config = std::env::var_os("BASH_CLASSIFY_CONFIG_DIR");
            let old_home = std::env::var_os("HOME");
            std::env::set_var("BASH_CLASSIFY_CONFIG_DIR", "");
            std::env::set_var("HOME", &tmp_path);
            let user_dir = get_user_commands_dir();
            restore_env(old_config);
            restore_named_env("HOME", old_home);
            assert_eq!(user_dir, tmp_path.join(".config/bash-classify/commands"));
        }

        fn restore_env(old: Option<std::ffi::OsString>) {
            if let Some(old) = old {
                std::env::set_var("BASH_CLASSIFY_CONFIG_DIR", old);
            } else {
                std::env::remove_var("BASH_CLASSIFY_CONFIG_DIR");
            }
        }

        fn restore_named_env(name: &str, old: Option<std::ffi::OsString>) {
            if let Some(old) = old {
                std::env::set_var(name, old);
            } else {
                std::env::remove_var(name);
            }
        }
    }
}
