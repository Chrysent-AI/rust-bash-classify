<!-- SPDX-FileCopyrightText: 2026 Chrysent LLC -->
<!-- SPDX-License-Identifier: MIT -->
# AGENTS.md — rust-bash-classify agent guidance

## Self-contained maintenance

This repository is the maintained source of truth. It originated as a behavioral
port of Python `bash-classify`; `VENDOR.json`, `PORTING.md`, `PORT_MANIFEST.md` and
`PARITY.md` retain historical provenance. No Python checkout, interpreter, uv or
upstream access is required to build, test or maintain it. Rust dependencies and
the committed `Cargo.lock` remain required.

Preserve existing observable behavior unless an intentional change is approved.
For behavior changes, reduce the case to exact Bash input, YAML or a CLI condition
and add focused Rust regression tests. Check stdout bytes, stderr, exit codes,
JSON key order, omitted fields and trailing newlines. Use an empty temporary
`BASH_CLASSIFY_CONFIG_DIR` except when testing user overrides. For verdict changes,
include adversarial cases that could become unexpectedly more permissive.

## Local inputs and historical evidence

- `vendor/commands/*.yaml` and `vendor/schemas/command.schema.json` are locally
  maintained inputs. Changes require Rust regression and schema validation tests.
- `src/commands_data.rs` is generated: change local YAML then run
  `scripts/gen_commands_data.sh`; do not hand-edit the embed table.
- `vendor/SPEC.md` is an unchanged historical specification, not a live authority.
  Preserve upstream attribution and licenses; document discrepancies in `PARITY.md`.
- `tests/golden/cases/` preserves imported byte-level regression evidence. Do not
  rewrite expected output merely to make failures pass. Intentional behavior
  changes require reviewed expectations and Rust regression tests.
- `tests/golden/manifest.json` and `PORT_MANIFEST.md` record the original import
  and test mapping, not a continuously generated inventory of future Rust tests.
- Comparative benchmark reports are historical evidence, not current performance
  gates. Preserve their figures; their external comparative tooling is retired.

The upstream vendoring, golden harvesting, port-manifest and comparative benchmark
scripts have been retired. Their metadata may be maintained locally without them.

## Python-semantics helpers remain behavior contracts

Do not replace these with Rust standard-library behavior without regression
coverage proving preservation of the accepted behavior:

- `py_dirname` and `py_basename`, including trailing-slash behavior;
- `py_strip`, including control characters `\x1c`–`\x1f`;
- `PyAsciiFormatter`, including ASCII escaping, surrogate pairs and DEL;
- Python `repr`-style parse-warning text;
- empty environment-variable truthiness and shared alias-resolution errors.

Name new intentionally Python-compatible helpers `py_*`, document the counterpart,
and test the behavior locally. Historical compatibility does not require Python.

## Required gates before handoff

```bash
scripts/gen_commands_data.sh
git diff --exit-code -- src/commands_data.rs
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build
cargo test
```

Run relevant focused tests as well. `cargo test` includes schema validation and
byte-level golden replay. CI runs these local gates on its configured platform.
When intentionally changing the generated table, review and commit that change
before expecting the drift check to pass.

Keep `tree-sitter-bash` pinned (`=0.25.1`) and `Cargo.lock` committed. Grammar,
JSON and YAML dependency changes are behavior-risk work requiring focused tests
and reviewed expectations. `serde_yaml` remains intentionally accepted despite
its archived upstream status; do not replace it opportunistically.

Optional secret/PII hooks and their opt-in setup are documented in
`.githooks/README.md`. Keep exceptions narrow and rule-specific.

## Accepted release path and owner decisions

Stable `vX.Y.Z` tags and six-target Linux/macOS binstall archives are the accepted
release path; `RELEASING.md` documents the manual version and local commit/tag
helpers. Keep `0.1.0` unreleased until an explicit release is requested. Registry
publication is a separate manual step under the intended chrysent account;
metadata attribution does not establish registry ownership. For release-tooling
changes, run `bash tests/release-tools.sh`, shellcheck and actionlint, and verify
the Cargo package payload/build and native archive layout without publishing.
Preserve environment-file exclusion and do not edit or source local loaders.

Changes to this version/distribution contract, the long-term binary/product name,
or a plugin switch-over plan remain owner decisions. The original project's
future is outside this repository's maintenance contract.
