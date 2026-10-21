<!-- SPDX-FileCopyrightText: 2026 Chrysent LLC -->
<!-- SPDX-License-Identifier: MIT -->

# rust-bash-classify

Classify Bash commands by side-effect risk as a fast, dependency-free native binary.

`rust-bash-classify` reads one Bash expression on stdin and emits JSON describing the risk of each command in the expression. It is built for AI-agent safety hooks: low-risk commands such as reads and status checks can be auto-approved, while writes, external effects, unknown commands, and dangerous commands can be routed to a human.

## Attribution

This project is inspired by and behaviorally ported from [bash-classify](https://github.com/fprochazka/bash-classify) by Filip Prokazka. The command database, JSON schema, and upstream specification are sourced from that project under MIT and vendored in `vendor/`.

Chrysent LLC maintains this Rust port and claims no ownership over vendored upstream files.

## Why use it

- **Fast hook startup:** native binary with no Python runtime dependency.
- **Compatibility evidence:** byte-identical CLI output against committed historical golden fixtures.
- **Agent-friendly JSON:** expression-level risk plus per-command details, delegated inner commands, redirects, read/write paths, and directory effects.
- **Configurable command rules:** built-in database plus user YAML overrides.

## Performance

The committed benchmark corpus contains 175 golden CLI cases harvested during the original Python port. In the historical committed measurement, Rust was roughly 100x faster while producing byte-identical output:

| Metric | Python | Rust |
|---|---:|---:|
| Aggregate corpus wall time | 13.3s | 109ms |
| Per-case p95 wall time | 89ms | ~1ms |
| Startup to first stdout byte | 75ms | ~0.6ms |

See `benchmarks/report.txt` for historical timings, not a current performance guarantee. Comparative tooling has been retired; no Python checkout is needed for development.

A second benchmark ran a real-world corpus of 51,882 bash commands through both
implementations. Spawned as a CLI per command — the permission-hook usage pattern — Rust was
**~43× faster (38.1s vs 27.3 min, both fully measured)**, while in a single long-lived process
the two engines are roughly tied — both are bound by the same tree-sitter grammar. Both
implementations produced identical aggregate verdicts on every command (64% READONLY, 15%
DANGEROUS, 17% UNKNOWN; 66% LOW risk). See `benchmarks/corpus-report.md`.

## Install

Requires Rust stable.

Build from a checkout:

```bash
git clone https://github.com/Chrysent-AI/rust-bash-classify.git
cd rust-bash-classify
cargo build --release
```

Install directly from Git:

```bash
cargo install --git https://github.com/Chrysent-AI/rust-bash-classify
```

The crate and binary are currently named `rust-bash-classify`.

Version 0.1.0 is not yet released. The release pipeline is prepared for x86_64 and
aarch64 Linux (GNU and musl) and Intel/Apple Silicon macOS. Once a version is
published on crates.io and its official GitHub archives are available, install
prebuilt binaries with `cargo binstall rust-bash-classify`. This is not a claim
that those assets are already published or that all hosted targets have run.
See `RELEASING.md` for the archive/checksum contract and manual publication steps.

## Usage

The CLI reads one Bash expression from stdin and writes formatted JSON to stdout.

```bash
printf '%s' 'git status && grep -R TODO src' | rust-bash-classify
```

```json
{
  "expression": "git status && grep -R TODO src",
  "classification": "READONLY",
  "risk": "LOW",
  "directories": [],
  "commands": [
    {
      "command": [
        "git",
        "status"
      ],
      "argv": [
        "git",
        "status"
      ],
      "classification": "READONLY",
      "risk": "LOW",
      "matched_rule": "git.status",
      "classification_reason": "base classification from rule git.status",
      "inner_commands": []
    },
    {
      "command": [
        "grep"
      ],
      "argv": [
        "grep",
        "-R",
        "TODO",
        "src"
      ],
      "classification": "READONLY",
      "risk": "LOW",
      "matched_rule": "grep",
      "remaining_options": [
        "-R"
      ],
      "classification_reason": "base classification from rule grep",
      "inner_commands": []
    }
  ]
}
```

Pipeline example:

```bash
printf '%s' 'cat access.log | grep ERROR | tail -20' | rust-bash-classify
```

```json
{
  "expression": "cat access.log | grep ERROR | tail -20",
  "classification": "READONLY",
  "risk": "LOW",
  "directories": [],
  "commands": [
    {
      "command": [
        "cat"
      ],
      "argv": [
        "cat",
        "access.log"
      ],
      "classification": "READONLY",
      "risk": "LOW",
      "matched_rule": "cat",
      "classification_reason": "base classification from rule cat",
      "inner_commands": []
    },
    {
      "command": [
        "grep"
      ],
      "argv": [
        "grep",
        "ERROR"
      ],
      "classification": "READONLY",
      "risk": "LOW",
      "matched_rule": "grep",
      "classification_reason": "base classification from rule grep",
      "inner_commands": []
    },
    {
      "command": [
        "tail"
      ],
      "argv": [
        "tail",
        "-20"
      ],
      "classification": "READONLY",
      "risk": "LOW",
      "matched_rule": "tail",
      "remaining_options": [
        "-20"
      ],
      "classification_reason": "base classification from rule tail",
      "inner_commands": []
    }
  ]
}
```

Delegation example:

```bash
printf '%s' 'sudo sh -c "kubectl get pods -n prod"' | rust-bash-classify
```

```json
{
  "expression": "sudo sh -c \"kubectl get pods -n prod\"",
  "classification": "DANGEROUS",
  "risk": "HIGH",
  "directories": [],
  "commands": [
    {
      "command": [
        "sudo"
      ],
      "argv": [
        "sudo",
        "sh",
        "-c",
        "kubectl get pods -n prod"
      ],
      "classification": "DANGEROUS",
      "risk": "HIGH",
      "matched_rule": "sudo",
      "classification_reason": "elevated by inner command",
      "inner_commands": [
        {
          "delegation_mode": "rest_are_argv",
          "delegation_source": "sudo",
          "command": [
            "sh"
          ],
          "argv": [
            "sh",
            "-c",
            "kubectl get pods -n prod"
          ],
          "classification": "DANGEROUS",
          "risk": "HIGH",
          "matched_rule": "sh",
          "inner_commands": [
            {
              "delegation_mode": "flag_value_is_expression",
              "delegation_source": "-c",
              "command": [
                "kubectl",
                "get"
              ],
              "argv": [
                "kubectl",
                "get",
                "pods",
                "-n",
                "prod"
              ],
              "classification": "READONLY",
              "risk": "LOW",
              "matched_rule": "kubectl.get",
              "inner_commands": []
            }
          ]
        }
      ]
    }
  ]
}
```

### Exit codes

| Code | Meaning |
|---:|---|
| 0 | Successfully classified non-empty input. |
| 1 | Empty input, or no stdin data arrived before the readiness timeout. |
| 2 | Internal error. |

### Help and version

```bash
rust-bash-classify -h
rust-bash-classify --version
```

`-h` and `--help` print usage. `--version` prints the binary name and package version.

## User command database

The binary embeds the vendored built-in command database at compile time. You can add or override command definitions with YAML files in:

```text
~/.config/bash-classify/commands/*.yaml
```

Set `BASH_CLASSIFY_CONFIG_DIR` to use a different config root. The command files are read from `$BASH_CLASSIFY_CONFIG_DIR/commands/*.yaml`. A user file with the same filename stem as a built-in command overrides that built-in rule set.

## Parity evidence

The repository maintains its behavior using committed local evidence:

- 463 Python tests ported to Rust and mapped in `PORT_MANIFEST.md`.
- 175 golden CLI cases replayed byte-for-byte in Rust, including stdout formatting and trailing newlines.
- Local schema tests and generated embed-table drift checks for the 150+ command database.
- Historical comparative benchmark reports (not current gates).

For more detail, see:

- `PORTING.md` for the module map and implementation notes.
- `PARITY.md` for authority rules and accepted differences.
- `AGENTS.md` for contributor guidance and script-owned file rules.

## Development

Development and CI use Rust and committed local inputs. No Python interpreter,
uv, upstream checkout or access to the original repository is required. Cargo
still needs the Rust dependencies recorded in `Cargo.lock` (or a populated cache).

Use Conventional Commit subjects such as `fix(parser): handle empty input`.
Optional local hooks enforce their structure and scan the whole message for
secrets/PII; see `.githooks/README.md` for syntax and setup. Cloning does not
activate them. Missing gitleaks skips scanning with a notice, not commit-format
validation. Automatic merge/revert/fixup titles must be rewritten to conform.
These local checks do not replace CI or a history audit.

Run the standard gates before sending changes for review. Version bumping and
local release commits/tags are explicit Make targets documented in `RELEASING.md`;
neither pushes or publishes automatically. Registry publication is a separate,
explicit `make publish` operation requiring an exported `CARGO_REGISTRY_TOKEN`
and manual account/permission verification; see `RELEASING.md`.

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build
cargo test
scripts/gen_commands_data.sh
git diff --exit-code -- src/commands_data.rs
```

Command YAML and the schema under `vendor/` are maintained locally; changes need
Rust regression and schema tests. Regenerate `src/commands_data.rs` with
`scripts/gen_commands_data.sh`, review and commit intentional table changes before
running the drift check. Do not hand-edit that generated table.

Upstream vendoring, golden harvesting, port-manifest generation and comparative
benchmark scripts are retired. Imported golden cases, the port mapping and
benchmark figures remain historical evidence. `VENDOR.json` records the original
source commit; `vendor/SPEC.md` preserves historical prose, not a live authority.
Preserve licenses and attribution when maintaining local data.

## License

This Rust port is distributed under the MIT License, Copyright (c) 2026 Chrysent LLC.

Portions vendored from [bash-classify](https://github.com/fprochazka/bash-classify) remain Copyright (c) 2025 Filip Prokazka under the upstream MIT license. See `NOTICE`, `vendor/NOTICE`, and `vendor/LICENSE`.
