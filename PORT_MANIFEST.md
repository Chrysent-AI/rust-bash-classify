# Port manifest

This file maps the Python pytest suite at `/home/chrysent/devel/bash-classify/tests` to the Rust tests that carry the same ported assertion obligations. Regenerate it with `scripts/gen_port_manifest.py` after adding, renaming, or moving ported tests.

## Suite arithmetic

| Python suite | Ported tests | Rust locations |
|---|---:|---|
| `tests/test_models.py` | 13 | `src/models.rs` |
| `tests/test_database.py` | 36 | `src/database.rs` |
| `tests/test_smoke.py` | 1 | `tests/smoke.rs` |
| `tests/test_parser.py` | 49 | `src/parser.rs` |
| `tests/test_matcher.py` | 134 | `tests/matcher.rs` |
| `tests/test_classifier.py` | 114 | `src/classifier.rs` |
| `tests/test_real_world.py` | 90 | `tests/real_world.rs` |
| `tests/test_cli_serialization.py` | 18 | `src/cli.rs` |
| `tests/test_cli.py` | 7 | `tests/cli.rs` |
| `tests/test_schema.py` | 1 | `tests/schema.rs` |
| **Total** | **13 + 36 + 1 + 49 + 134 + 114 + 90 + 18 + 7 + 1 = 463** | |

## Per-test map

| # | Python test id | Rust test id | Rust file |
|---:|---|---|---|
| 1 | `tests/test_models.py::TestClassificationSeverity::test_severity_ordering` | `models::test_models::test_classification_severity::test_severity_ordering` | `src/models.rs` |
| 2 | `tests/test_models.py::TestClassificationSeverity::test_max_severity_empty` | `models::test_models::test_classification_severity::test_max_severity_empty` | `src/models.rs` |
| 3 | `tests/test_models.py::TestClassificationSeverity::test_max_severity_single` | `models::test_models::test_classification_severity::test_max_severity_single` | `src/models.rs` |
| 4 | `tests/test_models.py::TestClassificationSeverity::test_max_severity_multiple` | `models::test_models::test_classification_severity::test_max_severity_multiple` | `src/models.rs` |
| 5 | `tests/test_models.py::TestClassificationSeverity::test_max_severity_ties` | `models::test_models::test_classification_severity::test_max_severity_ties` | `src/models.rs` |
| 6 | `tests/test_models.py::TestClassificationSeverity::test_max_severity_all_four` | `models::test_models::test_classification_severity::test_max_severity_all_four` | `src/models.rs` |
| 7 | `tests/test_models.py::TestRiskSeverity::test_severity_ordering` | `models::test_models::test_risk_severity::test_severity_ordering` | `src/models.rs` |
| 8 | `tests/test_models.py::TestRiskSeverity::test_max_severity_empty` | `models::test_models::test_risk_severity::test_max_severity_empty` | `src/models.rs` |
| 9 | `tests/test_models.py::TestRiskSeverity::test_max_severity_single` | `models::test_models::test_risk_severity::test_max_severity_single` | `src/models.rs` |
| 10 | `tests/test_models.py::TestRiskSeverity::test_max_severity_multiple` | `models::test_models::test_risk_severity::test_max_severity_multiple` | `src/models.rs` |
| 11 | `tests/test_models.py::TestRiskSeverity::test_max_severity_ties` | `models::test_models::test_risk_severity::test_max_severity_ties` | `src/models.rs` |
| 12 | `tests/test_models.py::TestRiskSeverity::test_max_severity_all_three` | `models::test_models::test_risk_severity::test_max_severity_all_three` | `src/models.rs` |
| 13 | `tests/test_models.py::TestSubcommandMode::test_subcommand_mode_values` | `models::test_models::test_subcommand_mode::test_subcommand_mode_values` | `src/models.rs` |
| 14 | `tests/test_database.py::TestDatabaseLoading::test_load_all_yaml_files_succeeds` | `database::test_database::test_database_loading::test_load_all_yaml_files_succeeds` | `src/database.rs` |
| 15 | `tests/test_database.py::TestDatabaseLoading::test_all_entries_are_command_defs` | `database::test_database::test_database_loading::test_all_entries_are_command_defs` | `src/database.rs` |
| 16 | `tests/test_database.py::TestKubectlYaml::test_kubectl_exists` | `database::test_database::test_kubectl_yaml::test_kubectl_exists` | `src/database.rs` |
| 17 | `tests/test_database.py::TestKubectlYaml::test_kubectl_global_options_context` | `database::test_database::test_kubectl_yaml::test_kubectl_global_options_context` | `src/database.rs` |
| 18 | `tests/test_database.py::TestKubectlYaml::test_kubectl_global_options_namespace` | `database::test_database::test_kubectl_yaml::test_kubectl_global_options_namespace` | `src/database.rs` |
| 19 | `tests/test_database.py::TestKubectlYaml::test_kubectl_alias_resolution_n` | `database::test_database::test_kubectl_yaml::test_kubectl_alias_resolution_n` | `src/database.rs` |
| 20 | `tests/test_database.py::TestKubectlYaml::test_kubectl_subcommand_get` | `database::test_database::test_kubectl_yaml::test_kubectl_subcommand_get` | `src/database.rs` |
| 21 | `tests/test_database.py::TestKubectlYaml::test_kubectl_subcommand_delete` | `database::test_database::test_kubectl_yaml::test_kubectl_subcommand_delete` | `src/database.rs` |
| 22 | `tests/test_database.py::TestKubectlYaml::test_kubectl_subcommand_exec` | `database::test_database::test_kubectl_yaml::test_kubectl_subcommand_exec` | `src/database.rs` |
| 23 | `tests/test_database.py::TestGitYaml::test_git_exists` | `database::test_database::test_git_yaml::test_git_exists` | `src/database.rs` |
| 24 | `tests/test_database.py::TestGitYaml::test_git_global_option_c_captures_directory` | `database::test_database::test_git_yaml::test_git_global_option_c_captures_directory` | `src/database.rs` |
| 25 | `tests/test_database.py::TestGitYaml::test_git_subcommand_status` | `database::test_database::test_git_yaml::test_git_subcommand_status` | `src/database.rs` |
| 26 | `tests/test_database.py::TestGitYaml::test_git_subcommand_push` | `database::test_database::test_git_yaml::test_git_subcommand_push` | `src/database.rs` |
| 27 | `tests/test_database.py::TestFindYaml::test_find_base_classification` | `database::test_database::test_find_yaml::test_find_base_classification` | `src/database.rs` |
| 28 | `tests/test_database.py::TestFindYaml::test_find_strict_false` | `database::test_database::test_find_yaml::test_find_strict_false` | `src/database.rs` |
| 29 | `tests/test_database.py::TestFindYaml::test_find_exec_delegates_to_terminated_argv` | `database::test_database::test_find_yaml::test_find_exec_delegates_to_terminated_argv` | `src/database.rs` |
| 30 | `tests/test_database.py::TestFindYaml::test_find_delete_overrides_dangerous` | `database::test_database::test_find_yaml::test_find_delete_overrides_dangerous` | `src/database.rs` |
| 31 | `tests/test_database.py::TestXargsYaml::test_xargs_delegates_to_rest_are_argv` | `database::test_database::test_xargs_yaml::test_xargs_delegates_to_rest_are_argv` | `src/database.rs` |
| 32 | `tests/test_database.py::TestShYaml::test_sh_delegates_to_flag_value_is_expression` | `database::test_database::test_sh_yaml::test_sh_delegates_to_flag_value_is_expression` | `src/database.rs` |
| 33 | `tests/test_database.py::TestSudoYaml::test_sudo_delegates_to_rest_are_argv_with_min_classification` | `database::test_database::test_sudo_yaml::test_sudo_delegates_to_rest_are_argv_with_min_classification` | `src/database.rs` |
| 34 | `tests/test_database.py::TestEnvYaml::test_env_delegates_to_rest_are_argv_with_strip_assignments` | `database::test_database::test_env_yaml::test_env_delegates_to_rest_are_argv_with_strip_assignments` | `src/database.rs` |
| 35 | `tests/test_database.py::TestDatabaseErrorHandling::test_invalid_yaml_includes_filename` | `database::test_database::test_database_error_handling::test_invalid_yaml_includes_filename` | `src/database.rs` |
| 36 | `tests/test_database.py::TestDatabaseErrorHandling::test_missing_command_key_includes_filename` | `database::test_database::test_database_error_handling::test_missing_command_key_includes_filename` | `src/database.rs` |
| 37 | `tests/test_database.py::TestEmptyAndNonDictYamlFiles::test_empty_yaml_file_raises_on_access` | `database::test_database::test_empty_and_non_dict_yaml_files::test_empty_yaml_file_raises_on_access` | `src/database.rs` |
| 38 | `tests/test_database.py::TestEmptyAndNonDictYamlFiles::test_list_yaml_raises_valueerror_on_access` | `database::test_database::test_empty_and_non_dict_yaml_files::test_list_yaml_raises_valueerror_on_access` | `src/database.rs` |
| 39 | `tests/test_database.py::TestStrictDefault::test_strict_defaults_to_true` | `database::test_database::test_strict_default::test_strict_defaults_to_true` | `src/database.rs` |
| 40 | `tests/test_database.py::TestStrictDefault::test_strict_false_on_find` | `database::test_database::test_strict_default::test_strict_false_on_find` | `src/database.rs` |
| 41 | `tests/test_database.py::TestStrictDefault::test_strict_false_on_grep` | `database::test_database::test_strict_default::test_strict_false_on_grep` | `src/database.rs` |
| 42 | `tests/test_database.py::TestAliasOf::test_alias_file_parses` | `database::test_database::test_alias_of::test_alias_file_parses` | `src/database.rs` |
| 43 | `tests/test_database.py::TestAliasOf::test_alias_file_with_subcommands_raises` | `database::test_database::test_alias_of::test_alias_file_with_subcommands_raises` | `src/database.rs` |
| 44 | `tests/test_database.py::TestAliasOf::test_alias_file_with_classification_raises` | `database::test_database::test_alias_of::test_alias_file_with_classification_raises` | `src/database.rs` |
| 45 | `tests/test_database.py::TestAliasOf::test_alias_file_only_command_description_alias_of_is_valid` | `database::test_database::test_alias_of::test_alias_file_only_command_description_alias_of_is_valid` | `src/database.rs` |
| 46 | `tests/test_database.py::TestUserCommandsDir::test_user_override_replaces_builtin` | `database::test_database::test_user_commands_dir::test_user_override_replaces_builtin` | `src/database.rs` |
| 47 | `tests/test_database.py::TestUserCommandsDir::test_user_adds_new_command` | `database::test_database::test_user_commands_dir::test_user_adds_new_command` | `src/database.rs` |
| 48 | `tests/test_database.py::TestUserCommandsDir::test_missing_user_dir_is_silently_skipped` | `database::test_database::test_user_commands_dir::test_missing_user_dir_is_silently_skipped` | `src/database.rs` |
| 49 | `tests/test_database.py::TestUserCommandsDir::test_explicit_commands_dir_skips_user_overrides` | `database::test_database::test_user_commands_dir::test_explicit_commands_dir_skips_user_overrides` | `src/database.rs` |
| 50 | `tests/test_smoke.py::test_import` | `test_crate_import_smoke` | `tests/smoke.rs` |
| 51 | `tests/test_parser.py::TestSimpleCommands::test_simple_command` | `parser::test_parser::test_simple_commands::test_simple_command` | `src/parser.rs` |
| 52 | `tests/test_parser.py::TestSimpleCommands::test_command_with_args` | `parser::test_parser::test_simple_commands::test_command_with_args` | `src/parser.rs` |
| 53 | `tests/test_parser.py::TestSimpleCommands::test_command_with_long_options` | `parser::test_parser::test_simple_commands::test_command_with_long_options` | `src/parser.rs` |
| 54 | `tests/test_parser.py::TestPipelines::test_simple_pipeline` | `parser::test_parser::test_pipelines::test_simple_pipeline` | `src/parser.rs` |
| 55 | `tests/test_parser.py::TestPipelines::test_two_command_pipeline` | `parser::test_parser::test_pipelines::test_two_command_pipeline` | `src/parser.rs` |
| 56 | `tests/test_parser.py::TestLogicalOperators::test_logical_and` | `parser::test_parser::test_logical_operators::test_logical_and` | `src/parser.rs` |
| 57 | `tests/test_parser.py::TestLogicalOperators::test_logical_or` | `parser::test_parser::test_logical_operators::test_logical_or` | `src/parser.rs` |
| 58 | `tests/test_parser.py::TestLogicalOperators::test_chained_operators` | `parser::test_parser::test_logical_operators::test_chained_operators` | `src/parser.rs` |
| 59 | `tests/test_parser.py::TestSemicolons::test_semicolons` | `parser::test_parser::test_semicolons::test_semicolons` | `src/parser.rs` |
| 60 | `tests/test_parser.py::TestCommandSubstitution::test_command_substitution` | `parser::test_parser::test_command_substitution::test_command_substitution` | `src/parser.rs` |
| 61 | `tests/test_parser.py::TestCommandSubstitution::test_nested_command_substitution_in_pipeline` | `parser::test_parser::test_command_substitution::test_nested_command_substitution_in_pipeline` | `src/parser.rs` |
| 62 | `tests/test_parser.py::TestProcessSubstitution::test_process_substitution` | `parser::test_parser::test_process_substitution::test_process_substitution` | `src/parser.rs` |
| 63 | `tests/test_parser.py::TestSubshell::test_subshell` | `parser::test_parser::test_subshell::test_subshell` | `src/parser.rs` |
| 64 | `tests/test_parser.py::TestRedirects::test_output_redirect` | `parser::test_parser::test_redirects::test_output_redirect` | `src/parser.rs` |
| 65 | `tests/test_parser.py::TestRedirects::test_stderr_redirect` | `parser::test_parser::test_redirects::test_stderr_redirect` | `src/parser.rs` |
| 66 | `tests/test_parser.py::TestRedirects::test_append_redirect` | `parser::test_parser::test_redirects::test_append_redirect` | `src/parser.rs` |
| 67 | `tests/test_parser.py::TestRedirects::test_input_redirect` | `parser::test_parser::test_redirects::test_input_redirect` | `src/parser.rs` |
| 68 | `tests/test_parser.py::TestRedirects::test_redirect_affects_classification` | `parser::test_parser::test_redirects::test_redirect_affects_classification` | `src/parser.rs` |
| 69 | `tests/test_parser.py::TestRedirects::test_devnull_redirect_no_affect` | `parser::test_parser::test_redirects::test_devnull_redirect_no_affect` | `src/parser.rs` |
| 70 | `tests/test_parser.py::TestVariableAssignment::test_variable_assignment_prefix` | `parser::test_parser::test_variable_assignment::test_variable_assignment_prefix` | `src/parser.rs` |
| 71 | `tests/test_parser.py::TestVariableAssignment::test_multiple_assignments` | `parser::test_parser::test_variable_assignment::test_multiple_assignments` | `src/parser.rs` |
| 72 | `tests/test_parser.py::TestVariableAssignment::test_bare_assignment_no_command` | `parser::test_parser::test_variable_assignment::test_bare_assignment_no_command` | `src/parser.rs` |
| 73 | `tests/test_parser.py::TestBackground::test_background` | `parser::test_parser::test_background::test_background` | `src/parser.rs` |
| 74 | `tests/test_parser.py::TestBackground::test_background_in_list` | `parser::test_parser::test_background::test_background_in_list` | `src/parser.rs` |
| 75 | `tests/test_parser.py::TestComplex::test_complex_expression` | `parser::test_parser::test_complex::test_complex_expression` | `src/parser.rs` |
| 76 | `tests/test_parser.py::TestHeredoc::test_heredoc` | `parser::test_parser::test_heredoc::test_heredoc` | `src/parser.rs` |
| 77 | `tests/test_parser.py::TestHeredoc::test_heredoc_content_not_parsed_as_commands` | `parser::test_parser::test_heredoc::test_heredoc_content_not_parsed_as_commands` | `src/parser.rs` |
| 78 | `tests/test_parser.py::TestEdgeCases::test_empty_expression` | `parser::test_parser::test_edge_cases::test_empty_expression` | `src/parser.rs` |
| 79 | `tests/test_parser.py::TestEdgeCases::test_comment_only` | `parser::test_parser::test_edge_cases::test_comment_only` | `src/parser.rs` |
| 80 | `tests/test_parser.py::TestEdgeCases::test_quoted_string_arg` | `parser::test_parser::test_edge_cases::test_quoted_string_arg` | `src/parser.rs` |
| 81 | `tests/test_parser.py::TestEdgeCases::test_single_quoted_string_arg` | `parser::test_parser::test_edge_cases::test_single_quoted_string_arg` | `src/parser.rs` |
| 82 | `tests/test_parser.py::TestBacktickCommandSubstitution::test_backtick_command_substitution` | `parser::test_parser::test_backtick_command_substitution::test_backtick_command_substitution` | `src/parser.rs` |
| 83 | `tests/test_parser.py::TestNegatedCommand::test_negated_command` | `parser::test_parser::test_negated_command::test_negated_command` | `src/parser.rs` |
| 84 | `tests/test_parser.py::TestAnsiCQuotedString::test_dollar_single_quoted_string` | `parser::test_parser::test_ansi_c_quoted_string::test_dollar_single_quoted_string` | `src/parser.rs` |
| 85 | `tests/test_parser.py::TestMultipleRedirects::test_multiple_redirects` | `parser::test_parser::test_multiple_redirects::test_multiple_redirects` | `src/parser.rs` |
| 86 | `tests/test_parser.py::TestConcatenation::test_concatenation` | `parser::test_parser::test_concatenation::test_concatenation` | `src/parser.rs` |
| 87 | `tests/test_parser.py::TestTestCommand::test_single_bracket_test` | `parser::test_parser::test_test_command::test_single_bracket_test` | `src/parser.rs` |
| 88 | `tests/test_parser.py::TestTestCommand::test_double_bracket_test` | `parser::test_parser::test_test_command::test_double_bracket_test` | `src/parser.rs` |
| 89 | `tests/test_parser.py::TestTestCommand::test_test_command_in_pipeline` | `parser::test_parser::test_test_command::test_test_command_in_pipeline` | `src/parser.rs` |
| 90 | `tests/test_parser.py::TestHerestring::test_herestring_redirect` | `parser::test_parser::test_herestring::test_herestring_redirect` | `src/parser.rs` |
| 91 | `tests/test_parser.py::TestRedirectedCompoundStatement::test_for_loop_redirect_not_duplicated` | `parser::test_parser::test_redirected_compound_statement::test_for_loop_redirect_not_duplicated` | `src/parser.rs` |
| 92 | `tests/test_parser.py::TestFdToFdRedirects::test_stderr_to_stdout_no_affect` | `parser::test_parser::test_fd_to_fd_redirects::test_stderr_to_stdout_no_affect` | `src/parser.rs` |
| 93 | `tests/test_parser.py::TestFdToFdRedirects::test_stdout_to_stderr_no_affect` | `parser::test_parser::test_fd_to_fd_redirects::test_stdout_to_stderr_no_affect` | `src/parser.rs` |
| 94 | `tests/test_parser.py::TestFdToFdRedirects::test_stderr_to_file_affects` | `parser::test_parser::test_fd_to_fd_redirects::test_stderr_to_file_affects` | `src/parser.rs` |
| 95 | `tests/test_parser.py::TestAmpersandRedirects::test_ampersand_redirect_to_file` | `parser::test_parser::test_ampersand_redirects::test_ampersand_redirect_to_file` | `src/parser.rs` |
| 96 | `tests/test_parser.py::TestAmpersandRedirects::test_ampersand_append_redirect_to_file` | `parser::test_parser::test_ampersand_redirects::test_ampersand_append_redirect_to_file` | `src/parser.rs` |
| 97 | `tests/test_parser.py::TestAmpersandRedirects::test_ampersand_redirect_to_devnull` | `parser::test_parser::test_ampersand_redirects::test_ampersand_redirect_to_devnull` | `src/parser.rs` |
| 98 | `tests/test_parser.py::TestProcessSubstitutionAsRedirectTarget::test_output_process_substitution` | `parser::test_parser::test_process_substitution_as_redirect_target::test_output_process_substitution` | `src/parser.rs` |
| 99 | `tests/test_parser.py::TestParseWarnings::test_syntax_error_produces_warning` | `parser::test_parser::test_parse_warnings::test_syntax_error_produces_warning` | `src/parser.rs` |
| 100 | `tests/test_matcher.py::TestBasicMatching::test_ls_readonly` | `test_basic_matching::test_ls_readonly` | `tests/matcher.rs` |
| 101 | `tests/test_matcher.py::TestBasicMatching::test_cat_readonly` | `test_basic_matching::test_cat_readonly` | `tests/matcher.rs` |
| 102 | `tests/test_matcher.py::TestBasicMatching::test_rm_classification` | `test_basic_matching::test_rm_classification` | `tests/matcher.rs` |
| 103 | `tests/test_matcher.py::TestBasicMatching::test_unknown_command` | `test_basic_matching::test_unknown_command` | `tests/matcher.rs` |
| 104 | `tests/test_matcher.py::TestGlobalOptionStripping::test_kubectl_context_equals_form` | `test_global_option_stripping::test_kubectl_context_equals_form` | `tests/matcher.rs` |
| 105 | `tests/test_matcher.py::TestGlobalOptionStripping::test_kubectl_context_separated_form` | `test_global_option_stripping::test_kubectl_context_separated_form` | `tests/matcher.rs` |
| 106 | `tests/test_matcher.py::TestGlobalOptionStripping::test_git_c_captures_directory` | `test_global_option_stripping::test_git_c_captures_directory` | `tests/matcher.rs` |
| 107 | `tests/test_matcher.py::TestSubcommandMatching::test_kubectl_get` | `test_subcommand_matching::test_kubectl_get` | `tests/matcher.rs` |
| 108 | `tests/test_matcher.py::TestSubcommandMatching::test_kubectl_delete` | `test_subcommand_matching::test_kubectl_delete` | `tests/matcher.rs` |
| 109 | `tests/test_matcher.py::TestSubcommandMatching::test_kubectl_rollout_status` | `test_subcommand_matching::test_kubectl_rollout_status` | `tests/matcher.rs` |
| 110 | `tests/test_matcher.py::TestSubcommandMatching::test_kubectl_rollout_undo` | `test_subcommand_matching::test_kubectl_rollout_undo` | `tests/matcher.rs` |
| 111 | `tests/test_matcher.py::TestSubcommandMatching::test_git_stash_list` | `test_subcommand_matching::test_git_stash_list` | `tests/matcher.rs` |
| 112 | `tests/test_matcher.py::TestSubcommandMatching::test_git_stash_drop` | `test_subcommand_matching::test_git_stash_drop` | `tests/matcher.rs` |
| 113 | `tests/test_matcher.py::TestOptionOverrides::test_git_push_force` | `test_option_overrides::test_git_push_force` | `tests/matcher.rs` |
| 114 | `tests/test_matcher.py::TestOptionOverrides::test_git_push_f_alias` | `test_option_overrides::test_git_push_f_alias` | `tests/matcher.rs` |
| 115 | `tests/test_matcher.py::TestOptionOverrides::test_git_push_force_with_lease` | `test_option_overrides::test_git_push_force_with_lease` | `tests/matcher.rs` |
| 116 | `tests/test_matcher.py::TestOptionOverrides::test_git_branch_d_dangerous` | `test_option_overrides::test_git_branch_d_dangerous` | `tests/matcher.rs` |
| 117 | `tests/test_matcher.py::TestOptionOverrides::test_git_reset_hard` | `test_option_overrides::test_git_reset_hard` | `tests/matcher.rs` |
| 118 | `tests/test_matcher.py::TestOptionOverrides::test_kubectl_apply_dry_run` | `test_option_overrides::test_kubectl_apply_dry_run` | `tests/matcher.rs` |
| 119 | `tests/test_matcher.py::TestOptionOverrides::test_kubectl_apply_dry_run_no_unknown` | `test_option_overrides::test_kubectl_apply_dry_run_no_unknown` | `tests/matcher.rs` |
| 120 | `tests/test_matcher.py::TestOptionOverrides::test_kubectl_delete_dry_run` | `test_option_overrides::test_kubectl_delete_dry_run` | `tests/matcher.rs` |
| 121 | `tests/test_matcher.py::TestOptionOverrides::test_git_push_force_overrides_to_dangerous` | `test_option_overrides::test_git_push_force_overrides_to_dangerous` | `tests/matcher.rs` |
| 122 | `tests/test_matcher.py::TestStrictMode::test_kubectl_top_unknown_flag` | `test_strict_mode::test_kubectl_top_unknown_flag` | `tests/matcher.rs` |
| 123 | `tests/test_matcher.py::TestStrictMode::test_grep_unknown_flag_stays_readonly` | `test_strict_mode::test_grep_unknown_flag_stays_readonly` | `tests/matcher.rs` |
| 124 | `tests/test_matcher.py::TestStrictMode::test_find_unknown_predicate_stays_readonly` | `test_strict_mode::test_find_unknown_predicate_stays_readonly` | `tests/matcher.rs` |
| 125 | `tests/test_matcher.py::TestDelegationRestAreArgv::test_xargs_delegates_inner_command` | `test_delegation::test_xargs_delegates_inner_command` | `tests/matcher.rs` |
| 126 | `tests/test_matcher.py::TestDelegationRestAreArgv::test_sudo_delegates_with_min_classification` | `test_delegation::test_sudo_delegates_with_min_classification` | `tests/matcher.rs` |
| 127 | `tests/test_matcher.py::TestDelegationRestAreArgv::test_nice_delegates_inner_command` | `test_delegation::test_nice_delegates_inner_command` | `tests/matcher.rs` |
| 128 | `tests/test_matcher.py::TestDelegationAfterSeparator::test_kubectl_exec_after_separator` | `test_delegation::test_kubectl_exec_after_separator` | `tests/matcher.rs` |
| 129 | `tests/test_matcher.py::TestDelegationTerminatedArgv::test_find_exec_terminated_argv` | `test_delegation::test_find_exec_terminated_argv` | `tests/matcher.rs` |
| 130 | `tests/test_matcher.py::TestDelegationFlagValueIsExpression::test_sh_c_parses_expression` | `test_delegation::test_sh_c_parses_expression` | `tests/matcher.rs` |
| 131 | `tests/test_matcher.py::TestDelegationEnvStripAssignments::test_env_strip_assignments` | `test_delegation::test_env_strip_assignments` | `tests/matcher.rs` |
| 132 | `tests/test_matcher.py::TestSkipLeadingPositionals::test_timeout_duration_then_readonly_inner` | `test_skip_leading_positionals::test_timeout_duration_then_readonly_inner` | `tests/matcher.rs` |
| 133 | `tests/test_matcher.py::TestSkipLeadingPositionals::test_timeout_duration_then_dangerous_inner_elevates` | `test_skip_leading_positionals::test_timeout_duration_then_dangerous_inner_elevates` | `tests/matcher.rs` |
| 134 | `tests/test_matcher.py::TestSkipLeadingPositionals::test_timeout_flag_then_duration_then_inner` | `test_skip_leading_positionals::test_timeout_flag_then_duration_then_inner` | `tests/matcher.rs` |
| 135 | `tests/test_matcher.py::TestSkipLeadingPositionals::test_timeout_only_duration_no_inner` | `test_skip_leading_positionals::test_timeout_only_duration_no_inner` | `tests/matcher.rs` |
| 136 | `tests/test_matcher.py::TestSkipLeadingPositionals::test_timeout_no_args` | `test_skip_leading_positionals::test_timeout_no_args` | `tests/matcher.rs` |
| 137 | `tests/test_matcher.py::TestCombinedShortFlags::test_combined_boolean_flags` | `test_combined_short_flags::test_combined_boolean_flags` | `tests/matcher.rs` |
| 138 | `tests/test_matcher.py::TestCombinedShortFlags::test_combined_flag_middle_takes_value` | `test_combined_short_flags::test_combined_flag_middle_takes_value` | `tests/matcher.rs` |
| 139 | `tests/test_matcher.py::TestCombinedShortFlags::test_combined_flag_last_takes_value_separate` | `test_combined_short_flags::test_combined_flag_last_takes_value_separate` | `tests/matcher.rs` |
| 140 | `tests/test_matcher.py::TestSshNoDelegation::test_ssh_no_inner_commands` | `test_misc_builtin_and_helpers::test_ssh_no_inner_commands` | `tests/matcher.rs` |
| 141 | `tests/test_matcher.py::TestKubectlOptions::test_kubectl_apply_dry_run_equals_form` | `test_misc_builtin_and_helpers::test_kubectl_apply_dry_run_equals_form` | `tests/matcher.rs` |
| 142 | `tests/test_matcher.py::TestKubectlOptions::test_kubectl_get_output` | `test_misc_builtin_and_helpers::test_kubectl_get_output` | `tests/matcher.rs` |
| 143 | `tests/test_matcher.py::TestKubectlOptions::test_kubectl_delete_filename` | `test_misc_builtin_and_helpers::test_kubectl_delete_filename` | `tests/matcher.rs` |
| 144 | `tests/test_matcher.py::TestSpecialBuiltins::test_cd_readonly` | `test_misc_builtin_and_helpers::test_cd_readonly` | `tests/matcher.rs` |
| 145 | `tests/test_matcher.py::TestSpecialBuiltins::test_pushd_readonly` | `test_misc_builtin_and_helpers::test_pushd_readonly` | `tests/matcher.rs` |
| 146 | `tests/test_matcher.py::TestSpecialBuiltins::test_eval_dangerous` | `test_misc_builtin_and_helpers::test_eval_dangerous` | `tests/matcher.rs` |
| 147 | `tests/test_matcher.py::TestSpecialBuiltins::test_source_dangerous` | `test_misc_builtin_and_helpers::test_source_dangerous` | `tests/matcher.rs` |
| 148 | `tests/test_matcher.py::TestSpecialBuiltins::test_dot_dangerous` | `test_misc_builtin_and_helpers::test_dot_dangerous` | `tests/matcher.rs` |
| 149 | `tests/test_matcher.py::TestSpecialBuiltins::test_exec_dangerous` | `test_misc_builtin_and_helpers::test_exec_dangerous` | `tests/matcher.rs` |
| 150 | `tests/test_matcher.py::TestEmptyArgv::test_empty_argv_unknown` | `test_misc_builtin_and_helpers::test_empty_argv_unknown` | `tests/matcher.rs` |
| 151 | `tests/test_matcher.py::TestTestBuiltin::test_test_builtin_readonly` | `test_misc_builtin_and_helpers::test_test_builtin_readonly` | `tests/matcher.rs` |
| 152 | `tests/test_matcher.py::TestTestBuiltin::test_bracket_readonly` | `test_misc_builtin_and_helpers::test_bracket_readonly` | `tests/matcher.rs` |
| 153 | `tests/test_matcher.py::TestTestBuiltin::test_double_bracket_readonly` | `test_misc_builtin_and_helpers::test_double_bracket_readonly` | `tests/matcher.rs` |
| 154 | `tests/test_matcher.py::TestPopdBuiltin::test_popd_readonly` | `test_misc_builtin_and_helpers::test_popd_readonly` | `tests/matcher.rs` |
| 155 | `tests/test_matcher.py::TestMultipleConflictingOverrides::test_highest_override_wins` | `test_misc_builtin_and_helpers::test_highest_override_wins` | `tests/matcher.rs` |
| 156 | `tests/test_matcher.py::TestIsTerminator::test_exact_match` | `test_misc_builtin_and_helpers::test_exact_match` | `tests/matcher.rs` |
| 157 | `tests/test_matcher.py::TestIsTerminator::test_backslash_form` | `test_misc_builtin_and_helpers::test_backslash_form` | `tests/matcher.rs` |
| 158 | `tests/test_matcher.py::TestIsTerminator::test_no_match` | `test_misc_builtin_and_helpers::test_no_match` | `tests/matcher.rs` |
| 159 | `tests/test_matcher.py::TestIsTerminator::test_none_terminator` | `test_misc_builtin_and_helpers::test_none_terminator` | `tests/matcher.rs` |
| 160 | `tests/test_matcher.py::TestRestAreArgvNoPositional::test_xargs_no_inner_command` | `test_misc_builtin_and_helpers::test_xargs_no_inner_command` | `tests/matcher.rs` |
| 161 | `tests/test_matcher.py::TestAfterSeparatorNoInnerArgs::test_kubectl_exec_no_inner_after_separator` | `test_misc_builtin_and_helpers::test_kubectl_exec_no_inner_after_separator` | `tests/matcher.rs` |
| 162 | `tests/test_matcher.py::TestFlagValueIsExpressionNotFound::test_sh_without_c_flag` | `test_misc_builtin_and_helpers::test_sh_without_c_flag` | `tests/matcher.rs` |
| 163 | `tests/test_matcher.py::TestStripQuotes::test_single_quoted_value` | `test_misc_builtin_and_helpers::test_single_quoted_value` | `tests/matcher.rs` |
| 164 | `tests/test_matcher.py::TestStripQuotes::test_double_quoted_value` | `test_misc_builtin_and_helpers::test_double_quoted_value` | `tests/matcher.rs` |
| 165 | `tests/test_matcher.py::TestStripQuotes::test_unquoted_value` | `test_misc_builtin_and_helpers::test_unquoted_value` | `tests/matcher.rs` |
| 166 | `tests/test_matcher.py::TestDefaultClassificationWhenNotSet::test_command_without_classification_defaults_to_readonly` | `test_default_classification_when_not_set::test_command_without_classification_defaults_to_readonly` | `tests/matcher.rs` |
| 167 | `tests/test_matcher.py::TestDefaultClassificationWhenNotSet::test_apt_help_readonly` | `test_default_classification_when_not_set::test_apt_help_readonly` | `tests/matcher.rs` |
| 168 | `tests/test_matcher.py::TestDefaultClassificationWhenNotSet::test_apt_search_readonly` | `test_default_classification_when_not_set::test_apt_search_readonly` | `tests/matcher.rs` |
| 169 | `tests/test_matcher.py::TestDefaultClassificationWhenNotSet::test_apt_install_dangerous` | `test_default_classification_when_not_set::test_apt_install_dangerous` | `tests/matcher.rs` |
| 170 | `tests/test_matcher.py::TestDefaultClassificationWhenNotSet::test_docker_bare_readonly` | `test_default_classification_when_not_set::test_docker_bare_readonly` | `tests/matcher.rs` |
| 171 | `tests/test_matcher.py::TestDefaultClassificationWhenNotSet::test_npm_bare_readonly` | `test_default_classification_when_not_set::test_npm_bare_readonly` | `tests/matcher.rs` |
| 172 | `tests/test_matcher.py::TestDefaultClassificationWhenNotSet::test_helm_bare_readonly` | `test_default_classification_when_not_set::test_helm_bare_readonly` | `tests/matcher.rs` |
| 173 | `tests/test_matcher.py::TestDefaultClassificationWhenNotSet::test_systemctl_bare_dangerous` | `test_default_classification_when_not_set::test_systemctl_bare_dangerous` | `tests/matcher.rs` |
| 174 | `tests/test_matcher.py::TestDefaultClassificationWhenNotSet::test_systemctl_start_dangerous` | `test_default_classification_when_not_set::test_systemctl_start_dangerous` | `tests/matcher.rs` |
| 175 | `tests/test_matcher.py::TestDefaultClassificationWhenNotSet::test_docker_system_prune_dangerous` | `test_default_classification_when_not_set::test_docker_system_prune_dangerous` | `tests/matcher.rs` |
| 176 | `tests/test_matcher.py::TestRemainingOptionsNonStrict::test_remaining_options_in_non_strict_mode` | `test_remaining_and_combined_unknown::test_remaining_options_in_non_strict_mode` | `tests/matcher.rs` |
| 177 | `tests/test_matcher.py::TestCombinedShortFlagsUnknownChars::test_combined_known_and_unknown_strict_mode` | `test_remaining_and_combined_unknown::test_combined_known_and_unknown_strict_mode` | `tests/matcher.rs` |
| 178 | `tests/test_matcher.py::TestCombinedShortFlagsUnknownChars::test_combined_known_first_unknown_second_strict` | `test_remaining_and_combined_unknown::test_combined_known_first_unknown_second_strict` | `tests/matcher.rs` |
| 179 | `tests/test_matcher.py::TestCombinedShortFlagsUnknownChars::test_combined_known_and_unknown_nonstrict_mode` | `test_remaining_and_combined_unknown::test_combined_known_and_unknown_nonstrict_mode` | `tests/matcher.rs` |
| 180 | `tests/test_matcher.py::TestEndOfOptionsInNonDelegation::test_double_dash_stops_option_parsing` | `test_remaining_and_combined_unknown::test_double_dash_stops_option_parsing` | `tests/matcher.rs` |
| 181 | `tests/test_matcher.py::TestOptionDelegationNonTerminatedMode::test_non_terminated_argv_returns_empty` | `test_remaining_and_combined_unknown::test_non_terminated_argv_returns_empty` | `tests/matcher.rs` |
| 182 | `tests/test_matcher.py::TestOverrideOrderDoesNotMatter::test_force_then_force_with_lease` | `test_remaining_and_combined_unknown::test_force_then_force_with_lease` | `tests/matcher.rs` |
| 183 | `tests/test_matcher.py::TestOverrideOrderDoesNotMatter::test_force_with_lease_then_force` | `test_remaining_and_combined_unknown::test_force_with_lease_then_force` | `tests/matcher.rs` |
| 184 | `tests/test_matcher.py::TestShFlagEqualsValue::test_sh_c_equals_form` | `test_remaining_and_combined_unknown::test_sh_c_equals_form` | `tests/matcher.rs` |
| 185 | `tests/test_matcher.py::TestDelegationFloor::test_sh_c_readonly_inner` | `test_delegation_floor::test_sh_c_readonly_inner` | `tests/matcher.rs` |
| 186 | `tests/test_matcher.py::TestDelegationFloor::test_bash_c_readonly_inner` | `test_delegation_floor::test_bash_c_readonly_inner` | `tests/matcher.rs` |
| 187 | `tests/test_matcher.py::TestDelegationFloor::test_sh_c_dangerous_inner_elevates` | `test_delegation_floor::test_sh_c_dangerous_inner_elevates` | `tests/matcher.rs` |
| 188 | `tests/test_matcher.py::TestDelegationFloor::test_sh_c_local_effects_inner` | `test_delegation_floor::test_sh_c_local_effects_inner` | `tests/matcher.rs` |
| 189 | `tests/test_matcher.py::TestDelegationFloor::test_sh_without_c_preserves_base` | `test_delegation_floor::test_sh_without_c_preserves_base` | `tests/matcher.rs` |
| 190 | `tests/test_matcher.py::TestDelegationFloor::test_sh_c_empty_preserves_base` | `test_delegation_floor::test_sh_c_empty_preserves_base` | `tests/matcher.rs` |
| 191 | `tests/test_matcher.py::TestDelegationFloor::test_sudo_cat_unchanged` | `test_delegation_floor::test_sudo_cat_unchanged` | `tests/matcher.rs` |
| 192 | `tests/test_matcher.py::TestDelegationFloor::test_sh_c_unknown_option_stays_dangerous` | `test_delegation_floor::test_sh_c_unknown_option_stays_dangerous` | `tests/matcher.rs` |
| 193 | `tests/test_matcher.py::TestDelegationFloor::test_uv_run_readonly_inner` | `test_delegation_floor::test_uv_run_readonly_inner` | `tests/matcher.rs` |
| 194 | `tests/test_matcher.py::TestDelegationFloor::test_uv_run_dangerous_inner_elevates` | `test_delegation_floor::test_uv_run_dangerous_inner_elevates` | `tests/matcher.rs` |
| 195 | `tests/test_matcher.py::TestDelegationFloor::test_npx_readonly_inner` | `test_delegation_floor::test_npx_readonly_inner` | `tests/matcher.rs` |
| 196 | `tests/test_matcher.py::TestDelegationFloor::test_zsh_c_readonly_inner` | `test_delegation_floor::test_zsh_c_readonly_inner` | `tests/matcher.rs` |
| 197 | `tests/test_matcher.py::TestDelegationFloor::test_nohup_min_classification` | `test_delegation_floor::test_nohup_min_classification` | `tests/matcher.rs` |
| 198 | `tests/test_matcher.py::TestDelegationFloor::test_kubectl_exec_floor` | `test_delegation_floor::test_kubectl_exec_floor` | `tests/matcher.rs` |
| 199 | `tests/test_matcher.py::TestDelegationFloor::test_kubectl_exec_dangerous_inner_elevates` | `test_delegation_floor::test_kubectl_exec_dangerous_inner_elevates` | `tests/matcher.rs` |
| 200 | `tests/test_matcher.py::TestShXargsIntegration::test_xargs_sh_c_readonly` | `test_sh_xargs_integration::test_xargs_sh_c_readonly` | `tests/matcher.rs` |
| 201 | `tests/test_matcher.py::TestGlobalOptionOverrides::test_help_before_subcommand` | `test_global_option_overrides::test_help_before_subcommand` | `tests/matcher.rs` |
| 202 | `tests/test_matcher.py::TestGlobalOptionOverrides::test_help_after_subcommand` | `test_global_option_overrides::test_help_after_subcommand` | `tests/matcher.rs` |
| 203 | `tests/test_matcher.py::TestGlobalOptionOverrides::test_help_on_dangerous_command` | `test_global_option_overrides::test_help_on_dangerous_command` | `tests/matcher.rs` |
| 204 | `tests/test_matcher.py::TestGlobalOptionOverrides::test_h_short_flag` | `test_global_option_overrides::test_h_short_flag` | `tests/matcher.rs` |
| 205 | `tests/test_matcher.py::TestGlobalOptionOverrides::test_h_after_subcommand` | `test_global_option_overrides::test_h_after_subcommand` | `tests/matcher.rs` |
| 206 | `tests/test_matcher.py::TestGlobalOptionOverrides::test_help_does_not_affect_normal_usage` | `test_global_option_overrides::test_help_does_not_affect_normal_usage` | `tests/matcher.rs` |
| 207 | `tests/test_matcher.py::TestGlobalOptionOverrides::test_help_on_command_without_global_options` | `test_global_option_overrides::test_help_on_command_without_global_options` | `tests/matcher.rs` |
| 208 | `tests/test_matcher.py::TestGlobalOptionOverrides::test_help_on_docker_run` | `test_global_option_overrides::test_help_on_docker_run` | `tests/matcher.rs` |
| 209 | `tests/test_matcher.py::TestGlobalOptionOverrides::test_help_on_terraform_apply` | `test_global_option_overrides::test_help_on_terraform_apply` | `tests/matcher.rs` |
| 210 | `tests/test_matcher.py::TestRiskDefaults::test_readonly_command_has_low_risk` | `test_risk::test_readonly_command_has_low_risk` | `tests/matcher.rs` |
| 211 | `tests/test_matcher.py::TestRiskDefaults::test_local_effects_command_has_medium_risk` | `test_risk::test_local_effects_command_has_medium_risk` | `tests/matcher.rs` |
| 212 | `tests/test_matcher.py::TestRiskDefaults::test_local_effects_command_with_explicit_low_risk` | `test_risk::test_local_effects_command_with_explicit_low_risk` | `tests/matcher.rs` |
| 213 | `tests/test_matcher.py::TestRiskDefaults::test_external_effects_command_has_medium_risk` | `test_risk::test_external_effects_command_has_medium_risk` | `tests/matcher.rs` |
| 214 | `tests/test_matcher.py::TestRiskDefaults::test_dangerous_command_has_high_risk` | `test_risk::test_dangerous_command_has_high_risk` | `tests/matcher.rs` |
| 215 | `tests/test_matcher.py::TestRiskDefaults::test_unknown_command_has_high_risk` | `test_risk::test_unknown_command_has_high_risk` | `tests/matcher.rs` |
| 216 | `tests/test_matcher.py::TestRiskDefaults::test_empty_argv_has_high_risk` | `test_risk::test_empty_argv_has_high_risk` | `tests/matcher.rs` |
| 217 | `tests/test_matcher.py::TestRiskOptionOverrides::test_option_classification_override_derives_risk` | `test_risk::test_option_classification_override_derives_risk` | `tests/matcher.rs` |
| 218 | `tests/test_matcher.py::TestRiskOptionOverrides::test_option_dry_run_override_derives_low_risk` | `test_risk::test_option_dry_run_override_derives_low_risk` | `tests/matcher.rs` |
| 219 | `tests/test_matcher.py::TestRiskOptionOverrides::test_strict_mode_unknown_option_high_risk` | `test_risk::test_strict_mode_unknown_option_high_risk` | `tests/matcher.rs` |
| 220 | `tests/test_matcher.py::TestRiskExplicitOverrideInCommandDef::test_explicit_risk_in_command_def` | `test_risk::test_explicit_risk_in_command_def` | `tests/matcher.rs` |
| 221 | `tests/test_matcher.py::TestRiskExplicitOverrideInCommandDef::test_explicit_risk_on_option` | `test_risk::test_explicit_risk_on_option` | `tests/matcher.rs` |
| 222 | `tests/test_matcher.py::TestRiskExplicitOverrideInCommandDef::test_option_with_both_overrides_and_risk` | `test_risk::test_option_with_both_overrides_and_risk` | `tests/matcher.rs` |
| 223 | `tests/test_matcher.py::TestRiskInnerCommands::test_inner_command_elevates_parent_risk` | `test_risk::test_inner_command_elevates_parent_risk` | `tests/matcher.rs` |
| 224 | `tests/test_matcher.py::TestRiskBuiltins::test_cd_low_risk` | `test_risk::test_cd_low_risk` | `tests/matcher.rs` |
| 225 | `tests/test_matcher.py::TestRiskBuiltins::test_eval_high_risk` | `test_risk::test_eval_high_risk` | `tests/matcher.rs` |
| 226 | `tests/test_matcher.py::TestRiskBuiltins::test_test_builtin_low_risk` | `test_risk::test_test_builtin_low_risk` | `tests/matcher.rs` |
| 227 | `tests/test_matcher.py::TestMatchAllSubcommandMode::test_single_known_goal` | `test_match_all_subcommand_mode::test_single_known_goal` | `tests/matcher.rs` |
| 228 | `tests/test_matcher.py::TestMatchAllSubcommandMode::test_multiple_known_goals_all_low` | `test_match_all_subcommand_mode::test_multiple_known_goals_all_low` | `tests/matcher.rs` |
| 229 | `tests/test_matcher.py::TestMatchAllSubcommandMode::test_multiple_goals_max_classification` | `test_match_all_subcommand_mode::test_multiple_goals_max_classification` | `tests/matcher.rs` |
| 230 | `tests/test_matcher.py::TestMatchAllSubcommandMode::test_unrecognized_goal_uses_base` | `test_match_all_subcommand_mode::test_unrecognized_goal_uses_base` | `tests/matcher.rs` |
| 231 | `tests/test_matcher.py::TestMatchAllSubcommandMode::test_no_goals_uses_base` | `test_match_all_subcommand_mode::test_no_goals_uses_base` | `tests/matcher.rs` |
| 232 | `tests/test_matcher.py::TestMatchAllSubcommandMode::test_options_still_processed` | `test_match_all_subcommand_mode::test_options_still_processed` | `tests/matcher.rs` |
| 233 | `tests/test_matcher.py::TestMatchAllSubcommandMode::test_option_override_works` | `test_match_all_subcommand_mode::test_option_override_works` | `tests/matcher.rs` |
| 234 | `tests/test_classifier.py::TestSpecExamples::test_kubectl_pipeline` | `classifier::test_classifier::test_kubectl_pipeline` | `src/classifier.rs` |
| 235 | `tests/test_classifier.py::TestSpecExamples::test_find_xargs_rm` | `classifier::test_classifier::test_find_xargs_rm` | `src/classifier.rs` |
| 236 | `tests/test_classifier.py::TestSpecExamples::test_find_exec_rm` | `classifier::test_classifier::test_find_exec_rm` | `src/classifier.rs` |
| 237 | `tests/test_classifier.py::TestSpecExamples::test_find_exec_grep_is_readonly` | `classifier::test_classifier::test_find_exec_grep_is_readonly` | `src/classifier.rs` |
| 238 | `tests/test_classifier.py::TestSpecExamples::test_kubectl_exec_cat` | `classifier::test_classifier::test_kubectl_exec_cat` | `src/classifier.rs` |
| 239 | `tests/test_classifier.py::TestSpecExamples::test_sh_c_expression` | `classifier::test_classifier::test_sh_c_expression` | `src/classifier.rs` |
| 240 | `tests/test_classifier.py::TestCompositeClassification::test_pipe_both_readonly` | `classifier::test_classifier::test_pipe_both_readonly` | `src/classifier.rs` |
| 241 | `tests/test_classifier.py::TestCompositeClassification::test_pipe_readonly_and_dangerous` | `classifier::test_classifier::test_pipe_readonly_and_dangerous` | `src/classifier.rs` |
| 242 | `tests/test_classifier.py::TestCompositeClassification::test_unknown_trumps_readonly` | `classifier::test_classifier::test_unknown_trumps_readonly` | `src/classifier.rs` |
| 243 | `tests/test_classifier.py::TestCompositeClassification::test_semicolons_both_readonly` | `classifier::test_classifier::test_semicolons_both_readonly` | `src/classifier.rs` |
| 244 | `tests/test_classifier.py::TestRedirectEffects::test_output_redirect_elevates_to_write` | `classifier::test_classifier::test_output_redirect_elevates_to_write` | `src/classifier.rs` |
| 245 | `tests/test_classifier.py::TestRedirectEffects::test_devnull_redirect_no_elevation` | `classifier::test_classifier::test_devnull_redirect_no_elevation` | `src/classifier.rs` |
| 246 | `tests/test_classifier.py::TestRedirectEffects::test_append_redirect_elevates_to_write` | `classifier::test_classifier::test_append_redirect_elevates_to_write` | `src/classifier.rs` |
| 247 | `tests/test_classifier.py::TestRedirectEffects::test_input_redirect_no_elevation` | `classifier::test_classifier::test_input_redirect_no_elevation` | `src/classifier.rs` |
| 248 | `tests/test_classifier.py::TestRedirectEffects::test_stderr_devnull_no_elevation` | `classifier::test_classifier::test_stderr_devnull_no_elevation` | `src/classifier.rs` |
| 249 | `tests/test_classifier.py::TestDirectoryDetection::test_cd_directory` | `classifier::test_classifier::test_cd_directory` | `src/classifier.rs` |
| 250 | `tests/test_classifier.py::TestDirectoryDetection::test_find_directory` | `classifier::test_classifier::test_find_directory` | `src/classifier.rs` |
| 251 | `tests/test_classifier.py::TestDirectoryDetection::test_git_c_directory` | `classifier::test_classifier::test_git_c_directory` | `src/classifier.rs` |
| 252 | `tests/test_classifier.py::TestCatDirectoryDetection::test_cat_extracts_dirname` | `classifier::test_classifier::test_cat_extracts_dirname` | `src/classifier.rs` |
| 253 | `tests/test_classifier.py::TestCatDirectoryDetection::test_cat_no_slash_no_directory` | `classifier::test_classifier::test_cat_no_slash_no_directory` | `src/classifier.rs` |
| 254 | `tests/test_classifier.py::TestCatDirectoryDetection::test_cat_nested_path` | `classifier::test_classifier::test_cat_nested_path` | `src/classifier.rs` |
| 255 | `tests/test_classifier.py::TestBackgrounding::test_background_elevates_to_write` | `classifier::test_classifier::test_background_elevates_to_write` | `src/classifier.rs` |
| 256 | `tests/test_classifier.py::TestEdgeCases::test_empty_string` | `classifier::test_classifier::test_empty_string` | `src/classifier.rs` |
| 257 | `tests/test_classifier.py::TestEdgeCases::test_comment_only` | `classifier::test_classifier::test_comment_only` | `src/classifier.rs` |
| 258 | `tests/test_classifier.py::TestEdgeCases::test_complex_nested_delegation_chain` | `classifier::test_classifier::test_complex_nested_delegation_chain` | `src/classifier.rs` |
| 259 | `tests/test_classifier.py::TestVariableInCommandPosition::test_variable_command` | `classifier::test_classifier::test_variable_command` | `src/classifier.rs` |
| 260 | `tests/test_classifier.py::TestAutoLoadDatabase::test_classify_without_database_arg` | `classifier::test_classifier::test_classify_without_database_arg` | `src/classifier.rs` |
| 261 | `tests/test_classifier.py::TestLsDirectoryDetection::test_ls_directory` | `classifier::test_classifier::test_ls_directory` | `src/classifier.rs` |
| 262 | `tests/test_classifier.py::TestHeadDirectoryDetection::test_head_directory` | `classifier::test_classifier::test_head_directory` | `src/classifier.rs` |
| 263 | `tests/test_classifier.py::TestBackgroundOnReadonly::test_background_elevates_readonly_to_write` | `classifier::test_classifier::test_background_elevates_readonly_to_write` | `src/classifier.rs` |
| 264 | `tests/test_classifier.py::TestInputRedirectNoElevation::test_input_redirect_no_elevation` | `classifier::test_classifier::test_input_redirect_no_elevation_a` | `src/classifier.rs` |
| 265 | `tests/test_classifier.py::TestDevNullRedirectNoElevation::test_devnull_redirect_no_elevation` | `classifier::test_classifier::test_devnull_redirect_no_elevation_a` | `src/classifier.rs` |
| 266 | `tests/test_classifier.py::TestHeredocHerestring::test_heredoc_no_elevation` | `classifier::test_classifier::test_heredoc_no_elevation` | `src/classifier.rs` |
| 267 | `tests/test_classifier.py::TestHeredocHerestring::test_herestring_no_elevation` | `classifier::test_classifier::test_herestring_no_elevation` | `src/classifier.rs` |
| 268 | `tests/test_classifier.py::TestMultipleMixedCommands::test_echo_redirect_then_cat` | `classifier::test_classifier::test_echo_redirect_then_cat` | `src/classifier.rs` |
| 269 | `tests/test_classifier.py::TestNegativeReadonlyNotWrite::test_cat_file_not_write` | `classifier::test_classifier::test_cat_file_not_write` | `src/classifier.rs` |
| 270 | `tests/test_classifier.py::TestNegativeReadonlyNotWrite::test_cat_file_is_readonly` | `classifier::test_classifier::test_cat_file_is_readonly` | `src/classifier.rs` |
| 271 | `tests/test_classifier.py::TestNegativeUnknownCommand::test_unknown_command_is_exactly_unknown` | `classifier::test_classifier::test_unknown_command_is_exactly_unknown` | `src/classifier.rs` |
| 272 | `tests/test_classifier.py::TestNegativeForceWithLease::test_force_with_lease_not_dangerous` | `classifier::test_classifier::test_force_with_lease_not_dangerous` | `src/classifier.rs` |
| 273 | `tests/test_classifier.py::TestReadBuiltin::test_read_is_readonly` | `classifier::test_classifier::test_read_is_readonly` | `src/classifier.rs` |
| 274 | `tests/test_classifier.py::TestFdToFdRedirectClassification::test_stderr_to_stdout_no_write_elevation` | `classifier::test_classifier::test_stderr_to_stdout_no_write_elevation` | `src/classifier.rs` |
| 275 | `tests/test_classifier.py::TestFdToFdRedirectClassification::test_stderr_to_file_elevates_to_write` | `classifier::test_classifier::test_stderr_to_file_elevates_to_write` | `src/classifier.rs` |
| 276 | `tests/test_classifier.py::TestParseWarningsIntegration::test_syntax_error_populates_warnings` | `classifier::test_classifier::test_syntax_error_populates_warnings` | `src/classifier.rs` |
| 277 | `tests/test_classifier.py::TestMalformedBashDefaultsToUnknown::test_malformed_bash_classifies_as_unknown` | `classifier::test_classifier::test_malformed_bash_classifies_as_unknown` | `src/classifier.rs` |
| 278 | `tests/test_classifier.py::TestMalformedBashDefaultsToUnknown::test_empty_string_stays_readonly` | `classifier::test_classifier::test_empty_string_stays_readonly` | `src/classifier.rs` |
| 279 | `tests/test_classifier.py::TestDevTcpUdpDetection::test_cat_dev_tcp_in_argv` | `classifier::test_classifier::test_cat_dev_tcp_in_argv` | `src/classifier.rs` |
| 280 | `tests/test_classifier.py::TestDevTcpUdpDetection::test_echo_redirect_to_dev_tcp` | `classifier::test_classifier::test_echo_redirect_to_dev_tcp` | `src/classifier.rs` |
| 281 | `tests/test_classifier.py::TestDevTcpUdpDetection::test_cat_input_redirect_from_dev_tcp` | `classifier::test_classifier::test_cat_input_redirect_from_dev_tcp` | `src/classifier.rs` |
| 282 | `tests/test_classifier.py::TestDevTcpUdpDetection::test_dev_udp_in_argv` | `classifier::test_classifier::test_dev_udp_in_argv` | `src/classifier.rs` |
| 283 | `tests/test_classifier.py::TestCommandBuiltinDelegation::test_command_delegates_to_cat` | `classifier::test_classifier::test_command_delegates_to_cat` | `src/classifier.rs` |
| 284 | `tests/test_classifier.py::TestCommandBuiltinDelegation::test_command_delegates_to_rm` | `classifier::test_classifier::test_command_delegates_to_rm` | `src/classifier.rs` |
| 285 | `tests/test_classifier.py::TestCommandBuiltinDelegation::test_builtin_delegates_to_echo` | `classifier::test_classifier::test_builtin_delegates_to_echo` | `src/classifier.rs` |
| 286 | `tests/test_classifier.py::TestPathPrefixStripping::test_usr_bin_cat_resolves` | `classifier::test_classifier::test_usr_bin_cat_resolves` | `src/classifier.rs` |
| 287 | `tests/test_classifier.py::TestPathPrefixStripping::test_usr_bin_rm_resolves` | `classifier::test_classifier::test_usr_bin_rm_resolves` | `src/classifier.rs` |
| 288 | `tests/test_classifier.py::TestPathPrefixStripping::test_relative_script_stays_unknown` | `classifier::test_classifier::test_relative_script_stays_unknown` | `src/classifier.rs` |
| 289 | `tests/test_classifier.py::TestAmpersandRedirectClassification::test_ampersand_redirect_to_file_is_write` | `classifier::test_classifier::test_ampersand_redirect_to_file_is_write` | `src/classifier.rs` |
| 290 | `tests/test_classifier.py::TestAmpersandRedirectClassification::test_ampersand_redirect_to_devnull_is_readonly` | `classifier::test_classifier::test_ampersand_redirect_to_devnull_is_readonly` | `src/classifier.rs` |
| 291 | `tests/test_classifier.py::TestPushdDirectoryDetection::test_pushd_directory` | `classifier::test_classifier::test_pushd_directory` | `src/classifier.rs` |
| 292 | `tests/test_classifier.py::TestTeeClassification::test_tee_is_write` | `classifier::test_classifier::test_tee_is_write` | `src/classifier.rs` |
| 293 | `tests/test_classifier.py::TestIsSystemPath::test_etc` | `classifier::test_classifier::test_etc` | `src/classifier.rs` |
| 294 | `tests/test_classifier.py::TestIsSystemPath::test_usr` | `classifier::test_classifier::test_usr` | `src/classifier.rs` |
| 295 | `tests/test_classifier.py::TestIsSystemPath::test_tmp_safe` | `classifier::test_classifier::test_tmp_safe` | `src/classifier.rs` |
| 296 | `tests/test_classifier.py::TestIsSystemPath::test_home_safe` | `classifier::test_classifier::test_home_safe` | `src/classifier.rs` |
| 297 | `tests/test_classifier.py::TestIsSystemPath::test_var_tmp_safe` | `classifier::test_classifier::test_var_tmp_safe` | `src/classifier.rs` |
| 298 | `tests/test_classifier.py::TestIsSystemPath::test_var_log_system` | `classifier::test_classifier::test_var_log_system` | `src/classifier.rs` |
| 299 | `tests/test_classifier.py::TestIsSystemPath::test_dev_null_safe` | `classifier::test_classifier::test_dev_null_safe` | `src/classifier.rs` |
| 300 | `tests/test_classifier.py::TestIsSystemPath::test_dev_sda_system` | `classifier::test_classifier::test_dev_sda_system` | `src/classifier.rs` |
| 301 | `tests/test_classifier.py::TestIsSystemPath::test_relative_path` | `classifier::test_classifier::test_relative_path` | `src/classifier.rs` |
| 302 | `tests/test_classifier.py::TestIsSystemPath::test_boot` | `classifier::test_classifier::test_boot` | `src/classifier.rs` |
| 303 | `tests/test_classifier.py::TestIsSystemPath::test_proc` | `classifier::test_classifier::test_proc` | `src/classifier.rs` |
| 304 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_write_to_etc_is_dangerous` | `classifier::test_classifier::test_write_to_etc_is_dangerous` | `src/classifier.rs` |
| 305 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_write_to_usr_is_dangerous` | `classifier::test_classifier::test_write_to_usr_is_dangerous` | `src/classifier.rs` |
| 306 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_read_from_etc_is_readonly` | `classifier::test_classifier::test_read_from_etc_is_readonly` | `src/classifier.rs` |
| 307 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_read_from_usr_is_readonly` | `classifier::test_classifier::test_read_from_usr_is_readonly` | `src/classifier.rs` |
| 308 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_write_to_tmp_not_elevated` | `classifier::test_classifier::test_write_to_tmp_not_elevated` | `src/classifier.rs` |
| 309 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_write_to_home_not_elevated` | `classifier::test_classifier::test_write_to_home_not_elevated` | `src/classifier.rs` |
| 310 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_write_to_var_tmp_not_elevated` | `classifier::test_classifier::test_write_to_var_tmp_not_elevated` | `src/classifier.rs` |
| 311 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_write_to_var_log_is_dangerous` | `classifier::test_classifier::test_write_to_var_log_is_dangerous` | `src/classifier.rs` |
| 312 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_redirect_to_etc_is_dangerous` | `classifier::test_classifier::test_redirect_to_etc_is_dangerous` | `src/classifier.rs` |
| 313 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_redirect_to_tmp_not_elevated` | `classifier::test_classifier::test_redirect_to_tmp_not_elevated` | `src/classifier.rs` |
| 314 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_mkdir_in_etc_is_dangerous` | `classifier::test_classifier::test_mkdir_in_etc_is_dangerous` | `src/classifier.rs` |
| 315 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_touch_in_opt_is_dangerous` | `classifier::test_classifier::test_touch_in_opt_is_dangerous` | `src/classifier.rs` |
| 316 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_chmod_in_usr_is_dangerous` | `classifier::test_classifier::test_chmod_in_usr_is_dangerous` | `src/classifier.rs` |
| 317 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_rm_is_already_dangerous` | `classifier::test_classifier::test_rm_is_already_dangerous` | `src/classifier.rs` |
| 318 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_dev_null_safe` | `classifier::test_classifier::test_dev_null_safe_class` | `src/classifier.rs` |
| 319 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_relative_path_not_elevated` | `classifier::test_classifier::test_relative_path_not_elevated` | `src/classifier.rs` |
| 320 | `tests/test_classifier.py::TestSystemDirectoryClassification::test_git_commit_with_system_path_message_not_elevated` | `classifier::test_classifier::test_git_commit_with_system_path_message_not_elevated` | `src/classifier.rs` |
| 321 | `tests/test_classifier.py::TestUserCommandOverride::test_custom_command_classifies_correctly` | `classifier::test_classifier::test_custom_command_classifies_correctly` | `src/classifier.rs` |
| 322 | `tests/test_classifier.py::TestRiskSystemPathElevation::test_write_to_system_path_high_risk` | `classifier::test_classifier::test_write_to_system_path_high_risk` | `src/classifier.rs` |
| 323 | `tests/test_classifier.py::TestRiskSystemPathElevation::test_redirect_to_system_path_high_risk` | `classifier::test_classifier::test_redirect_to_system_path_high_risk` | `src/classifier.rs` |
| 324 | `tests/test_classifier.py::TestRiskRedirectElevation::test_output_redirect_medium_risk` | `classifier::test_classifier::test_output_redirect_medium_risk` | `src/classifier.rs` |
| 325 | `tests/test_classifier.py::TestRiskRedirectElevation::test_devnull_redirect_stays_low_risk` | `classifier::test_classifier::test_devnull_redirect_stays_low_risk` | `src/classifier.rs` |
| 326 | `tests/test_classifier.py::TestRiskDevTcpUdp::test_dev_tcp_high_risk` | `classifier::test_classifier::test_dev_tcp_high_risk` | `src/classifier.rs` |
| 327 | `tests/test_classifier.py::TestRiskDevTcpUdp::test_dev_udp_high_risk` | `classifier::test_classifier::test_dev_udp_high_risk` | `src/classifier.rs` |
| 328 | `tests/test_classifier.py::TestRiskDevTcpUdp::test_redirect_to_dev_tcp_high_risk` | `classifier::test_classifier::test_redirect_to_dev_tcp_high_risk` | `src/classifier.rs` |
| 329 | `tests/test_classifier.py::TestRiskBackgrounding::test_backgrounding_at_least_medium_risk` | `classifier::test_classifier::test_backgrounding_at_least_medium_risk` | `src/classifier.rs` |
| 330 | `tests/test_classifier.py::TestRiskExpressionAggregation::test_expression_risk_is_max_across_commands` | `classifier::test_classifier::test_expression_risk_is_max_across_commands` | `src/classifier.rs` |
| 331 | `tests/test_classifier.py::TestRiskExpressionAggregation::test_expression_all_readonly_low_risk` | `classifier::test_classifier::test_expression_all_readonly_low_risk` | `src/classifier.rs` |
| 332 | `tests/test_classifier.py::TestRiskExpressionAggregation::test_empty_expression_low_risk` | `classifier::test_classifier::test_empty_expression_low_risk` | `src/classifier.rs` |
| 333 | `tests/test_classifier.py::TestRiskExpressionAggregation::test_variable_in_command_position_high_risk` | `classifier::test_classifier::test_variable_in_command_position_high_risk` | `src/classifier.rs` |
| 334 | `tests/test_classifier.py::TestRiskExpressionAggregation::test_malformed_bash_high_risk` | `classifier::test_classifier::test_malformed_bash_high_risk` | `src/classifier.rs` |
| 335 | `tests/test_classifier.py::TestFilePathDetection::test_write_to_tmp_stays_low_risk` | `classifier::test_classifier::test_write_to_tmp_stays_low_risk` | `src/classifier.rs` |
| 336 | `tests/test_classifier.py::TestFilePathDetection::test_write_to_var_tmp_stays_low_risk` | `classifier::test_classifier::test_write_to_var_tmp_stays_low_risk` | `src/classifier.rs` |
| 337 | `tests/test_classifier.py::TestFilePathDetection::test_write_to_home_elevates_risk` | `classifier::test_classifier::test_write_to_home_elevates_risk` | `src/classifier.rs` |
| 338 | `tests/test_classifier.py::TestFilePathDetection::test_mixed_tmp_and_home_elevates_risk` | `classifier::test_classifier::test_mixed_tmp_and_home_elevates_risk` | `src/classifier.rs` |
| 339 | `tests/test_classifier.py::TestFilePathDetection::test_read_redirect_detected` | `classifier::test_classifier::test_read_redirect_detected` | `src/classifier.rs` |
| 340 | `tests/test_classifier.py::TestFilePathDetection::test_read_and_write_redirect` | `classifier::test_classifier::test_read_and_write_redirect` | `src/classifier.rs` |
| 341 | `tests/test_classifier.py::TestFilePathDetection::test_dev_null_excluded_from_write_paths` | `classifier::test_classifier::test_dev_null_excluded_from_write_paths` | `src/classifier.rs` |
| 342 | `tests/test_classifier.py::TestFilePathDetection::test_heredoc_not_in_read_paths` | `classifier::test_classifier::test_heredoc_not_in_read_paths` | `src/classifier.rs` |
| 343 | `tests/test_classifier.py::TestFilePathDetection::test_expression_level_aggregation` | `classifier::test_classifier::test_expression_level_aggregation` | `src/classifier.rs` |
| 344 | `tests/test_classifier.py::TestAliasOfClassification::test_alias_resolves_to_target` | `classifier::test_classifier::test_alias_resolves_to_target` | `src/classifier.rs` |
| 345 | `tests/test_classifier.py::TestAliasOfClassification::test_alias_resolves_dangerous_subcommand` | `classifier::test_classifier::test_alias_resolves_dangerous_subcommand` | `src/classifier.rs` |
| 346 | `tests/test_classifier.py::TestAliasOfClassification::test_dangling_alias_raises` | `classifier::test_classifier::test_dangling_alias_raises` | `src/classifier.rs` |
| 347 | `tests/test_classifier.py::TestAliasOfClassification::test_alias_chain_too_deep_raises` | `classifier::test_classifier::test_alias_chain_too_deep_raises` | `src/classifier.rs` |
| 348 | `tests/test_real_world.py::TestGitWorkflows::test_git_status` | `test_git_status` | `tests/real_world.rs` |
| 349 | `tests/test_real_world.py::TestGitWorkflows::test_git_status_short` | `test_git_status_short` | `tests/real_world.rs` |
| 350 | `tests/test_real_world.py::TestGitWorkflows::test_git_diff_stat` | `test_git_diff_stat` | `tests/real_world.rs` |
| 351 | `tests/test_real_world.py::TestGitWorkflows::test_git_diff_cached` | `test_git_diff_cached` | `tests/real_world.rs` |
| 352 | `tests/test_real_world.py::TestGitWorkflows::test_git_diff_cached_stat` | `test_git_diff_cached_stat` | `tests/real_world.rs` |
| 353 | `tests/test_real_world.py::TestGitWorkflows::test_git_log_oneline` | `test_git_log_oneline` | `tests/real_world.rs` |
| 354 | `tests/test_real_world.py::TestGitWorkflows::test_git_log_range` | `test_git_log_range` | `tests/real_world.rs` |
| 355 | `tests/test_real_world.py::TestGitWorkflows::test_git_diff_range` | `test_git_diff_range` | `tests/real_world.rs` |
| 356 | `tests/test_real_world.py::TestGitWorkflows::test_git_show_historical` | `test_git_show_historical` | `tests/real_world.rs` |
| 357 | `tests/test_real_world.py::TestGitWorkflows::test_git_add_and_commit` | `test_git_add_and_commit` | `tests/real_world.rs` |
| 358 | `tests/test_real_world.py::TestGitWorkflows::test_git_add_diff_cached` | `test_git_add_diff_cached` | `tests/real_world.rs` |
| 359 | `tests/test_real_world.py::TestGitWorkflows::test_git_amend_no_edit` | `test_git_amend_no_edit` | `tests/real_world.rs` |
| 360 | `tests/test_real_world.py::TestGitWorkflows::test_git_amend_and_force_push` | `test_git_amend_and_force_push` | `tests/real_world.rs` |
| 361 | `tests/test_real_world.py::TestGitWorkflows::test_git_push_force` | `test_git_push_force` | `tests/real_world.rs` |
| 362 | `tests/test_real_world.py::TestGitWorkflows::test_git_rebase_continue` | `test_git_rebase_continue` | `tests/real_world.rs` |
| 363 | `tests/test_real_world.py::TestGitWorkflows::test_git_reset_soft` | `test_git_reset_soft` | `tests/real_world.rs` |
| 364 | `tests/test_real_world.py::TestGitWorkflows::test_git_c_flag` | `test_git_c_flag` | `tests/real_world.rs` |
| 365 | `tests/test_real_world.py::TestGitWorkflows::test_env_prefix_git_rebase` | `test_env_prefix_git_rebase` | `tests/real_world.rs` |
| 366 | `tests/test_real_world.py::TestGitWorkflows::test_git_rm_during_rebase` | `test_git_rm_during_rebase` | `tests/real_world.rs` |
| 367 | `tests/test_real_world.py::TestBuildAndTest::test_mvnw_package_with_grep` | `test_mvnw_package_with_grep` | `tests/real_world.rs` |
| 368 | `tests/test_real_world.py::TestBuildAndTest::test_mvnw_with_tail` | `test_mvnw_with_tail` | `tests/real_world.rs` |
| 369 | `tests/test_real_world.py::TestBuildAndTest::test_pytest_verbose` | `test_pytest_verbose` | `tests/real_world.rs` |
| 370 | `tests/test_real_world.py::TestBuildAndTest::test_npm_test` | `test_npm_test` | `tests/real_world.rs` |
| 371 | `tests/test_real_world.py::TestBuildAndTest::test_npm_run_build` | `test_npm_run_build` | `tests/real_world.rs` |
| 372 | `tests/test_real_world.py::TestBuildAndTest::test_npm_install` | `test_npm_install` | `tests/real_world.rs` |
| 373 | `tests/test_real_world.py::TestBuildAndTest::test_ruff_check_and_format` | `test_ruff_check_and_format` | `tests/real_world.rs` |
| 374 | `tests/test_real_world.py::TestBuildAndTest::test_make_with_target` | `test_make_with_target` | `tests/real_world.rs` |
| 375 | `tests/test_real_world.py::TestBuildAndTest::test_make_dry_run` | `test_make_dry_run` | `tests/real_world.rs` |
| 376 | `tests/test_real_world.py::TestPipesAndGrep::test_grep_with_head` | `test_grep_with_head` | `tests/real_world.rs` |
| 377 | `tests/test_real_world.py::TestPipesAndGrep::test_grep_case_insensitive_with_tail` | `test_grep_case_insensitive_with_tail` | `tests/real_world.rs` |
| 378 | `tests/test_real_world.py::TestPipesAndGrep::test_grep_with_line_numbers_sort_uniq` | `test_grep_with_line_numbers_sort_uniq` | `tests/real_world.rs` |
| 379 | `tests/test_real_world.py::TestPipesAndGrep::test_grep_context_lines` | `test_grep_context_lines` | `tests/real_world.rs` |
| 380 | `tests/test_real_world.py::TestPipesAndGrep::test_grep_before_and_after` | `test_grep_before_and_after` | `tests/real_world.rs` |
| 381 | `tests/test_real_world.py::TestPipesAndGrep::test_grep_inverse_pipe` | `test_grep_inverse_pipe` | `tests/real_world.rs` |
| 382 | `tests/test_real_world.py::TestPipesAndGrep::test_cat_pipe_head` | `test_cat_pipe_head` | `tests/real_world.rs` |
| 383 | `tests/test_real_world.py::TestPipesAndGrep::test_find_pipe_head` | `test_find_pipe_head` | `tests/real_world.rs` |
| 384 | `tests/test_real_world.py::TestPipesAndGrep::test_ls_pipe_tail` | `test_ls_pipe_tail` | `tests/real_world.rs` |
| 385 | `tests/test_real_world.py::TestPipesAndGrep::test_sort_pipe_uniq_count_sort` | `test_sort_pipe_uniq_count_sort` | `tests/real_world.rs` |
| 386 | `tests/test_real_world.py::TestPipesAndGrep::test_ps_pipe_grep` | `test_ps_pipe_grep` | `tests/real_world.rs` |
| 387 | `tests/test_real_world.py::TestPipesAndGrep::test_wc_l_pipeline` | `test_wc_l_pipeline` | `tests/real_world.rs` |
| 388 | `tests/test_real_world.py::TestCommandChains::test_and_chain_readonly` | `test_and_chain_readonly` | `tests/real_world.rs` |
| 389 | `tests/test_real_world.py::TestCommandChains::test_and_chain_mixed` | `test_and_chain_mixed` | `tests/real_world.rs` |
| 390 | `tests/test_real_world.py::TestCommandChains::test_or_chain_with_echo_fallback` | `test_or_chain_with_echo_fallback` | `tests/real_world.rs` |
| 391 | `tests/test_real_world.py::TestCommandChains::test_semicolon_chain` | `test_semicolon_chain` | `tests/real_world.rs` |
| 392 | `tests/test_real_world.py::TestCommandChains::test_docker_cleanup_chain` | `test_docker_cleanup_chain` | `tests/real_world.rs` |
| 393 | `tests/test_real_world.py::TestCommandChains::test_cd_and_command` | `test_cd_and_command` | `tests/real_world.rs` |
| 394 | `tests/test_real_world.py::TestCommandChains::test_mkdir_or_fallback` | `test_mkdir_or_fallback` | `tests/real_world.rs` |
| 395 | `tests/test_real_world.py::TestRedirects::test_stderr_to_stdout_pipe` | `test_stderr_to_stdout_pipe` | `tests/real_world.rs` |
| 396 | `tests/test_real_world.py::TestRedirects::test_stderr_to_dev_null` | `test_stderr_to_dev_null` | `tests/real_world.rs` |
| 397 | `tests/test_real_world.py::TestRedirects::test_redirect_to_file` | `test_redirect_to_file` | `tests/real_world.rs` |
| 398 | `tests/test_real_world.py::TestRedirects::test_append_redirect` | `test_append_redirect` | `tests/real_world.rs` |
| 399 | `tests/test_real_world.py::TestRedirects::test_unzip_with_fallback` | `test_unzip_with_fallback` | `tests/real_world.rs` |
| 400 | `tests/test_real_world.py::TestHeredocs::test_git_commit_with_heredoc` | `test_git_commit_with_heredoc` | `tests/real_world.rs` |
| 401 | `tests/test_real_world.py::TestHeredocs::test_cat_heredoc` | `test_cat_heredoc` | `tests/real_world.rs` |
| 402 | `tests/test_real_world.py::TestVariableExpansion::test_variable_in_command_position` | `test_variable_in_command_position` | `tests/real_world.rs` |
| 403 | `tests/test_real_world.py::TestVariableExpansion::test_env_prefix_command` | `test_env_prefix_command` | `tests/real_world.rs` |
| 404 | `tests/test_real_world.py::TestVariableExpansion::test_date_format` | `test_date_format` | `tests/real_world.rs` |
| 405 | `tests/test_real_world.py::TestKubernetes::test_kubectl_get_wide` | `test_kubectl_get_wide` | `tests/real_world.rs` |
| 406 | `tests/test_real_world.py::TestKubernetes::test_kubectl_get_deployment` | `test_kubectl_get_deployment` | `tests/real_world.rs` |
| 407 | `tests/test_real_world.py::TestKubernetes::test_kubectl_logs_tail` | `test_kubectl_logs_tail` | `tests/real_world.rs` |
| 408 | `tests/test_real_world.py::TestKubernetes::test_kubectl_exec_cat` | `test_kubectl_exec_cat` | `tests/real_world.rs` |
| 409 | `tests/test_real_world.py::TestKubernetes::test_kubectl_apply` | `test_kubectl_apply` | `tests/real_world.rs` |
| 410 | `tests/test_real_world.py::TestKubernetes::test_kubectl_apply_dry_run` | `test_kubectl_apply_dry_run` | `tests/real_world.rs` |
| 411 | `tests/test_real_world.py::TestDockerPatterns::test_docker_ps` | `test_docker_ps` | `tests/real_world.rs` |
| 412 | `tests/test_real_world.py::TestDockerPatterns::test_docker_run` | `test_docker_run` | `tests/real_world.rs` |
| 413 | `tests/test_real_world.py::TestDockerPatterns::test_docker_run_with_volume` | `test_docker_run_with_volume` | `tests/real_world.rs` |
| 414 | `tests/test_real_world.py::TestDockerPatterns::test_docker_images` | `test_docker_images` | `tests/real_world.rs` |
| 415 | `tests/test_real_world.py::TestDockerPatterns::test_docker_system_prune` | `test_docker_system_prune` | `tests/real_world.rs` |
| 416 | `tests/test_real_world.py::TestFileOperations::test_find_with_maxdepth` | `test_find_with_maxdepth` | `tests/real_world.rs` |
| 417 | `tests/test_real_world.py::TestFileOperations::test_find_type_f_pipe_head` | `test_find_type_f_pipe_head` | `tests/real_world.rs` |
| 418 | `tests/test_real_world.py::TestFileOperations::test_find_delete` | `test_find_delete` | `tests/real_world.rs` |
| 419 | `tests/test_real_world.py::TestFileOperations::test_xargs_grep` | `test_xargs_grep` | `tests/real_world.rs` |
| 420 | `tests/test_real_world.py::TestFileOperations::test_xargs_rm` | `test_xargs_rm` | `tests/real_world.rs` |
| 421 | `tests/test_real_world.py::TestFileOperations::test_cp_to_tmp` | `test_cp_to_tmp` | `tests/real_world.rs` |
| 422 | `tests/test_real_world.py::TestFileOperations::test_cp_to_etc` | `test_cp_to_etc` | `tests/real_world.rs` |
| 423 | `tests/test_real_world.py::TestFileOperations::test_mkdir_p` | `test_mkdir_p` | `tests/real_world.rs` |
| 424 | `tests/test_real_world.py::TestComplexRealWorld::test_source_and_build` | `test_source_and_build` | `tests/real_world.rs` |
| 425 | `tests/test_real_world.py::TestComplexRealWorld::test_conditional_test` | `test_conditional_test` | `tests/real_world.rs` |
| 426 | `tests/test_real_world.py::TestComplexRealWorld::test_double_bracket_test` | `test_double_bracket_test` | `tests/real_world.rs` |
| 427 | `tests/test_real_world.py::TestComplexRealWorld::test_for_loop_echo` | `test_for_loop_echo` | `tests/real_world.rs` |
| 428 | `tests/test_real_world.py::TestComplexRealWorld::test_while_read_echo` | `test_while_read_echo` | `tests/real_world.rs` |
| 429 | `tests/test_real_world.py::TestComplexRealWorld::test_if_then_cat` | `test_if_then_cat` | `tests/real_world.rs` |
| 430 | `tests/test_real_world.py::TestComplexRealWorld::test_sudo_with_pipe` | `test_sudo_with_pipe` | `tests/real_world.rs` |
| 431 | `tests/test_real_world.py::TestComplexRealWorld::test_git_log_pipe_xargs_show` | `test_git_log_pipe_xargs_show` | `tests/real_world.rs` |
| 432 | `tests/test_real_world.py::TestBuildTools::test_mvn_clean_install` | `test_mvn_clean_install` | `tests/real_world.rs` |
| 433 | `tests/test_real_world.py::TestBuildTools::test_mvn_clean_deploy` | `test_mvn_clean_deploy` | `tests/real_world.rs` |
| 434 | `tests/test_real_world.py::TestBuildTools::test_mvn_custom_plugin_goal` | `test_mvn_custom_plugin_goal` | `tests/real_world.rs` |
| 435 | `tests/test_real_world.py::TestBuildTools::test_gradle_clean_build_test` | `test_gradle_clean_build_test` | `tests/real_world.rs` |
| 436 | `tests/test_real_world.py::TestBuildTools::test_gradle_publish` | `test_gradle_publish` | `tests/real_world.rs` |
| 437 | `tests/test_real_world.py::TestBuildTools::test_gradle_dry_run` | `test_gradle_dry_run` | `tests/real_world.rs` |
| 438 | `tests/test_cli_serialization.py::TestRedirectToDict::test_redirect_affects_classification` | `cli::test_cli_serialization::test_redirect_affects_classification` | `src/cli.rs` |
| 439 | `tests/test_cli_serialization.py::TestRedirectToDict::test_redirect_does_not_affect_classification` | `cli::test_cli_serialization::test_redirect_does_not_affect_classification` | `src/cli.rs` |
| 440 | `tests/test_cli_serialization.py::TestCommandToDict::test_all_optional_fields_populated` | `cli::test_cli_serialization::test_all_optional_fields_populated` | `src/cli.rs` |
| 441 | `tests/test_cli_serialization.py::TestCommandToDict::test_all_optional_fields_empty_omitted` | `cli::test_cli_serialization::test_all_optional_fields_empty_omitted` | `src/cli.rs` |
| 442 | `tests/test_cli_serialization.py::TestInnerCommandToDict::test_with_delegation_mode_and_source` | `cli::test_cli_serialization::test_with_delegation_mode_and_source` | `src/cli.rs` |
| 443 | `tests/test_cli_serialization.py::TestResultToDict::test_with_redirects_and_parse_warnings` | `cli::test_cli_serialization::test_with_redirects_and_parse_warnings` | `src/cli.rs` |
| 444 | `tests/test_cli_serialization.py::TestResultToDict::test_empty_redirects_and_warnings_omitted` | `cli::test_cli_serialization::test_empty_redirects_and_warnings_omitted` | `src/cli.rs` |
| 445 | `tests/test_cli_serialization.py::TestClassificationSerializesAsString::test_classification_enum_serializes_as_string` | `cli::test_cli_serialization::test_classification_enum_serializes_as_string` | `src/cli.rs` |
| 446 | `tests/test_cli_serialization.py::TestClassificationSerializesAsString::test_all_classification_values_serialize_correctly` | `cli::test_cli_serialization::test_all_classification_values_serialize_correctly` | `src/cli.rs` |
| 447 | `tests/test_cli_serialization.py::TestRiskSerialization::test_risk_field_in_command_dict` | `cli::test_cli_serialization::test_risk_field_in_command_dict` | `src/cli.rs` |
| 448 | `tests/test_cli_serialization.py::TestRiskSerialization::test_risk_field_in_inner_command_dict` | `cli::test_cli_serialization::test_risk_field_in_inner_command_dict` | `src/cli.rs` |
| 449 | `tests/test_cli_serialization.py::TestRiskSerialization::test_risk_field_in_result_dict` | `cli::test_cli_serialization::test_risk_field_in_result_dict` | `src/cli.rs` |
| 450 | `tests/test_cli_serialization.py::TestRiskSerialization::test_all_risk_values_serialize_correctly` | `cli::test_cli_serialization::test_all_risk_values_serialize_correctly` | `src/cli.rs` |
| 451 | `tests/test_cli_serialization.py::TestWriteReadPaths::test_write_paths_in_command_dict` | `cli::test_cli_serialization::test_write_paths_in_command_dict` | `src/cli.rs` |
| 452 | `tests/test_cli_serialization.py::TestWriteReadPaths::test_read_paths_in_command_dict` | `cli::test_cli_serialization::test_read_paths_in_command_dict` | `src/cli.rs` |
| 453 | `tests/test_cli_serialization.py::TestWriteReadPaths::test_paths_omitted_when_empty` | `cli::test_cli_serialization::test_empty_optional_lists_omitted` | `src/cli.rs` |
| 454 | `tests/test_cli_serialization.py::TestWriteReadPaths::test_expression_write_read_paths` | `cli::test_cli_serialization::test_expression_write_read_paths` | `src/cli.rs` |
| 455 | `tests/test_cli_serialization.py::TestWriteReadPaths::test_expression_paths_omitted_when_empty` | `cli::test_cli_serialization::test_expression_paths_omitted_when_empty` | `src/cli.rs` |
| 456 | `tests/test_cli.py::TestCliBasic::test_ls_readonly` | `test_ls_readonly` | `tests/cli.rs` |
| 457 | `tests/test_cli.py::TestCliBasic::test_git_push_force_dangerous` | `test_git_push_force_dangerous` | `tests/cli.rs` |
| 458 | `tests/test_cli.py::TestCliBasic::test_empty_string_exits_with_1` | `test_empty_string_exits_with_1` | `tests/cli.rs` |
| 459 | `tests/test_cli.py::TestCliBasic::test_json_output_has_required_fields` | `test_json_output_has_required_fields` | `tests/cli.rs` |
| 460 | `tests/test_cli.py::TestCliDelegation::test_sudo_ls_has_inner_commands` | `test_sudo_ls_has_inner_commands` | `tests/cli.rs` |
| 461 | `tests/test_cli.py::TestCliRedirect::test_redirect_appears_in_json` | `test_redirect_appears_in_json` | `tests/cli.rs` |
| 462 | `tests/test_cli.py::TestCliClassificationReason::test_classification_reason_in_output` | `test_classification_reason_in_output` | `tests/cli.rs` |
| 463 | `tests/test_schema.py::test_yaml_validates_against_schema` | `test_yaml_validates_against_schema` | `tests/schema.rs` |

## Extra Rust regression/helper tests

These tests are additional coverage beyond the one-for-one Python port. They cover the golden replay gate, CLI matrix rows, user override scenarios, Rust-only helper seams, and serialization edge cases introduced by the native implementation.

| Rust test id | Rust file |
|---|---|
| `cli::test_cli_serialization::test_inner_optional_fields` | `src/cli.rs` |
| `cli::test_cli_serialization::test_key_order` | `src/cli.rs` |
| `cli::test_cli_serialization::test_py_ascii_formatter_matches_python_escape_set` | `src/cli.rs` |
| `cli::test_cli_serialization::test_py_strip_python_whitespace_edges` | `src/cli.rs` |
| `database::test_database::test_alias_of::test_alias_with_missing_target_loads_as_raw_alias` | `src/database.rs` |
| `database::test_database::test_empty_and_non_dict_yaml_files::test_non_mapping_option_body_raises_on_access` | `src/database.rs` |
| `database::test_database::test_empty_and_non_dict_yaml_files::test_non_mapping_subcommand_body_raises_on_access` | `src/database.rs` |
| `database::test_database::test_empty_and_non_dict_yaml_files::test_truthy_non_mapping_options_raises_on_access` | `src/database.rs` |
| `database::test_database::test_empty_and_non_dict_yaml_files::test_truthy_non_mapping_subcommands_raises_on_access` | `src/database.rs` |
| `database::test_database::test_user_commands_dir::test_empty_config_dir_falls_back_to_home_config` | `src/database.rs` |
| `golden_corpus_replays_byte_identically` | `tests/golden_replay.rs` |
| `matcher::tests::test_is_terminator` | `src/matcher.rs` |
| `matcher::tests::test_py_basename` | `src/matcher.rs` |
| `matcher::tests::test_strip_quotes` | `src/matcher.rs` |
| `matrix_empty_input` | `tests/cli_matrix.rs` |
| `matrix_help_and_version` | `tests/cli_matrix.rs` |
| `matrix_internal_error_invalid_user_yaml` | `tests/cli_matrix.rs` |
| `matrix_slow_stdin_after_first_byte_waits_for_eof` | `tests/cli_matrix.rs` |
| `matrix_success_piped_compound_subshell_redirect_outputs_match_python_fixtures` | `tests/cli_matrix.rs` |
| `matrix_timeout_without_stdin_data` | `tests/cli_matrix.rs` |
| `parser::test_parser::test_py_repr::test_control_escapes` | `src/parser.rs` |
| `parser::test_parser::test_py_repr::test_single_quote_preference` | `src/parser.rs` |
| `parser::test_parser::test_recursion_depth::test_deep_nesting_returns_parse_error` | `src/parser.rs` |
| `parser::test_parser::test_recursion_depth::test_moderate_nesting_still_parses` | `src/parser.rs` |
| `user_yaml_adds_new_command_from_custom_config_root` | `tests/user_overrides.rs` |
| `user_yaml_overrides_builtin_by_filename_stem` | `tests/user_overrides.rs` |

Ported Python tests: 463. Extra Rust tests: 26. Total Rust tests listed by Cargo: 489.
