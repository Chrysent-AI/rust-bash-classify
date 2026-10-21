<!-- SPDX-FileCopyrightText: 2026 Chrysent LLC -->
<!-- SPDX-License-Identifier: MIT -->
# Historical golden CLI fixtures

These 175 cases were captured from the original Python test suite and CLI at
commit `8553440aa4d935dd14a8cbb15338ecac92100720`. `manifest.json` retains case
provenance, content hashes, expected exit codes and the original source commit.
Obsolete Python dependency and generator metadata is not maintained.

`cargo test --test golden_replay` replays committed inputs through the Rust binary
and checks expected stdout bytes and exit codes without Python or another checkout.
`expected.json` files include exact formatting and trailing newlines. Preserve
this evidence; never edit expectations merely to hide a regression. Intentional
behavior changes need reviewed expectations and focused Rust regression tests.

Future maintenance uses local Rust tests, not upstream harvesting or live
comparisons. Manifest metadata may be maintained locally.
