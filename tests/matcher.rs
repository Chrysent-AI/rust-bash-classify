// SPDX-FileCopyrightText: 2026 Chrysent LLC
// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::sync::OnceLock;

use bash_classify::database::load_database;
use bash_classify::matcher::{
    handle_option_delegation, is_terminator, match_command, strip_quotes,
};
use bash_classify::models::{
    Classification, CommandDef, CommandInvocation, DelegationConfig, DelegationMode, OptionDef,
    Risk, SubcommandMode,
};
use bash_classify::{parse_expression, CommandDatabase};

fn db() -> &'static CommandDatabase {
    static DB: OnceLock<CommandDatabase> = OnceLock::new();
    DB.get_or_init(|| load_database(None).unwrap())
}

fn inv(argv: &[&str]) -> CommandInvocation {
    CommandInvocation {
        argv: argv.iter().map(|s| (*s).to_string()).collect(),
        redirects: vec![],
        position_in_pipeline: 0,
        pipeline_length: 1,
        context: "toplevel".to_string(),
        operator_before: None,
        is_background: false,
    }
}

fn opt() -> OptionDef {
    OptionDef::default()
}

fn cmd(name: &str, classification: Option<Classification>) -> CommandDef {
    let mut command = CommandDef::new(name);
    command.classification = classification;
    command
}

mod test_basic_matching {
    use super::*;

    #[test]
    fn test_ls_readonly() {
        let result = match_command(&inv(&["ls"]), db()).unwrap();
        assert_eq!(result.classification, Classification::Readonly);
        assert_eq!(result.matched_rule.as_deref(), Some("ls"));
    }

    #[test]
    fn test_cat_readonly() {
        let result = match_command(&inv(&["cat", "/etc/hosts"]), db()).unwrap();
        assert_eq!(result.classification, Classification::Readonly);
        assert_eq!(result.matched_rule.as_deref(), Some("cat"));
    }

    #[test]
    fn test_rm_classification() {
        let result = match_command(&inv(&["rm", "-rf", "/tmp/foo"]), db()).unwrap();
        assert_eq!(result.classification, Classification::Dangerous);
        assert_eq!(result.matched_rule.as_deref(), Some("rm"));
    }

    #[test]
    fn test_unknown_command() {
        let result = match_command(&inv(&["someunknowncommand"]), db()).unwrap();
        assert_eq!(result.classification, Classification::Unknown);
        assert_eq!(result.matched_rule, None);
    }
}

mod test_global_option_stripping {
    use super::*;

    #[test]
    fn test_kubectl_context_equals_form() {
        let result =
            match_command(&inv(&["kubectl", "--context=prod", "get", "pods"]), db()).unwrap();
        assert_eq!(result.classification, Classification::Readonly);
        assert_eq!(result.matched_rule.as_deref(), Some("kubectl.get"));
        assert!(result
            .ignored_options
            .unwrap()
            .contains(&"--context=prod".to_string()));
    }

    #[test]
    fn test_kubectl_context_separated_form() {
        let result =
            match_command(&inv(&["kubectl", "--context", "prod", "get", "pods"]), db()).unwrap();
        assert_eq!(result.classification, Classification::Readonly);
        assert_eq!(result.matched_rule.as_deref(), Some("kubectl.get"));
        let ignored = result.ignored_options.unwrap();
        assert!(ignored.contains(&"--context".to_string()));
        assert!(ignored.contains(&"prod".to_string()));
    }

    #[test]
    fn test_git_c_captures_directory() {
        let result = match_command(&inv(&["git", "-C", "/some/path", "status"]), db()).unwrap();
        assert_eq!(result.classification, Classification::Readonly);
        assert_eq!(result.matched_rule.as_deref(), Some("git.status"));
        let ignored = result.ignored_options.unwrap();
        assert!(ignored.contains(&"-C".to_string()));
        assert!(ignored.contains(&"/some/path".to_string()));
    }
}

mod test_subcommand_matching {
    use super::*;
    #[test]
    fn test_kubectl_get() {
        let r = match_command(&inv(&["kubectl", "get", "pods"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.matched_rule.as_deref(), Some("kubectl.get"));
    }
    #[test]
    fn test_kubectl_delete() {
        let r = match_command(&inv(&["kubectl", "delete", "pod", "mypod"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
        assert_eq!(r.matched_rule.as_deref(), Some("kubectl.delete"));
    }
    #[test]
    fn test_kubectl_rollout_status() {
        let r = match_command(&inv(&["kubectl", "rollout", "status", "deploy/app"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.matched_rule.as_deref(), Some("kubectl.rollout.status"));
    }
    #[test]
    fn test_kubectl_rollout_undo() {
        let r = match_command(&inv(&["kubectl", "rollout", "undo", "deploy/app"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
    }
    #[test]
    fn test_git_stash_list() {
        let r = match_command(&inv(&["git", "stash", "list"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.matched_rule.as_deref(), Some("git.stash.list"));
    }
    #[test]
    fn test_git_stash_drop() {
        let r = match_command(&inv(&["git", "stash", "drop"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
    }
}

mod test_option_overrides {
    use super::*;
    #[test]
    fn test_git_push_force() {
        assert_eq!(
            match_command(&inv(&["git", "push", "--force", "origin", "main"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_git_push_f_alias() {
        assert_eq!(
            match_command(&inv(&["git", "push", "-f", "origin"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_git_push_force_with_lease() {
        assert_eq!(
            match_command(&inv(&["git", "push", "--force-with-lease"]), db())
                .unwrap()
                .classification,
            Classification::ExternalEffects
        );
    }
    #[test]
    fn test_git_branch_d_dangerous() {
        assert_eq!(
            match_command(&inv(&["git", "branch", "-D", "feature"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_git_reset_hard() {
        assert_eq!(
            match_command(&inv(&["git", "reset", "--hard"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_kubectl_apply_dry_run() {
        assert_eq!(
            match_command(
                &inv(&["kubectl", "apply", "--dry-run", "client", "-f", "file.yaml"]),
                db()
            )
            .unwrap()
            .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_kubectl_apply_dry_run_no_unknown() {
        assert_eq!(
            match_command(&inv(&["kubectl", "apply", "--dry-run", "client"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_kubectl_delete_dry_run() {
        assert_eq!(
            match_command(
                &inv(&["kubectl", "delete", "--dry-run", "client", "pod", "mypod"]),
                db()
            )
            .unwrap()
            .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_git_push_force_overrides_to_dangerous() {
        assert_eq!(
            match_command(&inv(&["git", "push", "--force", "origin", "main"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
}

mod test_strict_mode {
    use super::*;
    #[test]
    fn test_kubectl_top_unknown_flag() {
        assert_eq!(
            match_command(&inv(&["kubectl", "top", "--unknown-flag"]), db())
                .unwrap()
                .classification,
            Classification::Unknown
        );
    }
    #[test]
    fn test_grep_unknown_flag_stays_readonly() {
        assert_eq!(
            match_command(&inv(&["grep", "--unknown-flag", "pattern"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_find_unknown_predicate_stays_readonly() {
        assert_eq!(
            match_command(&inv(&["find", ".", "--some-unknown-pred"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
}

mod test_delegation {
    use super::*;
    #[test]
    fn test_xargs_delegates_inner_command() {
        let r = match_command(&inv(&["xargs", "grep", "-r", "pattern"]), db()).unwrap();
        assert_eq!(r.inner_commands.len(), 1);
        assert_eq!(r.inner_commands[0].argv, ["grep", "-r", "pattern"]);
        assert_eq!(r.inner_commands[0].delegation_mode, "rest_are_argv");
    }
    #[test]
    fn test_sudo_delegates_with_min_classification() {
        let r = match_command(&inv(&["sudo", "rm", "-rf", "/"]), db()).unwrap();
        assert_eq!(r.inner_commands.len(), 1);
        assert_eq!(r.inner_commands[0].argv, ["rm", "-rf", "/"]);
        assert_eq!(
            r.inner_commands[0].classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_nice_delegates_inner_command() {
        let r = match_command(&inv(&["nice", "-n", "10", "make"]), db()).unwrap();
        assert_eq!(r.inner_commands.len(), 1);
        assert_eq!(r.inner_commands[0].argv, ["make"]);
    }
    #[test]
    fn test_kubectl_exec_after_separator() {
        let r = match_command(
            &inv(&[
                "kubectl",
                "exec",
                "-it",
                "my-pod",
                "--",
                "cat",
                "/etc/config",
            ]),
            db(),
        )
        .unwrap();
        assert_eq!(r.inner_commands.len(), 1);
        assert_eq!(r.inner_commands[0].argv, ["cat", "/etc/config"]);
        assert_eq!(r.inner_commands[0].delegation_mode, "after_separator");
        assert_eq!(r.inner_commands[0].delegation_source, "--");
    }
    #[test]
    fn test_find_exec_terminated_argv() {
        let r = match_command(
            &inv(&[
                "find", ".", "-name", "*.tmp", "-exec", "rm", "-f", "{}", ";",
            ]),
            db(),
        )
        .unwrap();
        assert_eq!(r.inner_commands.len(), 1);
        assert_eq!(r.inner_commands[0].argv, ["rm", "-f"]);
        assert_eq!(r.inner_commands[0].delegation_mode, "terminated_argv");
    }
    #[test]
    fn test_sh_c_parses_expression() {
        let r = match_command(&inv(&["sh", "-c", "ls /tmp | grep log"]), db()).unwrap();
        assert!(r.inner_commands.len() >= 2);
        let cmds: Vec<_> = r
            .inner_commands
            .iter()
            .map(|ic| ic.command.clone())
            .collect();
        assert!(cmds.contains(&vec!["ls".to_string()]));
        assert!(cmds.contains(&vec!["grep".to_string()]));
    }
    #[test]
    fn test_env_strip_assignments() {
        let r = match_command(
            &inv(&["env", "FOO=bar", "BAZ=qux", "some_command", "arg"]),
            db(),
        )
        .unwrap();
        assert_eq!(r.inner_commands.len(), 1);
        assert_eq!(r.inner_commands[0].argv, ["some_command", "arg"]);
    }
}

mod test_skip_leading_positionals {
    use super::*;
    #[test]
    fn test_timeout_duration_then_readonly_inner() {
        let r = match_command(&inv(&["timeout", "30", "cat", "/tmp/x"]), db()).unwrap();
        assert_eq!(r.inner_commands.len(), 1);
        assert_eq!(r.inner_commands[0].argv, ["cat", "/tmp/x"]);
        assert_eq!(r.inner_commands[0].command, ["cat"]);
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_timeout_duration_then_dangerous_inner_elevates() {
        let r = match_command(&inv(&["timeout", "30", "rm", "-rf", "/"]), db()).unwrap();
        assert_eq!(r.inner_commands.len(), 1);
        assert_eq!(r.inner_commands[0].argv, ["rm", "-rf", "/"]);
        assert_eq!(r.classification, Classification::Dangerous);
        assert_eq!(r.risk, Risk::High);
    }
    #[test]
    fn test_timeout_flag_then_duration_then_inner() {
        let r = match_command(
            &inv(&["timeout", "-s", "KILL", "30", "tail", "-f", "/var/log/x"]),
            db(),
        )
        .unwrap();
        assert_eq!(r.inner_commands.len(), 1);
        assert_eq!(r.inner_commands[0].argv, ["tail", "-f", "/var/log/x"]);
        assert_eq!(r.inner_commands[0].command, ["tail"]);
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_timeout_only_duration_no_inner() {
        let r = match_command(&inv(&["timeout", "30"]), db()).unwrap();
        assert_eq!(r.inner_commands.len(), 0);
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_timeout_no_args() {
        let r = match_command(&inv(&["timeout"]), db()).unwrap();
        assert_eq!(r.inner_commands.len(), 0);
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::Low);
    }
}

mod test_combined_short_flags {
    use super::*;
    #[test]
    fn test_combined_boolean_flags() {
        assert_eq!(
            match_command(&inv(&["ssh", "-Nvx", "host"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_combined_flag_middle_takes_value() {
        let r = match_command(&inv(&["ssh", "-vp22", "host"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
        assert!(r
            .remaining_options
            .as_ref()
            .is_none_or(|opts| !opts.contains(&"-vp22".to_string())));
    }
    #[test]
    fn test_combined_flag_last_takes_value_separate() {
        let r = match_command(&inv(&["ssh", "-vp", "22", "host"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
        assert!(r
            .remaining_options
            .as_ref()
            .is_none_or(|opts| !opts.contains(&"-vp".to_string())));
    }
}

mod test_misc_builtin_and_helpers {
    use super::*;
    #[test]
    fn test_ssh_no_inner_commands() {
        let r = match_command(&inv(&["ssh", "user@host", "ls", "-la"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
        assert_eq!(r.inner_commands.len(), 0);
    }
    #[test]
    fn test_kubectl_apply_dry_run_equals_form() {
        assert_eq!(
            match_command(
                &inv(&[
                    "kubectl",
                    "apply",
                    "--dry-run=client",
                    "-f",
                    "manifest.yaml"
                ]),
                db()
            )
            .unwrap()
            .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_kubectl_get_output() {
        assert_eq!(
            match_command(&inv(&["kubectl", "get", "pods", "-o", "json"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_kubectl_delete_filename() {
        assert_eq!(
            match_command(&inv(&["kubectl", "delete", "-f", "manifest.yaml"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_cd_readonly() {
        assert_eq!(
            match_command(&inv(&["cd", "/tmp"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_pushd_readonly() {
        assert_eq!(
            match_command(&inv(&["pushd", "/var"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_eval_dangerous() {
        assert_eq!(
            match_command(&inv(&["eval", "some code"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_source_dangerous() {
        assert_eq!(
            match_command(&inv(&["source", "script.sh"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_dot_dangerous() {
        assert_eq!(
            match_command(&inv(&[".", "script.sh"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_exec_dangerous() {
        assert_eq!(
            match_command(&inv(&["exec", "some_binary"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_empty_argv_unknown() {
        let r = match_command(&inv(&[]), db()).unwrap();
        assert_eq!(r.classification, Classification::Unknown);
        assert_eq!(r.command, Vec::<String>::new());
    }
    #[test]
    fn test_test_builtin_readonly() {
        assert_eq!(
            match_command(&inv(&["test", "-f", "file"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_bracket_readonly() {
        assert_eq!(
            match_command(&inv(&["[", "-f", "file", "]"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_double_bracket_readonly() {
        assert_eq!(
            match_command(&inv(&["[[", "-f", "file", "]]"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_popd_readonly() {
        assert_eq!(
            match_command(&inv(&["popd"]), db()).unwrap().classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_highest_override_wins() {
        assert_eq!(
            match_command(
                &inv(&["git", "push", "--force-with-lease", "--force"]),
                db()
            )
            .unwrap()
            .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_exact_match() {
        assert!(is_terminator(";", Some(";")));
    }

    #[test]
    fn test_backslash_form() {
        assert!(is_terminator("\\;", Some(";")));
    }

    #[test]
    fn test_no_match() {
        assert!(!is_terminator("+", Some(";")));
    }

    #[test]
    fn test_none_terminator() {
        assert!(!is_terminator(";", None));
    }
    #[test]
    fn test_xargs_no_inner_command() {
        assert_eq!(
            match_command(&inv(&["xargs"]), db())
                .unwrap()
                .inner_commands,
            vec![]
        );
    }
    #[test]
    fn test_kubectl_exec_no_inner_after_separator() {
        assert_eq!(
            match_command(&inv(&["kubectl", "exec", "my-pod", "--"]), db())
                .unwrap()
                .inner_commands,
            vec![]
        );
    }
    #[test]
    fn test_sh_without_c_flag() {
        assert_eq!(
            match_command(&inv(&["sh", "script.sh"]), db())
                .unwrap()
                .inner_commands,
            vec![]
        );
    }
    #[test]
    fn test_single_quoted_value() {
        assert_eq!(strip_quotes("'hello world'"), "hello world");
    }

    #[test]
    fn test_double_quoted_value() {
        assert_eq!(strip_quotes("\"hello world\""), "hello world");
    }

    #[test]
    fn test_unquoted_value() {
        assert_eq!(strip_quotes("hello"), "hello");
    }
}

mod test_default_classification_when_not_set {
    use super::*;
    #[test]
    fn test_command_without_classification_defaults_to_readonly() {
        let r = match_command(&inv(&["apt"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.matched_rule.as_deref(), Some("apt"));
    }
    #[test]
    fn test_apt_help_readonly() {
        assert_eq!(
            match_command(&inv(&["apt", "--help"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_apt_search_readonly() {
        assert_eq!(
            match_command(&inv(&["apt", "search", "vim"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_apt_install_dangerous() {
        assert_eq!(
            match_command(&inv(&["apt", "install", "vim"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_docker_bare_readonly() {
        assert_eq!(
            match_command(&inv(&["docker"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_npm_bare_readonly() {
        assert_eq!(
            match_command(&inv(&["npm"]), db()).unwrap().classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_helm_bare_readonly() {
        assert_eq!(
            match_command(&inv(&["helm"]), db()).unwrap().classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_systemctl_bare_dangerous() {
        assert_eq!(
            match_command(&inv(&["systemctl"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_systemctl_start_dangerous() {
        assert_eq!(
            match_command(&inv(&["systemctl", "start", "nginx"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_docker_system_prune_dangerous() {
        assert_eq!(
            match_command(&inv(&["docker", "system", "prune"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
}

mod test_remaining_and_combined_unknown {
    use super::*;
    #[test]
    fn test_remaining_options_in_non_strict_mode() {
        let r = match_command(&inv(&["grep", "--some-unknown", "pattern"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert!(r
            .remaining_options
            .unwrap()
            .contains(&"--some-unknown".to_string()));
    }
    #[test]
    fn test_combined_known_and_unknown_strict_mode() {
        let r = match_command(&inv(&["date", "-Z"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Unknown);
        assert!(r.remaining_options.unwrap().contains(&"-Z".to_string()));
    }
    #[test]
    fn test_combined_known_first_unknown_second_strict() {
        let r = match_command(&inv(&["sh", "-eZ"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
        assert!(r.remaining_options.is_none());
    }
    #[test]
    fn test_combined_known_and_unknown_nonstrict_mode() {
        assert_eq!(
            match_command(&inv(&["grep", "-rZ", "pattern"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_double_dash_stops_option_parsing() {
        assert_eq!(
            match_command(&inv(&["grep", "--", "-pattern", "file"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_non_terminated_argv_returns_empty() {
        let config = DelegationConfig {
            mode: DelegationMode::RestAreArgv,
            separator: None,
            terminator: None,
            flag: None,
            strip_assignments: false,
            skip_leading_positionals: 0,
            min_classification: None,
        };
        let results =
            handle_option_delegation("-x", &config, &["ls".to_string(), "-la".to_string()], db())
                .unwrap();
        assert_eq!(results, vec![]);
    }
    #[test]
    fn test_force_then_force_with_lease() {
        assert_eq!(
            match_command(
                &inv(&["git", "push", "--force", "--force-with-lease"]),
                db()
            )
            .unwrap()
            .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_force_with_lease_then_force() {
        assert_eq!(
            match_command(
                &inv(&["git", "push", "--force-with-lease", "--force"]),
                db()
            )
            .unwrap()
            .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_sh_c_equals_form() {
        let r = match_command(&inv(&["sh", "-c=ls /tmp"]), db()).unwrap();
        if !r.inner_commands.is_empty() {
            let cmds: Vec<_> = r
                .inner_commands
                .iter()
                .map(|ic| ic.command.clone())
                .collect();
            assert!(cmds.contains(&vec!["ls".to_string()]));
        }
    }
}

mod test_delegation_floor {
    use super::*;
    #[test]
    fn test_sh_c_readonly_inner() {
        let r = match_command(&inv(&["sh", "-c", "cat /tmp/x"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_bash_c_readonly_inner() {
        let r = match_command(&inv(&["bash", "-c", "cat /tmp/x"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_sh_c_dangerous_inner_elevates() {
        let r = match_command(&inv(&["sh", "-c", "rm -rf /"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
        assert_eq!(r.risk, Risk::High);
    }
    #[test]
    fn test_sh_c_local_effects_inner() {
        let r = match_command(&inv(&["sh", "-c", "git add ."]), db()).unwrap();
        assert_eq!(r.classification, Classification::LocalEffects);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_sh_without_c_preserves_base() {
        let r = match_command(&inv(&["sh"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
        assert_eq!(r.risk, Risk::High);
    }
    #[test]
    fn test_sh_c_empty_preserves_base() {
        let r = match_command(&inv(&["sh", "-c", ""]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
        assert_eq!(r.risk, Risk::High);
    }
    #[test]
    fn test_sudo_cat_unchanged() {
        assert_eq!(
            match_command(&inv(&["sudo", "cat", "/tmp/x"]), db())
                .unwrap()
                .classification,
            Classification::Dangerous
        );
    }
    #[test]
    fn test_sh_c_unknown_option_stays_dangerous() {
        let r = match_command(&inv(&["sh", "--bogus", "-c", "cat /tmp/x"]), db()).unwrap();
        assert_ne!(r.classification, Classification::Readonly);
    }
    #[test]
    fn test_uv_run_readonly_inner() {
        let r = match_command(&inv(&["uv", "run", "cat", "/tmp/x"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_uv_run_dangerous_inner_elevates() {
        let r = match_command(&inv(&["uv", "run", "rm", "-rf", "/tmp/x"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
        assert_eq!(r.risk, Risk::High);
    }
    #[test]
    fn test_npx_readonly_inner() {
        let r = match_command(&inv(&["npx", "cat", "/tmp/x"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_zsh_c_readonly_inner() {
        let r = match_command(&inv(&["zsh", "-c", "cat /tmp/x"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_nohup_min_classification() {
        assert_eq!(
            match_command(&inv(&["nohup", "cat", "/tmp/x"]), db())
                .unwrap()
                .classification,
            Classification::ExternalEffects
        );
    }
    #[test]
    fn test_kubectl_exec_floor() {
        assert_eq!(
            match_command(&inv(&["kubectl", "exec", "pod", "--", "ls"]), db())
                .unwrap()
                .classification,
            Classification::ExternalEffects
        );
    }
    #[test]
    fn test_kubectl_exec_dangerous_inner_elevates() {
        let r = match_command(
            &inv(&["kubectl", "exec", "pod", "--", "rm", "-rf", "/"]),
            db(),
        )
        .unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
        assert_eq!(r.risk, Risk::High);
    }
}

mod test_sh_xargs_integration {
    use super::*;

    #[test]
    fn test_xargs_sh_c_readonly() {
        let (invocations, _warnings) = parse_expression("xargs -I{} sh -c 'cat {}'").unwrap();
        let mut command_results = Vec::new();
        for invocation in invocations {
            command_results.push(match_command(&invocation, db()).unwrap());
        }
        let classification = command_results
            .iter()
            .map(|result| result.classification)
            .max_by_key(|classification| classification.severity())
            .unwrap_or(Classification::Readonly);
        let risk = command_results
            .iter()
            .map(|result| result.risk)
            .max_by_key(|risk| risk.severity())
            .unwrap_or(Risk::Low);
        assert_eq!(classification, Classification::Readonly);
        assert_eq!(risk, Risk::Low);
    }
}

mod test_global_option_overrides {
    use super::*;
    #[test]
    fn test_help_before_subcommand() {
        assert_eq!(
            match_command(&inv(&["kubectl", "--help"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_help_after_subcommand() {
        assert_eq!(
            match_command(&inv(&["kubectl", "apply", "--help"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_help_on_dangerous_command() {
        assert_eq!(
            match_command(&inv(&["kubectl", "delete", "--help"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_h_short_flag() {
        assert_eq!(
            match_command(&inv(&["git", "-h"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_h_after_subcommand() {
        assert_eq!(
            match_command(&inv(&["git", "push", "-h"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_help_does_not_affect_normal_usage() {
        assert_eq!(
            match_command(&inv(&["kubectl", "apply", "-f", "manifest.yaml"]), db())
                .unwrap()
                .classification,
            Classification::ExternalEffects
        );
    }
    #[test]
    fn test_help_on_command_without_global_options() {
        assert_eq!(
            match_command(&inv(&["rm", "--help"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_help_on_docker_run() {
        assert_eq!(
            match_command(&inv(&["docker", "run", "--help"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
    #[test]
    fn test_help_on_terraform_apply() {
        assert_eq!(
            match_command(&inv(&["terraform", "apply", "--help"]), db())
                .unwrap()
                .classification,
            Classification::Readonly
        );
    }
}

mod test_risk {
    use super::*;
    #[test]
    fn test_readonly_command_has_low_risk() {
        let r = match_command(&inv(&["ls"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_local_effects_command_has_medium_risk() {
        let r = match_command(&inv(&["git", "merge", "main"]), db()).unwrap();
        assert_eq!(r.classification, Classification::LocalEffects);
        assert_eq!(r.risk, Risk::Medium);
    }
    #[test]
    fn test_local_effects_command_with_explicit_low_risk() {
        let r = match_command(&inv(&["git", "commit", "-m", "test"]), db()).unwrap();
        assert_eq!(r.classification, Classification::LocalEffects);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_external_effects_command_has_medium_risk() {
        let r = match_command(&inv(&["git", "push"]), db()).unwrap();
        assert_eq!(r.classification, Classification::ExternalEffects);
        assert_eq!(r.risk, Risk::Medium);
    }
    #[test]
    fn test_dangerous_command_has_high_risk() {
        let r = match_command(&inv(&["rm", "-rf", "/tmp/foo"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
        assert_eq!(r.risk, Risk::High);
    }
    #[test]
    fn test_unknown_command_has_high_risk() {
        let r = match_command(&inv(&["someunknowncommand"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Unknown);
        assert_eq!(r.risk, Risk::High);
    }
    #[test]
    fn test_empty_argv_has_high_risk() {
        let r = match_command(&inv(&[]), db()).unwrap();
        assert_eq!(r.classification, Classification::Unknown);
        assert_eq!(r.risk, Risk::High);
    }
    #[test]
    fn test_option_classification_override_derives_risk() {
        let r = match_command(&inv(&["git", "push", "--force"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Dangerous);
        assert_eq!(r.risk, Risk::High);
    }
    #[test]
    fn test_option_dry_run_override_derives_low_risk() {
        let r = match_command(&inv(&["kubectl", "apply", "--dry-run", "client"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_strict_mode_unknown_option_high_risk() {
        let r = match_command(&inv(&["kubectl", "top", "--unknown-flag"]), db()).unwrap();
        assert_eq!(r.classification, Classification::Unknown);
        assert_eq!(r.risk, Risk::High);
    }
    #[test]
    fn test_explicit_risk_in_command_def() {
        let mut c = cmd("mycmd", Some(Classification::LocalEffects));
        c.risk = Some(Risk::Low);
        let database = HashMap::from([("mycmd".to_string(), c)]);
        let r = match_command(&inv(&["mycmd"]), &database).unwrap();
        assert_eq!(r.classification, Classification::LocalEffects);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_explicit_risk_on_option() {
        let mut c = cmd("mycmd", Some(Classification::Readonly));
        let mut od = opt();
        od.risk = Some(Risk::High);
        c.options.insert("--dangerous-flag".to_string(), od);
        let database = HashMap::from([("mycmd".to_string(), c)]);
        let r = match_command(&inv(&["mycmd", "--dangerous-flag"]), &database).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::High);
    }
    #[test]
    fn test_option_with_both_overrides_and_risk() {
        let mut c = cmd("mycmd", Some(Classification::Readonly));
        let mut od = opt();
        od.overrides = Some(Classification::LocalEffects);
        od.risk = Some(Risk::High);
        c.options.insert("--elevate".to_string(), od);
        let database = HashMap::from([("mycmd".to_string(), c)]);
        let r = match_command(&inv(&["mycmd", "--elevate"]), &database).unwrap();
        assert_eq!(r.classification, Classification::LocalEffects);
        assert_eq!(r.risk, Risk::High);
    }
    #[test]
    fn test_inner_command_elevates_parent_risk() {
        let r = match_command(&inv(&["sudo", "rm", "-rf", "/"]), db()).unwrap();
        assert_eq!(r.risk, Risk::High);
        assert_eq!(r.inner_commands.len(), 1);
        assert_eq!(r.inner_commands[0].risk, Risk::High);
    }
    #[test]
    fn test_cd_low_risk() {
        assert_eq!(
            match_command(&inv(&["cd", "/tmp"]), db()).unwrap().risk,
            Risk::Low
        );
    }
    #[test]
    fn test_eval_high_risk() {
        assert_eq!(
            match_command(&inv(&["eval", "code"]), db()).unwrap().risk,
            Risk::High
        );
    }
    #[test]
    fn test_test_builtin_low_risk() {
        assert_eq!(
            match_command(&inv(&["test", "-f", "file"]), db())
                .unwrap()
                .risk,
            Risk::Low
        );
    }
}

mod test_match_all_subcommand_mode {
    use super::*;

    fn make_match_all_command() -> CommandDef {
        let mut c = cmd("builder", Some(Classification::LocalEffects));
        c.subcommand_mode = SubcommandMode::MatchAll;
        c.strict = false;
        let mut clean = cmd("clean", Some(Classification::LocalEffects));
        clean.risk = Some(Risk::Low);
        let mut compile = cmd("compile", Some(Classification::LocalEffects));
        compile.risk = Some(Risk::Low);
        let mut test = cmd("test", Some(Classification::LocalEffects));
        test.risk = Some(Risk::Low);
        let deploy = cmd("deploy", Some(Classification::ExternalEffects));
        c.subcommands.insert("clean".to_string(), clean);
        c.subcommands.insert("compile".to_string(), compile);
        c.subcommands.insert("test".to_string(), test);
        c.subcommands.insert("deploy".to_string(), deploy);
        c.options.insert("-B".to_string(), opt());
        let mut dry = opt();
        dry.overrides = Some(Classification::Readonly);
        c.options.insert("--dry-run".to_string(), dry);
        c
    }

    fn database() -> HashMap<String, CommandDef> {
        HashMap::from([("builder".to_string(), make_match_all_command())])
    }

    #[test]
    fn test_single_known_goal() {
        let r = match_command(&inv(&["builder", "clean"]), &database()).unwrap();
        assert_eq!(r.classification, Classification::LocalEffects);
        assert_eq!(r.risk, Risk::Low);
        assert_eq!(r.matched_rule.as_deref(), Some("builder.clean"));
    }
    #[test]
    fn test_multiple_known_goals_all_low() {
        let r = match_command(&inv(&["builder", "clean", "compile", "test"]), &database()).unwrap();
        assert_eq!(r.classification, Classification::LocalEffects);
        assert_eq!(r.risk, Risk::Low);
        assert_eq!(
            r.matched_rule.as_deref(),
            Some("builder.clean.compile.test")
        );
    }
    #[test]
    fn test_multiple_goals_max_classification() {
        let r = match_command(&inv(&["builder", "clean", "deploy"]), &database()).unwrap();
        assert_eq!(r.classification, Classification::ExternalEffects);
        assert_eq!(r.risk, Risk::Medium);
    }
    #[test]
    fn test_unrecognized_goal_uses_base() {
        let r = match_command(
            &inv(&["builder", "clean", "custom-plugin:goal"]),
            &database(),
        )
        .unwrap();
        assert_eq!(r.classification, Classification::LocalEffects);
        assert_eq!(r.risk, Risk::Medium);
    }
    #[test]
    fn test_no_goals_uses_base() {
        let r = match_command(&inv(&["builder"]), &database()).unwrap();
        assert_eq!(r.classification, Classification::LocalEffects);
        assert_eq!(r.risk, Risk::Medium);
    }
    #[test]
    fn test_options_still_processed() {
        let r = match_command(&inv(&["builder", "-B", "clean", "compile"]), &database()).unwrap();
        assert_eq!(r.classification, Classification::LocalEffects);
        assert_eq!(r.risk, Risk::Low);
    }
    #[test]
    fn test_option_override_works() {
        let r = match_command(&inv(&["builder", "--dry-run", "clean"]), &database()).unwrap();
        assert_eq!(r.classification, Classification::Readonly);
        assert_eq!(r.risk, Risk::Low);
    }
}
