#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Chrysent LLC
# SPDX-License-Identifier: MIT
"""Harvest Python bash-classify test expressions into Rust golden fixtures.

Regeneration policy: run this script only after a vendor sync or Python
source-of-truth behavior update. Review the resulting fixture diff. Files under
``tests/golden/`` are script-owned expectations and must not be hand-edited to
make a failing replay test pass.
"""

from __future__ import annotations

import argparse
import ast
import filecmp
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 is unsupported here.
    print("Python 3.11+ is required", file=sys.stderr)
    raise

EXPECTED_SOURCE_COMMIT = "8553440aa4d935dd14a8cbb15338ecac92100720"
SOURCE_TAG = "v0.9.1"
DEFAULT_SOURCE_REPO = Path(os.environ.get("BASH_CLASSIFY_SOURCE_DIR", "/home/chrysent/devel/bash-classify"))
DEFAULT_OUTPUT = Path("tests/golden")
HARVESTED_CALLS = {"classify_expression", "_run_cli"}


@dataclass
class Origin:
    test_id: str
    file: str
    line: int
    call: str


@dataclass
class Case:
    index: int
    slug: str
    expression: str
    expected_stdout: bytes
    expected_exit: int
    origins: list[Origin] = field(default_factory=list)

    @property
    def name(self) -> str:
        return f"{self.index:03d}-{self.slug}"


class ExpressionVisitor(ast.NodeVisitor):
    def __init__(self, rel_file: str) -> None:
        self.rel_file = rel_file
        self.class_stack: list[str] = []
        self.function_stack: list[str] = []
        self.expressions: list[tuple[str, Origin]] = []

    def visit_ClassDef(self, node: ast.ClassDef) -> Any:
        self.class_stack.append(node.name)
        self.generic_visit(node)
        self.class_stack.pop()

    def visit_FunctionDef(self, node: ast.FunctionDef) -> Any:
        self.function_stack.append(node.name)
        self.generic_visit(node)
        self.function_stack.pop()

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> Any:
        self.visit_FunctionDef(node)

    def visit_Call(self, node: ast.Call) -> Any:
        call_name = call_func_name(node.func)
        if call_name in HARVESTED_CALLS and node.args:
            first = node.args[0]
            if isinstance(first, ast.Constant) and isinstance(first.value, str):
                test_id = self.current_test_id()
                self.expressions.append(
                    (
                        first.value,
                        Origin(
                            test_id=test_id,
                            file=self.rel_file,
                            line=node.lineno,
                            call=call_name,
                        ),
                    )
                )
        self.generic_visit(node)

    def current_test_id(self) -> str:
        parts = [*self.class_stack, *self.function_stack]
        if parts:
            return f"{self.rel_file}::{'::'.join(parts)}"
        return self.rel_file


def call_func_name(func: ast.expr) -> str | None:
    if isinstance(func, ast.Name):
        return func.id
    if isinstance(func, ast.Attribute):
        return func.attr
    return None


def slugify(expression: str) -> str:
    text = expression.strip().lower()
    text = re.sub(r"[^a-z0-9]+", "-", text)
    text = text.strip("-")
    if not text:
        text = "empty"
    return text[:48].strip("-") or "case"


def git_output(repo: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()


def ensure_clean_source(source_repo: Path) -> str:
    status = git_output(source_repo, "status", "--short")
    if status:
        raise SystemExit(f"source repo is dirty; refusing to harvest:\n{status}")
    commit = git_output(source_repo, "rev-parse", "HEAD")
    if commit != EXPECTED_SOURCE_COMMIT:
        raise SystemExit(
            f"source repo is at {commit}, expected {EXPECTED_SOURCE_COMMIT}; refusing to harvest"
        )
    return commit


def harvest_expressions(source_repo: Path) -> list[tuple[str, list[Origin]]]:
    seen: dict[str, list[Origin]] = {}
    ordered: list[str] = []
    tests_dir = source_repo / "tests"
    for path in sorted(tests_dir.glob("test_*.py")):
        rel_file = path.relative_to(source_repo).as_posix()
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        visitor = ExpressionVisitor(rel_file)
        visitor.visit(tree)
        for expression, origin in visitor.expressions:
            if expression not in seen:
                seen[expression] = []
                ordered.append(expression)
            seen[expression].append(origin)
    return [(expression, seen[expression]) for expression in ordered]


def run_python_cli(source_repo: Path, expression: str) -> tuple[bytes, int]:
    with tempfile.TemporaryDirectory(prefix="bash-classify-golden-config-") as cfg:
        env = os.environ.copy()
        env["BASH_CLASSIFY_CONFIG_DIR"] = cfg
        proc = subprocess.run(
            ["uv", "run", "--project", str(source_repo), "bash-classify"],
            input=expression.encode("utf-8"),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
            check=False,
        )
    if proc.stderr:
        print(
            f"warning: Python CLI wrote stderr for expression {expression!r}: "
            f"{proc.stderr.decode('utf-8', errors='replace')}",
            file=sys.stderr,
        )
    return proc.stdout, int(proc.returncode)


def lock_package_version(source_repo: Path) -> str:
    return subprocess.check_output(
        [
            "uv",
            "run",
            "--project",
            str(source_repo),
            "python",
            "-c",
            "import importlib.metadata; print(importlib.metadata.version('bash-classify'))",
        ],
        text=True,
    ).strip()


def package_versions_from_uv_lock(source_repo: Path) -> dict[str, str]:
    lock = tomllib.loads((source_repo / "uv.lock").read_text(encoding="utf-8"))
    versions: dict[str, str] = {}
    for package in lock.get("package", []):
        name = package.get("name")
        if name in {"tree-sitter", "tree-sitter-bash"}:
            versions[name.replace("-", "_")] = package.get("version", "")
    return versions


def rust_pins(rust_repo: Path) -> dict[str, str]:
    manifest = tomllib.loads((rust_repo / "Cargo.toml").read_text(encoding="utf-8"))
    lock = tomllib.loads((rust_repo / "Cargo.lock").read_text(encoding="utf-8"))
    pins: dict[str, str] = {}
    deps = manifest.get("dependencies", {})
    for name in ("tree-sitter", "tree-sitter-bash"):
        dep = deps.get(name)
        if isinstance(dep, str):
            pins[f"{name}_requirement"] = dep
        elif isinstance(dep, dict):
            pins[f"{name}_requirement"] = str(dep.get("version", ""))
    for package in lock.get("package", []):
        name = package.get("name")
        if name in {"tree-sitter", "tree-sitter-bash"}:
            pins[name.replace("-", "_")] = package.get("version", "")
    return pins


def sha256_file(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def make_cases(source_repo: Path, harvested: list[tuple[str, list[Origin]]]) -> list[Case]:
    cases: list[Case] = []
    used_slugs: Counter[str] = Counter()
    for index, (expression, origins) in enumerate(harvested, start=1):
        base_slug = slugify(expression)
        used_slugs[base_slug] += 1
        slug = base_slug if used_slugs[base_slug] == 1 else f"{base_slug}-{used_slugs[base_slug]}"
        stdout, exit_code = run_python_cli(source_repo, expression)
        cases.append(
            Case(
                index=index,
                slug=slug,
                expression=expression,
                expected_stdout=stdout,
                expected_exit=exit_code,
                origins=origins,
            )
        )
    return cases


def build_manifest(source_repo: Path, rust_repo: Path, script_path: Path, cases: list[Case]) -> dict[str, Any]:
    python_versions = package_versions_from_uv_lock(source_repo)
    manifest_cases = []
    for case in cases:
        manifest_cases.append(
            {
                "name": case.name,
                "origin": case.origins[0].test_id,
                "origin_file": case.origins[0].file,
                "origin_line": case.origins[0].line,
                "origin_call": case.origins[0].call,
                "all_origins": [origin.__dict__ for origin in case.origins],
                "expected_exit": case.expected_exit,
                "input_sha256": hashlib.sha256(case.expression.encode("utf-8")).hexdigest(),
                "expected_stdout_sha256": hashlib.sha256(case.expected_stdout).hexdigest(),
            }
        )
    return {
        "schema_version": 1,
        "regeneration_policy": (
            "Regenerate only by running scripts/harvest_goldens.py after a vendor sync "
            "or Python source-of-truth behavior update; review the diff; never hand-edit tests/golden outputs."
        ),
        "source": {
            "repository": "https://github.com/fprochazka/bash-classify",
            "commit": EXPECTED_SOURCE_COMMIT,
            "tag": SOURCE_TAG,
            "package_version": lock_package_version(source_repo),
        },
        "python": python_versions,
        "rust": rust_pins(rust_repo),
        "generator": {
            "path": "scripts/harvest_goldens.py",
            "sha256": sha256_file(script_path),
        },
        "case_count": len(cases),
        "cases": manifest_cases,
    }


def write_fixture_tree(output_dir: Path, manifest: dict[str, Any], cases: list[Case]) -> None:
    if output_dir.exists():
        shutil.rmtree(output_dir)
    cases_dir = output_dir / "cases"
    cases_dir.mkdir(parents=True)
    for case in cases:
        case_dir = cases_dir / case.name
        case_dir.mkdir()
        (case_dir / "input.txt").write_bytes(case.expression.encode("utf-8"))
        (case_dir / "expected.json").write_bytes(case.expected_stdout)
    (output_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    (output_dir / "README.md").write_text(
        "<!-- SPDX-FileCopyrightText: 2026 Chrysent LLC -->\n"
        "<!-- SPDX-License-Identifier: MIT -->\n\n"
        "# Golden corpus\n\n"
        "These fixtures are generated by `scripts/harvest_goldens.py` from the "
        "Python bash-classify test suite and Python CLI output.\n\n"
        "## Regeneration policy\n\n"
        "Regenerate only after a vendor sync or Python source-of-truth behavior "
        "update:\n\n"
        "```bash\n"
        "python3 scripts/harvest_goldens.py\n"
        "python3 scripts/harvest_goldens.py --check\n"
        "cargo test --test golden_replay\n"
        "```\n\n"
        "Review the fixture diff. `expected.json` files contain exact stdout "
        "bytes captured from the Python CLI, including trailing newlines, and "
        "must never be hand-edited to make Rust replay failures pass.\n\n"
        "## Future use\n\n"
        "Running the Python implementation as a live oracle inside CI "
        "(differential testing) is deliberately deferred. This corpus is the "
        "ready-made input for activating that harness later: the same case "
        "directories can be replayed against both binaries with the expected "
        "outputs regenerated from Python.\n",
        encoding="utf-8",
    )


def dirs_match(left: Path, right: Path) -> tuple[bool, list[str]]:
    differences: list[str] = []

    def walk(rel: Path = Path(".")) -> None:
        left_dir = left / rel
        right_dir = right / rel
        left_names = {p.name for p in left_dir.iterdir()} if left_dir.exists() else set()
        right_names = {p.name for p in right_dir.iterdir()} if right_dir.exists() else set()
        for name in sorted(left_names - right_names):
            differences.append(f"only in generated: {(rel / name).as_posix()}")
        for name in sorted(right_names - left_names):
            differences.append(f"only in committed: {(rel / name).as_posix()}")
        for name in sorted(left_names & right_names):
            child_rel = rel / name
            l_child = left / child_rel
            r_child = right / child_rel
            if l_child.is_dir() and r_child.is_dir():
                walk(child_rel)
            elif l_child.is_file() and r_child.is_file():
                if not filecmp.cmp(l_child, r_child, shallow=False):
                    differences.append(f"content differs: {child_rel.as_posix()}")
            else:
                differences.append(f"type differs: {child_rel.as_posix()}")

    walk()
    return not differences, differences


def source_breakdown(cases: list[Case]) -> dict[str, int]:
    counts: Counter[str] = Counter(case.origins[0].file for case in cases)
    return dict(sorted(counts.items()))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=DEFAULT_SOURCE_REPO)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--check", action="store_true", help="regenerate in a temp dir and compare")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    rust_repo = Path(__file__).resolve().parents[1]
    source_repo = args.source.resolve()
    output_dir = args.output if args.output.is_absolute() else rust_repo / args.output
    script_path = Path(__file__).resolve()

    ensure_clean_source(source_repo)
    harvested = harvest_expressions(source_repo)
    cases = make_cases(source_repo, harvested)
    manifest = build_manifest(source_repo, rust_repo, script_path, cases)

    if args.check:
        with tempfile.TemporaryDirectory(prefix="bash-classify-goldens-check-") as temp:
            generated = Path(temp) / "golden"
            write_fixture_tree(generated, manifest, cases)
            ok, differences = dirs_match(generated, output_dir)
            if not ok:
                print("golden fixtures are out of date", file=sys.stderr)
                for difference in differences[:200]:
                    print(difference, file=sys.stderr)
                if len(differences) > 200:
                    print(f"... and {len(differences) - 200} more differences", file=sys.stderr)
                return 1
    else:
        write_fixture_tree(output_dir, manifest, cases)

    print(f"harvested {len(cases)} golden cases")
    print("breakdown:")
    for file, count in source_breakdown(cases).items():
        print(f"  {file}: {count}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
