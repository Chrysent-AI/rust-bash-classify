#!/usr/bin/env bash
# Benchmark rust-bash-classify against the Python bash-classify oracle.
#
# Methodology:
# - Builds the Rust release binary first and measures that binary only.
# - Uses every tests/golden/cases/*/input.txt fixture as the benchmark corpus.
# - Runs one unmeasured warmup pass for each implementation over the full corpus.
# - Measures one fresh process per case, matching hook-style CLI invocation costs.
# - Reports aggregate wall time and per-case p95 wall time across the corpus.
# - Measures startup-to-first-output with one trivial expression by timing from
#   process spawn to the first stdout byte, also after a warmup.
# - Isolates user configuration with a temporary empty BASH_CLASSIFY_CONFIG_DIR so
#   local overrides do not affect parity or timings.
#
# The acceptance bar for SPEC-0001 SC-5 is loose parity: Rust must be no slower
# than Python for aggregate corpus time, corpus p95, and startup-to-first-output.

set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
PYTHON_REPO=${PYTHON_REPO:-../bash-classify}
REPORT=${REPORT:-"$ROOT_DIR/benchmarks/report.txt"}
RUST_BIN="$ROOT_DIR/target/release/rust-bash-classify"
PYTHON_CMD=(uv run --project "$PYTHON_REPO" bash-classify)

cd "$ROOT_DIR"
mkdir -p "$(dirname "$REPORT")"

cargo build --release >/dev/null

python3 - "$ROOT_DIR" "$PYTHON_REPO" "$RUST_BIN" "$REPORT" <<'PY'
from __future__ import annotations

import datetime as dt
import os
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

root = Path(sys.argv[1])
python_repo = Path(sys.argv[2])
rust_bin = Path(sys.argv[3])
report_path = Path(sys.argv[4])
python_cmd = ["uv", "run", "--project", str(python_repo), "bash-classify"]
rust_cmd = [str(rust_bin)]
case_paths = sorted((root / "tests" / "golden" / "cases").glob("*/input.txt"))
if not case_paths:
    raise SystemExit("no golden input cases found")

trivial = b"echo hello\n"


def run_once(cmd: list[str], payload: bytes, env: dict[str, str], require_stdout: bool = False) -> int:
    start = time.perf_counter_ns()
    proc = subprocess.run(cmd, input=payload, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env, check=False)
    end = time.perf_counter_ns()
    if proc.returncode not in {0, 1}:
        stderr = proc.stderr.decode("utf-8", errors="replace")
        raise SystemExit(f"command {cmd[0]} failed with exit {proc.returncode}: {stderr}")
    if require_stdout and not proc.stdout:
        raise SystemExit(f"command {cmd[0]} produced no stdout for non-empty input")
    return end - start


def warmup(cmd: list[str], env: dict[str, str]) -> None:
    for path in case_paths:
        run_once(cmd, path.read_bytes(), env)
    run_once(cmd, trivial, env, require_stdout=True)


def measure_corpus(cmd: list[str], env: dict[str, str]) -> list[int]:
    return [run_once(cmd, path.read_bytes(), env) for path in case_paths]


def first_byte_ns(cmd: list[str], env: dict[str, str]) -> int:
    start = time.perf_counter_ns()
    proc = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
    assert proc.stdin is not None
    assert proc.stdout is not None
    proc.stdin.write(trivial)
    proc.stdin.close()
    first = proc.stdout.read(1)
    first_at = time.perf_counter_ns()
    stdout_rest = proc.stdout.read()
    stderr = proc.stderr.read() if proc.stderr is not None else b""
    code = proc.wait()
    if code != 0:
        raise SystemExit(f"command {cmd[0]} failed during first-byte measurement: {stderr.decode('utf-8', errors='replace')}")
    if not first:
        raise SystemExit(f"command {cmd[0]} emitted no first byte; stderr={stderr!r}; rest={stdout_rest!r}")
    return first_at - start


def p95(values: list[int]) -> int:
    # Nearest-rank p95, deterministic for 175 samples.
    ordered = sorted(values)
    index = (95 * len(ordered) + 99) // 100 - 1
    return ordered[index]


def ms(ns: int) -> float:
    return ns / 1_000_000


def summarize(times: list[int], startup: int) -> dict[str, int]:
    return {"aggregate_ns": sum(times), "p95_ns": p95(times), "startup_first_byte_ns": startup}

with tempfile.TemporaryDirectory(prefix="rust-bash-classify-bench-config-") as cfg:
    env = os.environ.copy()
    env["BASH_CLASSIFY_CONFIG_DIR"] = cfg

    warmup(python_cmd, env)
    warmup(rust_cmd, env)

    python_times = measure_corpus(python_cmd, env)
    rust_times = measure_corpus(rust_cmd, env)

    # Warm the trivial path again immediately before first-byte measurements.
    run_once(python_cmd, trivial, env, require_stdout=True)
    run_once(rust_cmd, trivial, env, require_stdout=True)
    python_startup = first_byte_ns(python_cmd, env)
    rust_startup = first_byte_ns(rust_cmd, env)

python_summary = summarize(python_times, python_startup)
rust_summary = summarize(rust_times, rust_startup)
metrics = [
    ("aggregate corpus wall", "aggregate_ns"),
    ("per-case p95 wall", "p95_ns"),
    ("startup to first stdout byte", "startup_first_byte_ns"),
]

lines: list[str] = []
lines.append("# Benchmark report")
lines.append("")
lines.append(f"Generated: {dt.datetime.now(dt.timezone.utc).isoformat(timespec='seconds')}")
lines.append(f"Corpus: tests/golden/cases/*/input.txt ({len(case_paths)} cases)")
lines.append(f"Python: {' '.join(python_cmd)}")
lines.append(f"Rust: {rust_bin} (cargo build --release)")
lines.append("Warmup: one full unmeasured corpus pass per implementation plus trivial startup warmup")
lines.append("User config: temporary empty BASH_CLASSIFY_CONFIG_DIR")
lines.append("")
lines.append("| Metric | Python (ms) | Rust (ms) | Rust/Python | Result |")
lines.append("|---|---:|---:|---:|---|")
all_pass = True
for label, key in metrics:
    py = python_summary[key]
    rs = rust_summary[key]
    ok = rs <= py
    all_pass = all_pass and ok
    ratio = rs / py if py else float("inf")
    lines.append(f"| {label} | {ms(py):.3f} | {ms(rs):.3f} | {ratio:.3f}x | {'PASS' if ok else 'FAIL'} |")
lines.append("")
lines.append(f"SC-5 verdict: {'PASS' if all_pass else 'FAIL'} — Rust is {'not slower' if all_pass else 'slower on at least one metric'}.")
lines.append("")
lines.append("Per-case timing details are intentionally not committed; rerun scripts/bench.sh to reproduce with local machine timings.")
text = "\n".join(lines) + "\n"
report_path.write_text(text, encoding="utf-8")
print(text, end="")
if not all_pass:
    raise SystemExit(1)
PY
