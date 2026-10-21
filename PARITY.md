<!-- SPDX-FileCopyrightText: 2026 Chrysent LLC -->
<!-- SPDX-License-Identifier: MIT -->
# Parity notes

The Rust repository is maintained from local code, command definitions and tests.
Its original Python compatibility evidence is preserved without requiring Python.

## Maintenance authority

The original port used observed behavior at commit
`8553440aa4d935dd14a8cbb15338ecac92100720`. That provenance is preserved in:

- the ported test mapping in `PORT_MANIFEST.md`;
- byte-for-byte regression fixtures under `tests/golden/cases/`;
- historical import metadata in `VENDOR.json`.

Current Rust behavior and tests are the maintenance baseline. Intentional changes
require review and local regression coverage, not upstream approval or execution.
`vendor/SPEC.md` is historical prose; its known discrepancies remain listed below.

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
| YAML implementation | Python uses PyYAML. | Rust uses `serde_yaml`. | No YAML 1.1 semantic difference is known from the vendored corpus or ported tests; future discoveries require local regression tests and review. |
| Library error representation | Python raises exceptions from fallible library calls. | Rust returns typed `Result` values and maps them to matching CLI error text. | Language-level API difference; CLI behavior remains the parity surface. |

## Future behavior changes

1. Reduce the issue to exact Bash input, command YAML or a CLI condition.
2. Run the Rust binary under an empty temporary `BASH_CLASSIFY_CONFIG_DIR` unless
   the issue concerns user overrides.
3. Check stdout bytes, stderr and exit code, including JSON field order, omitted
   fields and trailing newlines.
4. Add focused Rust regression coverage for the accepted behavior. For command
   data changes, also run schema tests and regenerate the local embed table.
5. Review intentional expectation changes; never alter imported evidence merely
   to conceal a regression. No external checkout or generator is required.

## Evidence locations

- `PORT_MANIFEST.md` — one-for-one Python test port manifest, with extra Rust regression/helper tests listed separately.
- `tests/golden/manifest.json` and `tests/golden/cases/` — committed CLI regression inputs and expected outputs.
- `benchmarks/report.txt` — historical comparative benchmark report over the golden corpus.
- `PORTING.md` — module map, implementation seams, and vendor policy.
