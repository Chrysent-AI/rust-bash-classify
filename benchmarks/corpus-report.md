<!-- SPDX-FileCopyrightText: 2026 Chrysent LLC -->
<!-- SPDX-License-Identifier: MIT -->

# Large-corpus benchmark

A real-world corpus of **51,882 bash commands** (one JSON-encoded command per
line; the corpus itself is external to this repository and is not included)
was classified end-to-end by both the original Python `bash-classify` and this
Rust implementation. Four measurement modes were used so that process-model
costs and engine costs are reported separately.

## Methodology

- **Library, single process ("amortized")** — one process classifies the whole
  corpus via the public API with the command database loaded once. Measures
  engine throughput without per-invocation costs.
- **CLI, one process per command ("as-spawned")** — the real hook usage
  pattern: a fresh process per command, stdin to stdout. This is how an agent
  permission hook invokes the tool.
- Python side: project virtualenv, `classify_expression` API and the
  `bash-classify` console script. Rust side: release build
  (`cargo build --release`), `examples/bulk_classify` for the library mode and
  the `rust-bash-classify` binary for the CLI mode.
- Python per-command CLI cost was measured on a random 1,000-command sample
  (seeded) and extrapolated; a full per-command Python run would take on the
  order of half an hour on this hardware.
- Timings are from one workstation run; treat them as indicative, not lab-grade.

## Results

| Mode | Python | Rust | Rust vs Python |
|---|---:|---:|---:|
| Library, single process | 2.31 s (44.5 µs/cmd) | 2.22 s (42.8 µs/cmd) | ~1.0× |
| CLI, one process per command | ~27 min (extrapolated; 31.3 ms/cmd sampled) | 38.1 s (734 µs/cmd) | **~43×** |

Reading the numbers honestly:

- **As spawned CLIs — the way permission hooks actually call this tool — Rust
  is ~43× faster.** The Python per-command cost is dominated by interpreter
  startup; the Rust per-command cost is dominated by process spawn itself.
- **Amortized (one long-lived process), the two engines are roughly tied.**
  Both spend nearly all their time in the same tree-sitter bash grammar, which
  is the same C code on both sides; classification logic on top is a small
  fraction. The Python engine was never slow — its process model was.

## Classification overview (aggregate only)

All four measurement modes produced byte-for-byte identical aggregate tallies
across all 51,882 commands, with zero errors and zero unclassifiable lines:

| Classification | Share |
|---|---:|
| READONLY | 64.3% |
| UNKNOWN | 16.5% |
| DANGEROUS | 15.4% |
| LOCAL_EFFECTS | 3.1% |
| EXTERNAL_EFFECTS | 0.6% |

| Risk | Share |
|---|---:|
| LOW | 66.4% |
| HIGH | 31.9% |
| MEDIUM | 1.7% |

The identical tallies are an unplanned but welcome large-scale parity signal:
51,882 real-world commands classified with exactly matching verdict
distributions by both implementations.

## Reproducing

```bash
cargo build --release --example bulk_classify
target/release/examples/bulk_classify /path/to/corpus.jsonl
```

The Rust CLI mode is a plain per-line process loop over the same corpus; the
Python side uses its own API and console script from a checkout of the
original repository at the vendored commit.
