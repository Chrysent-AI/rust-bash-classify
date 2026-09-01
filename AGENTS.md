<!-- SPDX-FileCopyrightText: 2026 Chrysent LLC -->
<!-- SPDX-License-Identifier: MIT -->
# AGENTS.md — rust-bash-classify agent guidance

This repository is a behavioral parity port of the Python `bash-classify` project. Do not treat it as an independent Rust reimplementation.

## Core rule: parity first

- The Python repository at `../bash-classify` is the behavior oracle for this Rust repository.
- The recorded oracle commit is in `VENDOR.json`; `PARITY.md` explains the authority rule.
- If `vendor/SPEC.md` and observed Python behavior disagree, follow observed Python behavior unless the owner explicitly changes the accepted contract.
- Do not "fix," simplify, or idiomatically improve behavior in Rust just because the Python behavior looks odd. First reduce the issue to a Python/Rust differential case.

## How to verify behavior changes

For any behavior-affecting change:

1. Reduce the case to an exact Bash input, YAML fixture, or CLI condition.
2. Run the Python oracle from `../bash-classify` at the recorded/synced commit (e.g. `printf '%s' '<expr>' | uv run --project ../bash-classify bash-classify`).
3. Run the Rust implementation with an empty temporary `BASH_CLASSIFY_CONFIG_DIR` unless the test is specifically about user overrides.
4. Compare exit code, stdout bytes, stderr, JSON key order, omitted fields, and trailing newline — byte level, not string level.
5. Add a focused Rust regression test. If the case belongs in end-to-end CLI parity, regenerate golden fixtures with `scripts/harvest_goldens.py`; never hand-edit expected output.

## Script-owned files: do not hand-edit

Do not hand-edit these generated or vendored artifacts:

- `vendor/commands/*.yaml`
- `vendor/schemas/command.schema.json`
- `vendor/SPEC.md`
- `src/commands_data.rs`
- `tests/golden/**` generated fixture content, including `tests/golden/README.md` and `tests/golden/manifest.json`
- `PORT_MANIFEST.md`
- `benchmarks/report.txt` except by rerunning the benchmark script when intentionally updating benchmark evidence

Use the owning script instead:

- `scripts/vendor.sh` for vendored files and `VENDOR.json` sync metadata
- `scripts/gen_commands_data.sh` for `src/commands_data.rs`
- `scripts/harvest_goldens.py` for `tests/golden/**`
- `scripts/gen_port_manifest.py` for `PORT_MANIFEST.md`
- `scripts/bench.sh` for benchmark report updates

If a generated file needs text changes, change the generator/template and regenerate. The drift checks will reject direct edits to generated output — this is by design and has already caught a wrong-path fix during the port.

## Python-semantics helpers are mandatory

Do not replace Python-observable helper behavior with Rust standard-library behavior unless a differential check proves equivalence for the case.

Important helpers/seams include:

- `py_dirname` semantics in classifier directory capture; Rust `Path` behavior is not a substitute (trailing-slash behavior diverges: Python `os.path.dirname('/a/b/')` is `'/a/b'`).
- `py_basename` for command lookup basename fallback.
- `py_strip` semantics for Python `str.strip()` whitespace in CLI input handling (includes `\x1c`-`\x1f`, which Rust `trim()` does not strip).
- Python-style JSON escaping via `PyAsciiFormatter` (ensure_ascii semantics, surrogate pairs for astral chars, DEL handling); plain `serde_json` escaping is not enough for byte parity.
- Python `repr`-style parse-warning text helpers.
- Python truthiness rules around environment variables (empty `BASH_CLASSIFY_CONFIG_DIR` behaves as unset).
- Alias resolution error text and chain-depth behavior through the shared alias-resolution seam.

When adding a new helper, name it `py_*` if it intentionally mirrors Python semantics, document the Python counterpart, and add focused tests.

## Required gates before handoff

Run the relevant focused tests plus the standard gates before handing work off:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build
cargo test
scripts/vendor.sh --check
scripts/harvest_goldens.py --check
scripts/gen_port_manifest.py && git diff --exit-code -- PORT_MANIFEST.md
scripts/bench.sh    # when performance evidence or release-readiness docs are touched
```

Notes:

- `cargo test` includes unit tests, integration tests, schema validation, and golden replay (489 tests at last count: 463 ported Python tests + regression/helper additions).
- These gates are enforced in CI (`.github/workflows/ci.yml`, Linux + macOS), including a vendored-data drift step that clones upstream at the `VENDOR.json` commit.
- If you modify generated fixtures or manifests, also run the owning generator and verify `git diff` is expected.
- If you touch CLI serialization, run byte-level Python/Rust differential checks, including non-ASCII/control characters and error/timeout paths.
- If you touch parser/matcher/classifier behavior, include Python/Rust differential probes in the handoff — for verdict-path changes, probe in the adversarial direction (find inputs where Rust is more permissive than Python; those are the unacceptable ones).

## Grammar and dependency pin discipline

- `tree-sitter-bash` is pinned (`=0.25.1`) to match the Python side's resolved grammar version. The version pair is recorded in `VENDOR.json` and enforced by vendor checks.
- Do not casually bump `tree-sitter`, `tree-sitter-bash`, YAML, JSON, or schema dependencies. A grammar bump requires sync discipline and golden regeneration if behavior changes.
- `serde_yaml` is intentionally accepted (archived upstream) because the YAML format is frozen and schema-validated; changing YAML parsers is behavior-risk work.
- Keep `Cargo.lock` committed.

## Behavioral contract locations

- `vendor/SPEC.md` is the vendored upstream prose specification.
- `PARITY.md` is this repo's authority and discrepancy log.
- `PORTING.md` explains module mapping, intentional Rust seams, and documented deviations.
- `PORT_MANIFEST.md` maps all 463 Python tests to Rust tests.
- `tests/golden/manifest.json` and `tests/golden/cases/` are byte-level CLI oracle evidence (175 cases).

Authority rule: observed Python behavior pinned by tests/goldens beats prose when they disagree. Document known discrepancies in `PARITY.md`.

## Vendoring sync workflow

When the Python source of truth changes:

1. Sync from `../bash-classify` with `scripts/vendor.sh`.
2. Regenerate `src/commands_data.rs` with `scripts/gen_commands_data.sh`.
3. If behavior changed, regenerate golden fixtures with `scripts/harvest_goldens.py`.
4. Regenerate `PORT_MANIFEST.md` if tests were added/renamed/moved.
5. Run `scripts/vendor.sh --check`, `scripts/harvest_goldens.py --check`, `cargo test`, fmt, clippy, and any relevant differential probes.
6. Inspect fixture diffs carefully. Golden regeneration is valid only when Python behavior intentionally changed; otherwise a replay failure is a Rust regression or accidental fixture edit — investigate, do not regenerate around it.

## Open decisions that still need the owner

Do not decide these unilaterally in code or docs:

- Version scheme and `--version` output.
- Distribution channel: source build, `cargo install`, crates.io, prebuilt binaries, or another release path.
- Long-term binary/product naming: `rust-bash-classify`, `bash-classify`, both, or a plugin-specific wrapper.
- Claude Code/plugin continuity and switch-over plan.
- Long-term fate of the Python repository: permanent oracle vs eventual freeze/deprecation.

Docs may say these are open; they must not imply a chosen release/distribution/plugin path without owner confirmation.

## Keep the upstream Python repo clean

The Python repo is the oracle. Treat it as read-only during Rust work except for explicit oracle commands. After differential probes, check both repositories for accidental modifications or generated environment directories (a `.venv/` from `uv run` is expected and git-ignored there; nothing else should appear).
