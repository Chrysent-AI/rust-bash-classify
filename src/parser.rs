// SPDX-FileCopyrightText: 2026 Chrysent LLC
// SPDX-License-Identifier: MIT
//! Bash expression parser using tree-sitter-bash.

use std::{error::Error, fmt};

use tree_sitter::{Node, Parser};

use crate::models::{CommandInvocation, Redirect};

// Mirrors Python's RecursionError text so the CLI can preserve the Python error class/message.
const MAX_WALK_DEPTH: usize = 256;
const RECURSION_ERROR_MESSAGE: &str = "maximum recursion depth exceeded";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    fn recursion_depth_exceeded() -> Self {
        Self {
            message: RECURSION_ERROR_MESSAGE.to_string(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for ParseError {}

pub fn parse_expression(
    expression: &str,
) -> Result<(Vec<CommandInvocation>, Vec<String>), ParseError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_bash::LANGUAGE.into())
        .expect("Error loading Bash parser");
    let tree = parser
        .parse(expression.as_bytes(), None)
        .expect("tree-sitter returned no tree");

    let mut warnings = Vec::new();
    if tree.root_node().has_error() {
        warnings.push(format!(
            "tree-sitter reported a syntax error in expression: {}",
            py_repr(expression)
        ));
    }
    // Python stops runaway recursion with sys.getrecursionlimit(); this lower Rust limit leaves
    // wide margin for the default 8 MiB main-thread stack. Exact depth parity is not required.
    let invocations = walk_node(
        tree.root_node(),
        expression.as_bytes(),
        "toplevel",
        None,
        0,
        1,
        0,
    )?;
    Ok((invocations, warnings))
}

fn check_depth(depth: usize) -> Result<(), ParseError> {
    if depth > MAX_WALK_DEPTH {
        Err(ParseError::recursion_depth_exceeded())
    } else {
        Ok(())
    }
}

fn walk_node(
    node: Node<'_>,
    source: &[u8],
    context: &str,
    operator_before: Option<String>,
    pipeline_position: usize,
    pipeline_length: usize,
    depth: usize,
) -> Result<Vec<CommandInvocation>, ParseError> {
    check_depth(depth)?;
    let mut results = Vec::new();

    // Dispatch shape tracks parser.py::_walk_node; keep Rust/tree-sitter divergences explicit.
    match node.kind() {
        "program" => results.extend(walk_children_as_list(node, source, context, depth + 1)?),
        "list" => results.extend(walk_list_node(
            node,
            source,
            context,
            operator_before,
            depth + 1,
        )?),
        "pipeline" => results.extend(walk_pipeline_node(
            node,
            source,
            context,
            operator_before,
            depth + 1,
        )?),
        "command" => {
            if let Some(inv) = extract_command(
                node,
                source,
                context,
                operator_before,
                pipeline_position,
                pipeline_length,
                depth + 1,
            )? {
                results.push(inv);
            }
            results.extend(extract_nested_from_command_args(node, source, depth + 1)?);
        }
        "redirected_statement" => results.extend(walk_redirected_statement(
            node,
            source,
            context,
            operator_before,
            pipeline_position,
            pipeline_length,
            depth + 1,
        )?),
        "subshell" => results.extend(walk_subshell(
            node,
            source,
            "subshell",
            operator_before,
            depth + 1,
        )?),
        "command_substitution" => {
            results.extend(walk_compound_inner(
                node,
                source,
                "command_substitution",
                depth + 1,
            )?);
        }
        "process_substitution" => {
            results.extend(walk_compound_inner(
                node,
                source,
                "process_substitution",
                depth + 1,
            )?);
        }
        "test_command" => {
            if let Some(inv) = extract_test_command(
                node,
                source,
                context,
                operator_before,
                pipeline_position,
                pipeline_length,
                depth + 1,
            )? {
                results.push(inv);
            }
        }
        "negated_command" => {
            for child in children(node) {
                if child.kind() != "!" {
                    results.extend(walk_node(
                        child,
                        source,
                        context,
                        operator_before.clone(),
                        pipeline_position,
                        pipeline_length,
                        depth + 1,
                    )?);
                }
            }
        }
        _ => {
            for child in children(node) {
                results.extend(walk_node(
                    child,
                    source,
                    context,
                    operator_before.clone(),
                    pipeline_position,
                    pipeline_length,
                    depth + 1,
                )?);
            }
        }
    }

    Ok(results)
}

fn walk_children_as_list(
    node: Node<'_>,
    source: &[u8],
    context: &str,
    depth: usize,
) -> Result<Vec<CommandInvocation>, ParseError> {
    check_depth(depth)?;
    let mut results = Vec::new();
    let mut operator_before: Option<String> = None;

    for child in children(node) {
        match child.kind() {
            "\n" | ";" | "&" | ";;" => {
                if child.kind() == ";" {
                    operator_before = Some(";".to_string());
                } else if child.kind() == "&" {
                    mark_last_background(&mut results);
                    operator_before = None;
                }
                continue;
            }
            _ => {}
        }

        let child_results = walk_node(
            child,
            source,
            context,
            operator_before.clone(),
            0,
            1,
            depth + 1,
        )?;
        results.extend(child_results);
        operator_before = None;
    }

    Ok(results)
}

fn walk_list_node(
    node: Node<'_>,
    source: &[u8],
    context: &str,
    operator_before: Option<String>,
    depth: usize,
) -> Result<Vec<CommandInvocation>, ParseError> {
    check_depth(depth)?;
    let mut results = Vec::new();
    let mut current_operator = operator_before;

    // Matches parser.py::_walk_list_node operator carry-forward and background handling.
    for child in children(node) {
        match child.kind() {
            "&&" | "||" | ";" => {
                current_operator = Some(child.kind().to_string());
                continue;
            }
            "&" => {
                mark_last_background(&mut results);
                current_operator = None;
                continue;
            }
            "\n" => continue,
            _ => {}
        }

        let child_results = walk_node(
            child,
            source,
            context,
            current_operator.clone(),
            0,
            1,
            depth + 1,
        )?;
        results.extend(child_results);
        current_operator = None;
    }

    Ok(results)
}

fn walk_pipeline_node(
    node: Node<'_>,
    source: &[u8],
    context: &str,
    operator_before: Option<String>,
    depth: usize,
) -> Result<Vec<CommandInvocation>, ParseError> {
    check_depth(depth)?;
    let command_nodes: Vec<_> = children(node)
        .into_iter()
        .filter(|child| child.kind() != "|" && child.kind() != "|&")
        .collect();
    let pipe_len = command_nodes.len();

    let mut results = Vec::new();
    // Mirrors parser.py::_walk_pipeline_node: pipeline position is structural, not argv text.
    for (i, cmd_node) in command_nodes.into_iter().enumerate() {
        let op = if i == 0 {
            operator_before.clone()
        } else {
            None
        };
        results.extend(walk_node(
            cmd_node,
            source,
            context,
            op,
            i,
            pipe_len,
            depth + 1,
        )?);
    }
    Ok(results)
}

fn walk_redirected_statement(
    node: Node<'_>,
    source: &[u8],
    context: &str,
    operator_before: Option<String>,
    pipeline_position: usize,
    pipeline_length: usize,
    depth: usize,
) -> Result<Vec<CommandInvocation>, ParseError> {
    check_depth(depth)?;
    let mut results = Vec::new();
    let redirects = extract_redirects_from_node(node, source, depth + 1)?;

    let body_node = children(node).into_iter().find(|child| {
        !matches!(
            child.kind(),
            "file_redirect" | "heredoc_redirect" | "herestring_redirect"
        )
    });

    let Some(body_node) = body_node else {
        return Ok(results);
    };

    let mut inner = walk_node(
        body_node,
        source,
        context,
        operator_before,
        pipeline_position,
        pipeline_length,
        depth + 1,
    )?;

    // Python attaches outer redirected_statement redirects once; avoid duplicating them on children.
    let mut redirects_attached = false;
    for inv in &mut inner {
        if !redirects_attached && inv.position_in_pipeline == pipeline_position {
            if inv.redirects.is_empty() {
                inv.redirects = redirects.clone();
            } else {
                inv.redirects.extend(redirects.clone());
            }
            redirects_attached = true;
        }
    }

    results.extend(inner);
    Ok(results)
}

fn walk_subshell(
    node: Node<'_>,
    source: &[u8],
    context: &str,
    mut operator_before: Option<String>,
    depth: usize,
) -> Result<Vec<CommandInvocation>, ParseError> {
    check_depth(depth)?;
    let mut results = Vec::new();
    for child in children(node) {
        if matches!(child.kind(), "(" | ")") {
            continue;
        }
        results.extend(walk_node(
            child,
            source,
            context,
            operator_before.clone(),
            0,
            1,
            depth + 1,
        )?);
        operator_before = None;
    }
    Ok(results)
}

fn walk_compound_inner(
    node: Node<'_>,
    source: &[u8],
    context: &str,
    depth: usize,
) -> Result<Vec<CommandInvocation>, ParseError> {
    check_depth(depth)?;
    let mut results = Vec::new();
    for child in children(node) {
        if matches!(child.kind(), "$(" | ")" | "<(" | ">(" | "`") {
            continue;
        }
        results.extend(walk_node(child, source, context, None, 0, 1, depth + 1)?);
    }
    Ok(results)
}

fn extract_command(
    node: Node<'_>,
    source: &[u8],
    context: &str,
    operator_before: Option<String>,
    pipeline_position: usize,
    pipeline_length: usize,
    depth: usize,
) -> Result<Option<CommandInvocation>, ParseError> {
    check_depth(depth)?;
    let mut argv = Vec::new();
    let mut has_command_name = false;

    // Mirrors parser.py::_extract_command: leading assignments and redirects are metadata, not argv.
    for child in children(node) {
        match child.kind() {
            "variable_assignment" => {
                if !has_command_name {
                    continue;
                }
                argv.push(node_text(child, source, depth + 1)?);
            }
            "command_name" => {
                has_command_name = true;
                argv.push(node_text(child, source, depth + 1)?);
            }
            "file_redirect" | "heredoc_redirect" | "herestring_redirect" | "comment" => continue,
            _ => argv.push(node_text(child, source, depth + 1)?),
        }
    }

    if argv.is_empty() {
        return Ok(None);
    }

    Ok(Some(CommandInvocation {
        argv,
        redirects: extract_redirects_from_node(node, source, depth + 1)?,
        position_in_pipeline: pipeline_position,
        pipeline_length,
        context: context.to_string(),
        operator_before,
        is_background: false,
    }))
}

fn extract_test_command(
    node: Node<'_>,
    source: &[u8],
    context: &str,
    operator_before: Option<String>,
    pipeline_position: usize,
    pipeline_length: usize,
    depth: usize,
) -> Result<Option<CommandInvocation>, ParseError> {
    check_depth(depth)?;
    let mut argv = Vec::new();
    for child in children(node) {
        if matches!(child.kind(), "[" | "[[") {
            argv.push(raw_text(child, source));
        } else if matches!(child.kind(), "]" | "]]") {
            continue;
        } else if child.is_named() {
            argv.push(node_text(child, source, depth + 1)?);
        }
    }

    if argv.is_empty() {
        return Ok(None);
    }

    Ok(Some(CommandInvocation {
        argv,
        redirects: Vec::new(),
        position_in_pipeline: pipeline_position,
        pipeline_length,
        context: context.to_string(),
        operator_before,
        is_background: false,
    }))
}

fn extract_nested_from_command_args(
    node: Node<'_>,
    source: &[u8],
    depth: usize,
) -> Result<Vec<CommandInvocation>, ParseError> {
    check_depth(depth)?;
    let mut results = Vec::new();
    for child in children(node) {
        if child.kind() == "command_name" {
            for sub in children(child) {
                results.extend(find_nested_substitutions(sub, source, depth + 1)?);
            }
        } else if matches!(
            child.kind(),
            "file_redirect" | "heredoc_redirect" | "herestring_redirect" | "variable_assignment"
        ) {
            continue;
        } else {
            results.extend(find_nested_substitutions(child, source, depth + 1)?);
        }
    }
    Ok(results)
}

fn find_nested_substitutions(
    node: Node<'_>,
    source: &[u8],
    depth: usize,
) -> Result<Vec<CommandInvocation>, ParseError> {
    check_depth(depth)?;
    let mut results = Vec::new();
    if node.kind() == "command_substitution" {
        results.extend(walk_compound_inner(
            node,
            source,
            "command_substitution",
            depth + 1,
        )?);
    } else if node.kind() == "process_substitution" {
        results.extend(walk_compound_inner(
            node,
            source,
            "process_substitution",
            depth + 1,
        )?);
    } else {
        for child in children(node) {
            results.extend(find_nested_substitutions(child, source, depth + 1)?);
        }
    }
    Ok(results)
}

fn extract_redirects_from_node(
    node: Node<'_>,
    source: &[u8],
    depth: usize,
) -> Result<Vec<Redirect>, ParseError> {
    check_depth(depth)?;
    let mut redirects = Vec::new();
    // Redirect collection mirrors parser.py redirect extraction; heredocs record markers, not bodies.
    for child in children(node) {
        match child.kind() {
            "file_redirect" => redirects.push(parse_file_redirect(child, source, depth + 1)?),
            "heredoc_redirect" => redirects.push(parse_heredoc_redirect(child, source)),
            "herestring_redirect" => {
                redirects.push(parse_herestring_redirect(child, source, depth + 1)?)
            }
            _ => {}
        }
    }
    Ok(redirects)
}

fn parse_file_redirect(
    node: Node<'_>,
    source: &[u8],
    depth: usize,
) -> Result<Redirect, ParseError> {
    check_depth(depth)?;
    let mut operator_parts = Vec::new();
    let mut target = String::new();

    for child in children(node) {
        if child.kind() == "file_descriptor" || is_redirect_operator(child.kind()) {
            operator_parts.push(raw_text(child, source));
        } else if child.is_named() {
            target = node_text(child, source, depth + 1)?;
        } else {
            let text = raw_text(child, source);
            if is_redirect_operator(&text) {
                operator_parts.push(text);
            } else if target.is_empty() {
                target = text;
            }
        }
    }

    let mut operator = operator_parts.join("");
    if operator.is_empty() {
        let full_text = raw_text(node, source);
        if full_text.contains('>') || full_text.contains('<') {
            operator = full_text
                .split_whitespace()
                .next()
                .unwrap_or(&full_text)
                .to_string();
        }
    }

    // Keep fd-to-fd redirects and /dev/null quiet like the Python classifier input model.
    let affects_classification = redirect_affects_classification(&operator, &target);
    Ok(Redirect {
        operator,
        target,
        affects_classification,
    })
}

fn parse_heredoc_redirect(node: Node<'_>, source: &[u8]) -> Redirect {
    let mut operator = "<<".to_string();
    let mut target = String::new();

    // Python parser exposes heredoc redirection metadata without treating the body as argv.
    for child in children(node) {
        if matches!(child.kind(), "<<" | "<<-") {
            operator = raw_text(child, source);
        } else if child.kind() == "heredoc_start" {
            target = raw_text(child, source);
        }
    }

    Redirect {
        operator,
        target,
        affects_classification: false,
    }
}

fn parse_herestring_redirect(
    node: Node<'_>,
    source: &[u8],
    depth: usize,
) -> Result<Redirect, ParseError> {
    check_depth(depth)?;
    let mut target = String::new();
    for child in children(node) {
        if child.kind() == "<<<" {
            continue;
        } else if child.is_named() {
            target = node_text(child, source, depth + 1)?;
        } else {
            let text = raw_text(child, source);
            if text != "<<<" && target.is_empty() {
                target = text;
            }
        }
    }

    Ok(Redirect {
        operator: "<<<".to_string(),
        target,
        affects_classification: false,
    })
}

fn redirect_affects_classification(operator: &str, target: &str) -> bool {
    if target == "/dev/null" {
        return false;
    }
    if operator.contains(">&") && target.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }
    if matches!(operator, ">" | ">>" | "&>" | "&>>")
        || (operator.ends_with('>') && !operator.contains('<'))
    {
        return true;
    }
    operator.len() >= 2
        && operator
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_digit())
        && operator.contains('>')
}

fn node_text(node: Node<'_>, source: &[u8], depth: usize) -> Result<String, ParseError> {
    check_depth(depth)?;
    // String/argv reconstruction follows Python's parser output: strip shell quote wrappers and
    // concatenate adjacent parts, but do not evaluate shell expansions.
    match node.kind() {
        "string" => {
            let text = raw_text(node, source);
            if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
                Ok(text[1..text.len() - 1].to_string())
            } else {
                Ok(text)
            }
        }
        "raw_string" => {
            let text = raw_text(node, source);
            if text.len() >= 2 && text.starts_with('\'') && text.ends_with('\'') {
                Ok(text[1..text.len() - 1].to_string())
            } else if text.len() >= 3 && text.starts_with("$'") && text.ends_with('\'') {
                Ok(text[2..text.len() - 1].to_string())
            } else {
                Ok(text)
            }
        }
        "ansi_c_string" => {
            let text = raw_text(node, source);
            if text.len() >= 3 && text.starts_with("$'") && text.ends_with('\'') {
                Ok(text[2..text.len() - 1].to_string())
            } else {
                Ok(text)
            }
        }
        "concatenation" => Ok(children(node)
            .into_iter()
            .map(|child| node_text(child, source, depth + 1))
            .collect::<Result<Vec<_>, _>>()?
            .join("")),
        _ => Ok(raw_text(node, source)),
    }
}

fn raw_text(node: Node<'_>, source: &[u8]) -> String {
    node.utf8_text(source).unwrap_or("").to_string()
}

fn mark_last_background(results: &mut [CommandInvocation]) {
    if let Some(inv) = results.last_mut() {
        inv.is_background = true;
    }
}

fn is_redirect_operator(text: &str) -> bool {
    matches!(
        text,
        ">" | ">>" | "<" | ">&" | "&>" | "&>>" | "<&" | "<<" | "<<<"
    )
}

fn children(node: Node<'_>) -> Vec<Node<'_>> {
    (0..node.child_count())
        .filter_map(|index| node.child(index))
        .collect()
}

fn py_repr(value: &str) -> String {
    // Match Python repr() warning text closely enough for parity snapshots and CLI errors.
    let has_single = value.contains('\'');
    let has_double = value.contains('"');
    let quote = if has_single && !has_double { '"' } else { '\'' };
    let mut out = String::new();
    out.push(quote);
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' if quote == '\'' => out.push_str("\\'"),
            '"' if quote == '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\x08"),
            '\x0c' => out.push_str("\\x0c"),
            ch if ch.is_control() => {
                let code = ch as u32;
                if code <= 0xff {
                    out.push_str(&format!("\\x{code:02x}"));
                } else if code <= 0xffff {
                    out.push_str(&format!("\\u{code:04x}"));
                } else {
                    out.push_str(&format!("\\U{code:08x}"));
                }
            }
            ch => out.push(ch),
        }
    }
    out.push(quote);
    out
}

#[cfg(test)]
mod test_parser {
    use super::{parse_expression, py_repr};

    mod test_simple_commands {
        use super::parse_expression;

        #[test]
        fn test_simple_command() {
            let (result, _) = parse_expression("ls").unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].argv, ["ls"]);
            assert_eq!(result[0].context, "toplevel");
            assert_eq!(result[0].operator_before, None);
            assert_eq!(result[0].position_in_pipeline, 0);
            assert_eq!(result[0].pipeline_length, 1);
            assert!(!result[0].is_background);
        }

        #[test]
        fn test_command_with_args() {
            let (result, _) = parse_expression("ls -la /tmp").unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].argv, ["ls", "-la", "/tmp"]);
        }

        #[test]
        fn test_command_with_long_options() {
            let (result, _) = parse_expression("grep --recursive --ignore-case pattern .").unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(
                result[0].argv,
                ["grep", "--recursive", "--ignore-case", "pattern", "."]
            );
        }
    }

    mod test_pipelines {
        use super::parse_expression;

        #[test]
        fn test_simple_pipeline() {
            let (result, _) = parse_expression("cat file | grep pattern | wc -l").unwrap();
            let pipeline_cmds: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(pipeline_cmds.len(), 3);
            assert_eq!(pipeline_cmds[0].argv, ["cat", "file"]);
            assert_eq!(pipeline_cmds[0].position_in_pipeline, 0);
            assert_eq!(pipeline_cmds[0].pipeline_length, 3);
            assert_eq!(pipeline_cmds[1].argv, ["grep", "pattern"]);
            assert_eq!(pipeline_cmds[1].position_in_pipeline, 1);
            assert_eq!(pipeline_cmds[1].pipeline_length, 3);
            assert_eq!(pipeline_cmds[2].argv, ["wc", "-l"]);
            assert_eq!(pipeline_cmds[2].position_in_pipeline, 2);
            assert_eq!(pipeline_cmds[2].pipeline_length, 3);
        }

        #[test]
        fn test_two_command_pipeline() {
            let (result, _) = parse_expression("ps aux | grep python").unwrap();
            let pipeline_cmds: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(pipeline_cmds.len(), 2);
            assert_eq!(pipeline_cmds[0].pipeline_length, 2);
            assert_eq!(pipeline_cmds[1].pipeline_length, 2);
        }
    }

    mod test_logical_operators {
        use super::parse_expression;

        #[test]
        fn test_logical_and() {
            let (result, _) = parse_expression("make && make install").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 2);
            assert_eq!(toplevel[0].argv, ["make"]);
            assert_eq!(toplevel[0].operator_before, None);
            assert_eq!(toplevel[1].argv, ["make", "install"]);
            assert_eq!(toplevel[1].operator_before.as_deref(), Some("&&"));
        }

        #[test]
        fn test_logical_or() {
            let (result, _) = parse_expression("cmd1 || cmd2").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 2);
            assert_eq!(toplevel[0].operator_before, None);
            assert_eq!(toplevel[1].operator_before.as_deref(), Some("||"));
        }

        #[test]
        fn test_chained_operators() {
            let (result, _) = parse_expression("a && b || c").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 3);
            assert_eq!(toplevel[0].operator_before, None);
            assert_eq!(toplevel[1].operator_before.as_deref(), Some("&&"));
            assert_eq!(toplevel[2].operator_before.as_deref(), Some("||"));
        }
    }

    mod test_semicolons {
        use super::parse_expression;

        #[test]
        fn test_semicolons() {
            let (result, _) = parse_expression("echo a; echo b; echo c").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 3);
            assert_eq!(toplevel[0].argv, ["echo", "a"]);
            assert_eq!(toplevel[0].operator_before, None);
            assert_eq!(toplevel[1].argv, ["echo", "b"]);
            assert_eq!(toplevel[1].operator_before.as_deref(), Some(";"));
            assert_eq!(toplevel[2].argv, ["echo", "c"]);
            assert_eq!(toplevel[2].operator_before.as_deref(), Some(";"));
        }
    }

    mod test_command_substitution {
        use super::parse_expression;

        #[test]
        fn test_command_substitution() {
            let (result, _) = parse_expression("echo $(date)").unwrap();
            let outer: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            let inner: Vec<_> = result
                .iter()
                .filter(|r| r.context == "command_substitution")
                .collect();
            assert_eq!(outer.len(), 1);
            assert_eq!(outer[0].argv, ["echo", "$(date)"]);
            assert_eq!(inner.len(), 1);
            assert_eq!(inner[0].argv, ["date"]);
        }

        #[test]
        fn test_nested_command_substitution_in_pipeline() {
            let (result, _) = parse_expression("echo $(cat file | wc -l)").unwrap();
            let inner: Vec<_> = result
                .iter()
                .filter(|r| r.context == "command_substitution")
                .collect();
            assert_eq!(inner.len(), 2);
            assert_eq!(inner[0].argv, ["cat", "file"]);
            assert_eq!(inner[1].argv, ["wc", "-l"]);
        }
    }

    mod test_recursion_depth {
        use super::parse_expression;

        fn nested_command_substitution(depth: usize) -> String {
            let mut expression = String::from("echo ");
            for _ in 0..depth {
                expression.push_str("$(echo ");
            }
            expression.push('x');
            for _ in 0..depth {
                expression.push(')');
            }
            expression
        }

        #[test]
        fn test_moderate_nesting_still_parses() {
            let expression = nested_command_substitution(50);
            let (result, _) = parse_expression(&expression).unwrap();
            assert_eq!(result[0].argv[0], "echo");
            assert_eq!(result.iter().filter(|r| r.argv[0] == "echo").count(), 51);
        }

        #[test]
        fn test_deep_nesting_returns_parse_error() {
            for depth in [1000, 5000, 50000] {
                let expression = nested_command_substitution(depth);
                let error = parse_expression(&expression).unwrap_err();
                assert_eq!(error.message(), "maximum recursion depth exceeded");
            }
        }
    }

    mod test_process_substitution {
        use super::parse_expression;

        #[test]
        fn test_process_substitution() {
            let (result, _) = parse_expression("diff <(ls dir1) <(ls dir2)").unwrap();
            let outer: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            let inner: Vec<_> = result
                .iter()
                .filter(|r| r.context == "process_substitution")
                .collect();
            assert_eq!(outer.len(), 1);
            assert_eq!(outer[0].argv, ["diff", "<(ls dir1)", "<(ls dir2)"]);
            assert_eq!(inner.len(), 2);
            assert_eq!(inner[0].argv, ["ls", "dir1"]);
            assert_eq!(inner[1].argv, ["ls", "dir2"]);
        }
    }

    mod test_subshell {
        use super::parse_expression;

        #[test]
        fn test_subshell() {
            let (result, _) = parse_expression("(cd /tmp && ls)").unwrap();
            let subshell_cmds: Vec<_> = result.iter().filter(|r| r.context == "subshell").collect();
            assert_eq!(subshell_cmds.len(), 2);
            assert_eq!(subshell_cmds[0].argv, ["cd", "/tmp"]);
            assert_eq!(subshell_cmds[0].operator_before, None);
            assert_eq!(subshell_cmds[1].argv, ["ls"]);
            assert_eq!(subshell_cmds[1].operator_before.as_deref(), Some("&&"));
        }
    }

    mod test_redirects {
        use super::parse_expression;

        #[test]
        fn test_output_redirect() {
            let (result, _) = parse_expression("echo hello > output.txt").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].argv, ["echo", "hello"]);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert_eq!(toplevel[0].redirects[0].operator, ">");
            assert_eq!(toplevel[0].redirects[0].target, "output.txt");
        }

        #[test]
        fn test_stderr_redirect() {
            let (result, _) = parse_expression("cmd 2>/dev/null").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert_eq!(toplevel[0].redirects[0].operator, "2>");
            assert_eq!(toplevel[0].redirects[0].target, "/dev/null");
        }

        #[test]
        fn test_append_redirect() {
            let (result, _) = parse_expression("cmd >> file.log").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert_eq!(toplevel[0].redirects[0].operator, ">>");
            assert_eq!(toplevel[0].redirects[0].target, "file.log");
        }

        #[test]
        fn test_input_redirect() {
            let (result, _) = parse_expression("cmd < input.txt").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert_eq!(toplevel[0].redirects[0].operator, "<");
            assert_eq!(toplevel[0].redirects[0].target, "input.txt");
        }

        #[test]
        fn test_redirect_affects_classification() {
            let (result, _) = parse_expression("echo hello > output.txt").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert!(toplevel[0].redirects[0].affects_classification);
        }

        #[test]
        fn test_devnull_redirect_no_affect() {
            let (result, _) = parse_expression("cmd > /dev/null").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert!(!toplevel[0].redirects[0].affects_classification);
        }
    }

    mod test_variable_assignment {
        use super::parse_expression;

        #[test]
        fn test_variable_assignment_prefix() {
            let (result, _) = parse_expression("FOO=bar baz").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].argv, ["baz"]);
        }

        #[test]
        fn test_multiple_assignments() {
            let (result, _) = parse_expression("FOO=bar BAZ=qux cmd arg1").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].argv, ["cmd", "arg1"]);
        }

        #[test]
        fn test_bare_assignment_no_command() {
            let (result, _) = parse_expression("FOO=bar").unwrap();
            assert_eq!(result.len(), 0);
        }
    }

    mod test_background {
        use super::parse_expression;

        #[test]
        fn test_background() {
            let (result, _) = parse_expression("sleep 10 &").unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].argv, ["sleep", "10"]);
            assert!(result[0].is_background);
        }

        #[test]
        fn test_background_in_list() {
            let (result, _) = parse_expression("cmd1 & cmd2").unwrap();
            assert_eq!(result.len(), 2);
            assert!(result[0].is_background);
            assert!(!result[1].is_background);
        }
    }

    mod test_complex {
        use super::parse_expression;

        #[test]
        fn test_complex_expression() {
            let (result, _) =
                parse_expression("git status && (cd src && make) | tee log.txt").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            let subshell: Vec<_> = result.iter().filter(|r| r.context == "subshell").collect();
            let git_status: Vec<_> = toplevel.iter().filter(|r| r.argv[0] == "git").collect();
            assert_eq!(git_status.len(), 1);
            assert_eq!(git_status[0].argv, ["git", "status"]);
            assert_eq!(subshell.len(), 2);
            assert_eq!(subshell[0].argv, ["cd", "src"]);
            assert_eq!(subshell[1].argv, ["make"]);
            let tee: Vec<_> = toplevel.iter().filter(|r| r.argv[0] == "tee").collect();
            assert_eq!(tee.len(), 1);
            assert_eq!(tee[0].argv, ["tee", "log.txt"]);
        }
    }

    mod test_heredoc {
        use super::parse_expression;

        #[test]
        fn test_heredoc() {
            let expression = "cat <<EOF\nhello\nworld\nEOF";
            let (result, _) = parse_expression(expression).unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].argv, ["cat"]);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert_eq!(toplevel[0].redirects[0].operator, "<<");
            assert_eq!(toplevel[0].redirects[0].target, "EOF");
        }

        #[test]
        fn test_heredoc_content_not_parsed_as_commands() {
            let expression = "cat <<EOF\nls -la\nrm -rf /\nEOF";
            let (result, _) = parse_expression(expression).unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].argv, ["cat"]);
        }
    }

    mod test_edge_cases {
        use super::parse_expression;

        #[test]
        fn test_empty_expression() {
            let (result, _) = parse_expression("").unwrap();
            assert_eq!(result, []);
        }

        #[test]
        fn test_comment_only() {
            let (result, _) = parse_expression("# this is a comment").unwrap();
            assert_eq!(result, []);
        }

        #[test]
        fn test_quoted_string_arg() {
            let (result, _) = parse_expression("echo \"hello world\"").unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].argv, ["echo", "hello world"]);
        }

        #[test]
        fn test_single_quoted_string_arg() {
            let (result, _) = parse_expression("echo 'hello world'").unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].argv, ["echo", "hello world"]);
        }
    }

    mod test_backtick_command_substitution {
        use super::parse_expression;

        #[test]
        fn test_backtick_command_substitution() {
            let (result, _) = parse_expression("echo `date`").unwrap();
            let outer: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            let inner: Vec<_> = result
                .iter()
                .filter(|r| r.context == "command_substitution")
                .collect();
            assert_eq!(outer.len(), 1);
            assert_eq!(inner.len(), 1);
            assert_eq!(inner[0].argv, ["date"]);
            assert_eq!(inner[0].context, "command_substitution");
        }
    }

    mod test_negated_command {
        use super::parse_expression;

        #[test]
        fn test_negated_command() {
            let (result, _) = parse_expression("! grep pattern file").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].argv, ["grep", "pattern", "file"]);
        }
    }

    mod test_ansi_c_quoted_string {
        use super::parse_expression;

        #[test]
        fn test_dollar_single_quoted_string() {
            let (result, _) = parse_expression("echo $'hello'").unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].argv, ["echo", "hello"]);
        }
    }

    mod test_multiple_redirects {
        use super::parse_expression;

        #[test]
        fn test_multiple_redirects() {
            let (result, _) = parse_expression("cmd > out.txt 2> err.txt").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].redirects.len(), 2);
        }
    }

    mod test_concatenation {
        use super::parse_expression;

        #[test]
        fn test_concatenation() {
            let (result, _) = parse_expression("echo foo\"bar\"baz").unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].argv, ["echo", "foobarbaz"]);
        }
    }

    mod test_test_command {
        use super::parse_expression;

        #[test]
        fn test_single_bracket_test() {
            let (result, _) = parse_expression("[ -f file ]").unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].argv[0], "[");
        }

        #[test]
        fn test_double_bracket_test() {
            let (result, _) = parse_expression("[[ -f file ]]").unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].argv[0], "[[");
        }

        #[test]
        fn test_test_command_in_pipeline() {
            let (result, _) = parse_expression("[ -f file ] && echo exists").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 2);
            assert_eq!(toplevel[0].argv[0], "[");
            assert_eq!(toplevel[1].argv, ["echo", "exists"]);
        }
    }

    mod test_herestring {
        use super::parse_expression;

        #[test]
        fn test_herestring_redirect() {
            let (result, _) = parse_expression("cat <<< hello").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].argv, ["cat"]);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert_eq!(toplevel[0].redirects[0].operator, "<<<");
            assert_eq!(toplevel[0].redirects[0].target, "hello");
            assert!(!toplevel[0].redirects[0].affects_classification);
        }
    }

    mod test_redirected_compound_statement {
        use super::parse_expression;

        #[test]
        fn test_for_loop_redirect_not_duplicated() {
            let (result, _) = parse_expression("for i in 1 2; do echo $i; done > out.txt").unwrap();
            let cmds_with_redirects: Vec<_> =
                result.iter().filter(|r| !r.redirects.is_empty()).collect();
            assert_eq!(cmds_with_redirects.len(), 1);
            assert_eq!(cmds_with_redirects[0].redirects[0].operator, ">");
            assert_eq!(cmds_with_redirects[0].redirects[0].target, "out.txt");
        }
    }

    mod test_fd_to_fd_redirects {
        use super::parse_expression;

        #[test]
        fn test_stderr_to_stdout_no_affect() {
            let (result, _) = parse_expression("cmd 2>&1").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert!(!toplevel[0].redirects[0].affects_classification);
        }

        #[test]
        fn test_stdout_to_stderr_no_affect() {
            let (result, _) = parse_expression("cmd 1>&2").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert!(!toplevel[0].redirects[0].affects_classification);
        }

        #[test]
        fn test_stderr_to_file_affects() {
            let (result, _) = parse_expression("cmd 2>error.log").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert!(toplevel[0].redirects[0].affects_classification);
        }
    }

    mod test_ampersand_redirects {
        use super::parse_expression;

        #[test]
        fn test_ampersand_redirect_to_file() {
            let (result, _) = parse_expression("cmd &> output.txt").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert_eq!(toplevel[0].redirects[0].operator, "&>");
            assert!(toplevel[0].redirects[0].affects_classification);
        }

        #[test]
        fn test_ampersand_append_redirect_to_file() {
            let (result, _) = parse_expression("cmd &>> output.txt").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert_eq!(toplevel[0].redirects[0].operator, "&>>");
            assert!(toplevel[0].redirects[0].affects_classification);
        }

        #[test]
        fn test_ampersand_redirect_to_devnull() {
            let (result, _) = parse_expression("cmd &> /dev/null").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert_eq!(toplevel[0].redirects[0].operator, "&>");
            assert!(!toplevel[0].redirects[0].affects_classification);
        }
    }

    mod test_process_substitution_as_redirect_target {
        use super::parse_expression;

        #[test]
        fn test_output_process_substitution() {
            let (result, _) = parse_expression("echo hello > >(tee log.txt)").unwrap();
            let toplevel: Vec<_> = result.iter().filter(|r| r.context == "toplevel").collect();
            assert_eq!(toplevel.len(), 1);
            assert_eq!(toplevel[0].redirects.len(), 1);
            assert!(toplevel[0].redirects[0].target.contains("tee"));
        }
    }

    mod test_parse_warnings {
        use super::{parse_expression, py_repr};

        #[test]
        fn test_syntax_error_produces_warning() {
            let (_, warnings) = parse_expression("if then fi else").unwrap();
            assert!(!warnings.is_empty());
            assert!(warnings[0].contains("syntax error"));
            assert_eq!(
                warnings[0],
                format!(
                    "tree-sitter reported a syntax error in expression: {}",
                    py_repr("if then fi else")
                )
            );
        }
    }

    mod test_py_repr {
        use super::py_repr;

        #[test]
        fn test_single_quote_preference() {
            assert_eq!(py_repr("plain"), "'plain'");
            assert_eq!(py_repr("has ' single"), "\"has ' single\"");
            assert_eq!(py_repr("has ' and \" both"), "'has \\' and \" both'");
        }

        #[test]
        fn test_control_escapes() {
            assert_eq!(py_repr("a\nb\tc\\d"), "'a\\nb\\tc\\\\d'");
            assert_eq!(py_repr("\x01"), "'\\x01'");
        }
    }
}
