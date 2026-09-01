<!-- SPDX-FileCopyrightText: 2026 Chrysent LLC -->
<!-- SPDX-License-Identifier: MIT -->
# Parity notes

The Rust port is judged against the observed behavior of Python `bash-classify` at the vendored source commit, not against an idealized interpretation of the prose specification.

## Authority rule

Observed Python behavior at commit `8553440aa4d935dd14a8cbb15338ecac92100720` is the oracle. The evidence is:

- the ported Python test suite listed in `PORT_MANIFEST.md`;
- byte-for-byte golden replay fixtures under `tests/golden/cases/`;
- vendor integrity checks against `VENDOR.json`.

When `vendor/SPEC.md` and observed Python behavior disagree, Rust follows Python and documents the discrepancy here.

## Known discrepancies and accepted differences

| Area | Python behavior or source text | Rust behavior | Parity status |
|---|---|---|---|
| Exit code 1 prose | `vendor/SPEC.md` describes exit code 1 as a parse-error outcome. | Rust follows Python: stripped-empty stdin exits 1 with no output, and stdin timeout exits 1 with an error message. Parse irregularities normally appear as `parse_warnings` in successful JSON output. | Documented source/prose discrepancy. |
| Output schema prose | The vendored spec's output-schema prose omits some `risk` fields. | Rust emits `risk` at expression, command, and inner-command levels because Python does. | Documented source/prose discrepancy. |
| Help text program name | Python help says `bash-classify`. | Rust help says `rust-bash-classify`. | Product-name difference pending the distribution/name decision. |
| Timeout/internal-error program name | Python stderr prefixes use `bash-classify`. | Rust stderr prefixes use `rust-bash-classify`. | Product-name difference pending the wording/name decision. |
| Version output | Python prints `bash-classify 0.9.1` at the recorded source commit. | Rust prints `rust-bash-classify 0.1.0`. | Version-scheme decision remains open. |
| Invalid UTF-8 on stdin | Python's exact decode failure wording comes from Python's text I/O path. | Rust reports invalid input through its stderr-only error path. | Error-path-only wording difference; no successful JSON parity impact. |
| Recursion depth | Python's parser fails at the interpreter recursion limit for extreme nesting. | Rust uses `MAX_WALK_DEPTH = 256` and reports the same class of maximum-recursion-depth parse warning. | Threshold approximation; outcome class preserved. |
| YAML implementation | Python uses PyYAML. | Rust uses `serde_yaml`. | No YAML 1.1 semantic difference is known from the vendored corpus or ported tests; future discoveries follow the oracle rule. |
| Library error representation | Python raises exceptions from fallible library calls. | Rust returns typed `Result` values and maps them to matching CLI error text. | Language-level API difference; CLI behavior remains the parity surface. |

## Future divergence adjudication

Use this process for any suspected divergence:

1. Reduce the divergence to an exact Bash input, command YAML fixture, or CLI condition.
2. Run the Python oracle at the recorded commit and the Rust binary under an empty temporary `BASH_CLASSIFY_CONFIG_DIR`.
3. Compare stdout bytes, stderr, and exit code. For successful classifications, compare the complete JSON output, including field order and omitted fields.
4. If Python and Rust differ, fix Rust unless a human explicitly changes the accepted parity contract.
5. Add or update a focused Rust test and, when the case belongs in the end-to-end corpus, regenerate golden fixtures with `scripts/harvest_goldens.py` instead of hand-editing expected output.

After a vendor sync, rerun the differential check by regenerating goldens from the new Python commit and inspecting the fixture diff. Golden regeneration is appropriate only when the Python source of truth changes behavior; otherwise replay failures indicate a Rust regression or an accidental fixture edit.

## Evidence locations

- `PORT_MANIFEST.md` — one-for-one Python test port manifest, with extra Rust regression/helper tests listed separately.
- `tests/golden/manifest.json` and `tests/golden/cases/` — harvested CLI oracle outputs.
- `benchmarks/report.txt` — local performance benchmark report over the golden corpus.
- `PORTING.md` — module map, implementation seams, and vendor policy.
