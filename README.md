# rust-bash-classify

`rust-bash-classify` is a standalone Rust port of [bash-classify](https://github.com/fprochazka/bash-classify), a command-line tool that parses Bash expressions and classifies each command by side-effect risk. It is designed for coding-agent approval hooks: low-risk commands can be auto-approved, while risky, unknown, or destructive commands can be sent to a human for approval.

The Rust port aims to provide the same verdicts as the Python original in a single native binary with no Python runtime dependency.

## What it reports

For each input expression, the tool emits formatted JSON with:

- an expression-level `classification`: `READONLY`, `LOCAL_EFFECTS`, `EXTERNAL_EFFECTS`, `DANGEROUS`, or `UNKNOWN`;
- an expression-level `risk`: `LOW`, `MEDIUM`, or `HIGH`;
- per-command results, including matched rule, argv, inner delegated commands, directories, redirects, and read/write paths when present.

Example:

```bash
$ echo 'git status && grep -R TODO src' | rust-bash-classify
{
  "expression": "git status && grep -R TODO src",
  "classification": "READONLY",
  "risk": "LOW",
  "directories": [],
  "commands": [
    ...
  ]
}
```

## Build from source

The current way to get the tool is to build it from a source checkout:

```bash
cargo build --release
echo 'kubectl get pods -n production' | target/release/rust-bash-classify
```

The distribution channel is not chosen yet. Future packaging may use crates.io, prebuilt binaries, plugin integration, or another release path.

## Usage

```bash
echo 'ls -la' | rust-bash-classify
rust-bash-classify --help
rust-bash-classify --version
```

The CLI reads one Bash expression from stdin. Non-empty input is classified and written to stdout as JSON. Empty input exits with status 1 and no output. If no stdin data is available before the readiness timeout, the command exits with status 1 and a timeout message on stderr. Internal errors exit with status 2.

## Relationship to the Python original

The Python project remains the source of truth for command definitions, schema, and observed behavior. This repository vendors those inputs under `vendor/`:

- `vendor/commands/*.yaml` — built-in command classification database;
- `vendor/schemas/command.schema.json` — YAML schema;
- `vendor/SPEC.md` — upstream behavior specification.

The Rust binary embeds the vendored command database at compile time, so normal runtime classification does not need external YAML files. User overrides are still supported from `~/.config/bash-classify/commands/` or `$BASH_CLASSIFY_CONFIG_DIR/commands/`, matching the Python behavior.

Parity evidence is maintained in three layers:

1. ported Rust tests for the Python pytest suite, audited in [`PORT_MANIFEST.md`](PORT_MANIFEST.md);
2. golden CLI fixtures harvested from Python and replayed byte-for-byte by `tests/golden_replay.rs`;
3. vendor drift checks that ensure the copied YAML/schema/spec data still matches the recorded Python commit.

See [`PORTING.md`](PORTING.md) for the module map and implementation notes, and [`PARITY.md`](PARITY.md) for known accepted differences such as binary name and version wording.

## Development

Useful local checks:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build
cargo test
scripts/vendor.sh --check
scripts/harvest_goldens.py --check
scripts/gen_port_manifest.py
scripts/bench.sh
```

`cargo test` includes unit tests, integration tests, schema validation, and golden replay. `scripts/bench.sh` builds the release binary and compares Rust against Python over the committed golden corpus; the latest committed result is in `benchmarks/report.txt`.

## Vendor updates

Do not edit vendored files by hand. To sync from the Python source repository, use `scripts/vendor.sh`, regenerate `src/commands_data.rs`, and regenerate golden outputs only when Python behavior has intentionally changed. The recorded source commit and tree-sitter version pair live in `VENDOR.json`.

## License

MIT, matching the Python project.
