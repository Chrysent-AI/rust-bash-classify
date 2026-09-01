#!/usr/bin/env python3
"""Generate PORT_MANIFEST.md from the Python pytest suite and Rust test list.

The manifest maps each Python test function to the Rust test that covers the
same behavior. Extra Rust-only regression/helper tests are listed separately.
"""

from __future__ import annotations

import ast
import re
import subprocess
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PYTHON_REPO = Path("../bash-classify")
PYTHON_TESTS = PYTHON_REPO / "tests"
OUTPUT = ROOT / "PORT_MANIFEST.md"
EXPECTED_SUITE_COUNTS = {
    "test_models.py": 13,
    "test_database.py": 36,
    "test_smoke.py": 1,
    "test_parser.py": 49,
    "test_matcher.py": 134,
    "test_classifier.py": 114,
    "test_real_world.py": 90,
    "test_cli_serialization.py": 18,
    "test_cli.py": 7,
    "test_schema.py": 1,
}
SUITE_ORDER = [
    "test_models.py",
    "test_database.py",
    "test_smoke.py",
    "test_parser.py",
    "test_matcher.py",
    "test_classifier.py",
    "test_real_world.py",
    "test_cli_serialization.py",
    "test_cli.py",
    "test_schema.py",
]
SPECIAL_NAME_MAP = {
    ("test_cli_serialization.py", "TestWriteReadPaths", "test_paths_omitted_when_empty"): "test_empty_optional_lists_omitted",
    ("test_smoke.py", None, "test_import"): "test_crate_import_smoke",
    ("test_classifier.py", "TestDevNullRedirectNoElevation", "test_devnull_redirect_no_elevation"): "test_devnull_redirect_no_elevation_a",
    ("test_classifier.py", "TestInputRedirectNoElevation", "test_input_redirect_no_elevation"): "test_input_redirect_no_elevation_a",
    ("test_classifier.py", "TestSystemDirectoryClassification", "test_dev_null_safe"): "test_dev_null_safe_class",
}
EXPECTED_EXTRA_TESTS = {
    "matrix_empty_input",
    "matrix_help_and_version",
    "matrix_internal_error_invalid_user_yaml",
    "matrix_success_piped_compound_subshell_redirect_outputs_match_python_fixtures",
    "matrix_timeout_without_stdin_data",
    "golden_corpus_replays_byte_identically",
    "user_yaml_adds_new_command_from_custom_config_root",
    "user_yaml_overrides_builtin_by_filename_stem",
}


@dataclass(frozen=True)
class PythonTest:
    suite: str
    class_name: str | None
    function: str
    line: int

    @property
    def id(self) -> str:
        if self.class_name:
            return f"tests/{self.suite}::{self.class_name}::{self.function}"
        return f"tests/{self.suite}::{self.function}"


def python_tests() -> list[PythonTest]:
    tests: list[PythonTest] = []
    for suite in SUITE_ORDER:
        path = PYTHON_TESTS / suite
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        for node in tree.body:
            if isinstance(node, ast.FunctionDef) and node.name.startswith("test_"):
                tests.append(PythonTest(suite, None, node.name, node.lineno))
            elif isinstance(node, ast.ClassDef):
                for child in node.body:
                    if isinstance(child, ast.FunctionDef) and child.name.startswith("test_"):
                        tests.append(PythonTest(suite, node.name, child.name, child.lineno))
    return tests


def snake_case(name: str) -> str:
    text = re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()
    return text


def rust_tests() -> dict[str, str]:
    proc = subprocess.run(
        ["cargo", "test", "--", "--list"],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=True,
    )
    tests: dict[str, str] = {}
    current_file = ""
    for line in proc.stdout.splitlines():
        running = re.search(r"Running (?:unittests (src/lib.rs|src/bin/[^ ]+)|tests/([^ ]+))", line)
        if running:
            current_file = running.group(1) or f"tests/{running.group(2)}"
            continue
        line = line.strip()
        if line.endswith(": test"):
            tests[line[: -len(": test")]] = current_file
    return dict(sorted(tests.items()))


def candidate_suite(rust_id: str, rust_file: str) -> str | None:
    head = rust_id.split("::", 1)[0]
    if rust_file == "src/lib.rs" and head in {"models", "database", "parser", "classifier", "cli", "matcher"}:
        if head == "cli":
            return "test_cli_serialization.py"
        return f"test_{head}.py"
    if rust_file == "tests/cli.rs":
        return "test_cli.py"
    if rust_file == "tests/matcher.rs":
        return "test_matcher.py"
    if rust_file == "tests/real_world.rs":
        return "test_real_world.py"
    if rust_file == "tests/schema.rs":
        return "test_schema.py"
    if rust_file == "tests/smoke.rs":
        return "test_smoke.py"
    return None


def rust_location(rust_id: str, rust_files: dict[str, str]) -> str:
    rust_file = rust_files[rust_id]
    if rust_file == "src/lib.rs":
        head = rust_id.split("::", 1)[0]
        if head in {"models", "database", "parser", "classifier", "cli", "matcher"}:
            return f"src/{head}.rs"
    return rust_file


def main() -> None:
    py_tests = python_tests()
    rust_files = rust_tests()
    rust_ids = list(rust_files)

    by_suite_name: dict[tuple[str, str], list[str]] = defaultdict(list)
    for rust_id in rust_ids:
        suite = candidate_suite(rust_id, rust_files[rust_id])
        if suite:
            by_suite_name[(suite, rust_id.split("::")[-1])].append(rust_id)

    rows: list[tuple[PythonTest, str]] = []
    used: set[str] = set()
    for py in py_tests:
        rust_name = SPECIAL_NAME_MAP.get((py.suite, py.class_name, py.function), py.function)
        matches = [m for m in by_suite_name.get((py.suite, rust_name), []) if m not in used]
        if py.class_name:
            class_module = snake_case(py.class_name)
            class_matches = [m for m in matches if f"::{class_module}::" in m]
            if class_matches:
                matches = class_matches
        if len(matches) != 1:
            raise SystemExit(f"expected one Rust match for {py.id} via {rust_name}, found {matches}")
        rust_id = matches[0]
        used.add(rust_id)
        rows.append((py, rust_id))

    suite_counts = Counter(py.suite for py, _ in rows)
    for suite, expected in EXPECTED_SUITE_COUNTS.items():
        actual = suite_counts[suite]
        if actual != expected:
            raise SystemExit(f"{suite}: expected {expected}, got {actual}")

    extra = [test for test in rust_ids if test not in used]
    expected_extra_missing = EXPECTED_EXTRA_TESTS - set(extra)
    if expected_extra_missing:
        raise SystemExit(f"expected extra tests missing from cargo list: {sorted(expected_extra_missing)}")

    lines: list[str] = []
    lines.append("# Port manifest")
    lines.append("")
    lines.append("This generated manifest maps the Python pytest suite at `../bash-classify/tests` to the Rust tests that cover the same behavior. Regenerate it with `scripts/gen_port_manifest.py` after adding, renaming, or moving ported tests.")
    lines.append("")
    lines.append("## Suite arithmetic")
    lines.append("")
    lines.append("| Python suite | Ported tests | Rust locations |")
    lines.append("|---|---:|---|")
    for suite in SUITE_ORDER:
        locations = sorted({rust_location(rust_id, rust_files) for py, rust_id in rows if py.suite == suite})
        lines.append(f"| `tests/{suite}` | {suite_counts[suite]} | {', '.join(f'`{loc}`' for loc in locations)} |")
    arithmetic = " + ".join(str(EXPECTED_SUITE_COUNTS[suite]) for suite in SUITE_ORDER)
    lines.append(f"| **Total** | **{arithmetic} = {sum(EXPECTED_SUITE_COUNTS.values())}** | |")
    lines.append("")
    lines.append("## Per-test map")
    lines.append("")
    lines.append("| # | Python test id | Rust test id | Rust file |")
    lines.append("|---:|---|---|---|")
    for index, (py, rust_id) in enumerate(rows, 1):
        lines.append(f"| {index} | `{py.id}` | `{rust_id}` | `{rust_location(rust_id, rust_files)}` |")
    lines.append("")
    lines.append("## Extra Rust regression/helper tests")
    lines.append("")
    lines.append("These tests are additional coverage beyond the one-for-one Python port. They cover the golden replay gate, CLI matrix rows, user override scenarios, Rust-only helper seams, and serialization edge cases introduced by the native implementation.")
    lines.append("")
    lines.append("| Rust test id | Rust file |")
    lines.append("|---|---|")
    for rust_id in extra:
        lines.append(f"| `{rust_id}` | `{rust_location(rust_id, rust_files)}` |")
    lines.append("")
    lines.append(f"Ported Python tests: {len(rows)}. Extra Rust tests: {len(extra)}. Total Rust tests listed by Cargo: {len(rust_ids)}.")
    lines.append("")
    OUTPUT.write_text("\n".join(lines), encoding="utf-8")


if __name__ == "__main__":
    main()
