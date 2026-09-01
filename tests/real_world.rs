use std::sync::OnceLock;

use bash_classify::database::load_database;
use bash_classify::models::Classification::*;
use bash_classify::models::Risk::*;
use bash_classify::{classify_expression, CommandDatabase};

fn db() -> &'static CommandDatabase {
    static DB: OnceLock<CommandDatabase> = OnceLock::new();
    DB.get_or_init(|| load_database(None).unwrap())
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
class_case!(test_git_status, "git status", Readonly);
class_case!(test_git_status_short, "git status --short", Readonly);
class_case!(test_git_diff_stat, "git diff --stat", Readonly);
class_case!(test_git_diff_cached, "git diff --cached", Readonly);
class_case!(
    test_git_diff_cached_stat,
    "git diff --cached --stat",
    Readonly
);
class_case!(test_git_log_oneline, "git log --oneline -5", Readonly);
class_case!(
    test_git_log_range,
    "git log master..HEAD --oneline",
    Readonly
);
class_case!(test_git_diff_range, "git diff master...HEAD", Readonly);
class_case!(
    test_git_show_historical,
    "git show HEAD~10:src/main.py",
    Readonly
);
class_case!(
    test_git_add_and_commit,
    "git add -A && git commit -m \"fix: resolve issue\"",
    LocalEffects
);
class_case!(
    test_git_add_diff_cached,
    "git add -A && git diff --cached --stat",
    LocalEffects
);
class_case!(
    test_git_amend_no_edit,
    "git add -A && git commit --amend --no-edit",
    LocalEffects
);
class_case!(
    test_git_amend_and_force_push,
    "git add -A && git commit --amend --no-edit && git push --force-with-lease",
    ExternalEffects
);
class_case!(
    test_git_push_force,
    "git push --force origin main",
    Dangerous
);
class_case!(
    test_git_rebase_continue,
    "git rebase --continue",
    LocalEffects
);
class_case!(test_git_reset_soft, "git reset --soft HEAD~1", LocalEffects);
class_case!(
    test_git_c_flag,
    "git -C /home/user/repos/myrepo status",
    Readonly
);
class_case!(
    test_env_prefix_git_rebase,
    "GIT_EDITOR=true git rebase --continue",
    LocalEffects
);
class_case!(
    test_git_rm_during_rebase,
    "git rm conflicted-file.txt && git rebase --continue",
    LocalEffects
);
class_case!(
    test_mvnw_package_with_grep,
    "./mvnw package -DskipTests --batch-mode 2>&1 | grep -E \"\\[ERROR\\]|BUILD\"",
    LocalEffects
);
class_case!(
    test_mvnw_with_tail,
    "./mvnw package --batch-mode -DskipTests=true 2>&1 | tail -30",
    LocalEffects
);
class_case!(
    test_pytest_verbose,
    "uv run pytest -v --tb=short",
    LocalEffects
);
class_case!(test_npm_test, "npm test", LocalEffects);
class_case!(test_npm_run_build, "npm run build", LocalEffects);
class_case!(test_npm_install, "npm install", LocalEffects);
class_case!(
    test_ruff_check_and_format,
    "ruff check --fix . && ruff format .",
    LocalEffects
);
class_case!(test_make_with_target, "make build", LocalEffects);
class_case!(test_make_dry_run, "make -n install", Readonly);
class_case!(
    test_grep_with_head,
    "grep -E \"pattern1|pattern2\" file.txt | head -20",
    Readonly
);
class_case!(
    test_grep_case_insensitive_with_tail,
    "grep -i 'search_term' logfile | tail -5",
    Readonly
);
class_case!(
    test_grep_with_line_numbers_sort_uniq,
    "grep -n \"pattern\" file | sort | uniq",
    Readonly
);
class_case!(
    test_grep_context_lines,
    "grep -A 20 \"search_term\" file.txt | head -30",
    Readonly
);
class_case!(
    test_grep_before_and_after,
    "grep -B 5 -A 10 \"pattern\" file.txt",
    Readonly
);
class_case!(
    test_grep_inverse_pipe,
    "grep -iE \"\\[WARN\\]|\\[ERROR\\]\" output.log | grep -v \"^\\[INFO\\]\"",
    Readonly
);
class_case!(
    test_cat_pipe_head,
    "cat /path/to/file | head -100",
    Readonly
);
class_case!(
    test_find_pipe_head,
    "find /home/user/project -type f -name \"*.java\" | head -20",
    Readonly
);
class_case!(test_ls_pipe_tail, "ls /path/to/dir | tail -5", Readonly);
class_case!(
    test_sort_pipe_uniq_count_sort,
    "awk \"{print $1}\" access.log | sort | uniq -c | sort -rn | head -20",
    Readonly
);
class_case!(test_ps_pipe_grep, "ps aux | grep python", Readonly);
class_case!(
    test_wc_l_pipeline,
    "find . -name \"*.py\" | wc -l",
    Readonly
);
class_case!(
    test_and_chain_readonly,
    "git status && git log --oneline -3",
    Readonly
);
class_case!(
    test_and_chain_mixed,
    "git add -A && git diff --cached --stat",
    LocalEffects
);
class_case!(
    test_or_chain_with_echo_fallback,
    "cat /path/to/file || echo \"file not found\"",
    Readonly
);
class_case!(
    test_semicolon_chain,
    "echo start; ls -la; echo done",
    Readonly
);
class_case!(
    test_docker_cleanup_chain,
    "docker ps -aq --filter \"label=test\" | xargs -r docker rm -f 2>/dev/null; echo \"done\"",
    Dangerous
);
class_case!(
    test_cd_and_command,
    "cd /home/user/project && ls -la",
    Readonly
);
class_case!(
    test_mkdir_or_fallback,
    "cd /nonexistent || mkdir -p /home/user/tmp/workdir",
    LocalEffects
);
class_case!(
    test_stderr_to_stdout_pipe,
    "./build.sh 2>&1 | grep -E \"ERROR|FAILURE\"",
    Unknown
);
class_case!(
    test_stderr_to_dev_null,
    "find /path -name \"*.log\" 2>/dev/null",
    Readonly
);
class_case!(
    test_redirect_to_file,
    "echo hello > /tmp/test.txt",
    LocalEffects
);
class_case!(
    test_append_redirect,
    "echo line >> /tmp/log.txt",
    LocalEffects
);
class_case!(
    test_unzip_with_fallback,
    "unzip -p archive.jar META-INF/MANIFEST.MF 2>/dev/null || echo \"not found\"",
    LocalEffects
);
class_case!(test_git_commit_with_heredoc, "git commit -m \"$(cat <<'EOF'\nFix authentication bug in login flow\n\nThe session token was not being refreshed properly.\nEOF\n)\" ", LocalEffects);
class_case!(
    test_cat_heredoc,
    "cat <<'EOF'\nhello world\nthis is content\nEOF",
    Readonly
);
class_case!(
    test_variable_in_command_position,
    "$CMD arg1 arg2",
    Dangerous
);
class_case!(
    test_env_prefix_command,
    "FOO=bar BAZ=1 echo hello",
    Readonly
);
class_case!(test_date_format, "date '+%Y%m%d%H%M%S'", Readonly);
class_case!(
    test_kubectl_get_wide,
    "kubectl get pods --context my-cluster -n my-namespace -o wide",
    Readonly
);
class_case!(
    test_kubectl_get_deployment,
    "kubectl get deployment my-app --context my-cluster -o yaml",
    Readonly
);
class_case!(
    test_kubectl_logs_tail,
    "kubectl logs my-pod --tail=30",
    Readonly
);
class_case!(
    test_kubectl_exec_cat,
    "kubectl exec -it my-pod -- cat /etc/config",
    Dangerous
);
class_case!(
    test_kubectl_apply,
    "kubectl apply -f deployment.yaml",
    ExternalEffects
);
class_case!(
    test_kubectl_apply_dry_run,
    "kubectl apply --dry-run=client -f deployment.yaml",
    Readonly
);
class_case!(test_docker_ps, "docker ps -a", Readonly);
class_case!(
    test_docker_run,
    "docker run --rm python:3.12-slim echo hello",
    Dangerous
);
class_case!(
    test_docker_run_with_volume,
    "docker run --rm -v \"$(pwd):/app:ro\" python:3.12-slim cat /app/README.md",
    Dangerous
);
class_case!(test_docker_images, "docker images", Readonly);
class_case!(
    test_docker_system_prune,
    "docker system prune -af",
    Dangerous
);
class_case!(
    test_find_with_maxdepth,
    "find /path -maxdepth 4 -name \"*.json\" 2>/dev/null",
    Readonly
);
class_case!(
    test_find_type_f_pipe_head,
    "find . -type f -name \"*.py\" | head -20",
    Readonly
);
class_case!(
    test_find_delete,
    "find . -name \"*.pyc\" -delete",
    Dangerous
);
class_case!(
    test_xargs_grep,
    "find . -name \"*.py\" | xargs grep \"TODO\"",
    Readonly
);
class_case!(
    test_xargs_rm,
    "find /tmp -name \"*.tmp\" | xargs rm",
    Dangerous
);
class_case!(test_cp_to_tmp, "cp file.txt /tmp/backup.txt", LocalEffects);
class_case!(test_cp_to_etc, "cp config.txt /etc/myapp/config", Dangerous);
class_case!(
    test_mkdir_p,
    "mkdir -p /home/user/project/src",
    LocalEffects
);
class_case!(
    test_source_and_build,
    "source \"$HOME/.sdkman/bin/sdkman-init.sh\" && make build",
    Dangerous
);
class_case!(
    test_conditional_test,
    "[ -f \"file.txt\" ] && cat file.txt",
    Readonly
);
class_case!(
    test_double_bracket_test,
    "[[ -d \"$dir\" ]] && echo \"found\" || echo \"not found\"",
    Readonly
);
class_case!(
    test_for_loop_echo,
    "for f in *.txt; do echo \"$f\"; done",
    Readonly
);
class_case!(
    test_while_read_echo,
    "while read line; do echo \"$line\"; done < file.txt",
    Readonly
);
class_case!(
    test_if_then_cat,
    "if [ -f file.txt ]; then cat file.txt; fi",
    Readonly
);
class_case!(
    test_sudo_with_pipe,
    "sudo cat /var/log/syslog | grep error | tail -20",
    Dangerous
);
class_case!(
    test_git_log_pipe_xargs_show,
    "git log --format=\"%H\" | head -5 | xargs -I{} git show --stat {}",
    Readonly
);
#[test]
fn test_mvn_clean_install() {
    let result = classify_expression("mvn clean install", Some(db())).unwrap();
    assert_eq!(result.classification, LocalEffects);
    assert_eq!(result.risk, Low);
}

#[test]
fn test_mvn_clean_deploy() {
    let result = classify_expression("mvn clean deploy", Some(db())).unwrap();
    assert_eq!(result.classification, ExternalEffects);
    assert_eq!(result.risk, Medium);
}
class_case!(
    test_mvn_custom_plugin_goal,
    "mvn clean some-plugin:goal",
    LocalEffects
);
class_case!(
    test_gradle_clean_build_test,
    "gradle clean build test",
    LocalEffects
);
class_case!(test_gradle_publish, "gradle clean publish", ExternalEffects);
class_case!(
    test_gradle_dry_run,
    "gradle --dry-run clean build",
    Readonly
);
