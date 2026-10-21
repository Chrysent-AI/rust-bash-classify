<!-- SPDX-FileCopyrightText: 2026 Chrysent LLC -->
<!-- SPDX-License-Identifier: MIT -->
# Versioning and releases

The crate and binary remain `rust-bash-classify`. The initial `0.1.0` is still
unreleased. Stable releases use `vX.Y.Z` tags. Publication is a human operation;
preparing this tooling does not establish crates.io ownership or publish anything.

## Prepare locally

Use Bash, Git, make and stable Cargo, with the locked dependencies already cached
(the helper validates the lockfile offline). Work on a branch, not detached HEAD.
Commit the release tooling before using it. Do not edit the working tree or index
concurrently with a helper.

```bash
make version
make bump-version BUMP=patch       # also minor or major; patch is the default
make bump-version VERSION=0.2.0    # exact stable version instead of BUMP
```

Bump-only changes `Cargo.toml`, the root package entry of `Cargo.lock`, and
`CHANGELOG.md`, without staging, committing, tagging, pushing or publishing.
Dependency versions do not change. Choose **either** `BUMP` or `VERSION`.
Versions must not decrease; components are decimal integers from 0 to 999999999,
without leading zeros. Prerelease and build suffixes are not supported.

Keep upcoming notes under `## [Unreleased]`. A bump moves those notes under a
new dated version heading. For the initial placeholder `## [0.1.0] - Unreleased`,
it replaces that heading with the chosen version and today's UTC date, retaining
the initial notes. No release date is recorded until a maintainer runs the helper.

To make a local release commit and annotated tag in one step:

```bash
make release VERSION=0.1.0         # initial release, without incrementing 0.1.0
# Later releases, choose one:
make release BUMP=patch
make release VERSION=0.2.0
```

After a separate bump-only step, use `make release VERSION=<prepared-version>`
to commit that same version, rather than bumping it twice. If the version and
dated changelog are already committed and there are no changes, the helper refuses
to make an empty commit; inspect the commit and create its annotated tag manually.

The helpers reject invalid/dual arguments, existing target tags, detached HEAD,
conflicts and unrelated tracked or staged changes **before** editing. Only the
version trio may contain pending edits, including staged edits; review those edits
because release commits their final working-tree contents. Untracked unrelated
files are never staged. Failures before a commit restore the trio and exact index
snapshot. Hooks run normally. The generated subject is
`chore(release): release vX.Y.Z`, compatible with the optional Conventional Commit
hook described in `.githooks/README.md`; that hook still scans the entire message
for secrets/PII. Missing gitleaks skips scanning, not format validation.
If a commit succeeds but tagging fails, the commit
is retained and the error tells you to inspect HEAD and finish the tag manually;
there is no destructive rollback. Neither helper pushes or publishes.

## Verify and push deliberately

Run the gates in `AGENTS.md`, plus:

```bash
bash tests/release-tools.sh
bash tests/commit-msg.sh
shellcheck .githooks/commit-msg tests/commit-msg.sh scripts/version.sh scripts/package-release.sh tests/release-tools.sh tests/package-release.sh
actionlint
cargo package --locked
cargo build --release --locked
bash tests/package-release.sh
```

The helper tests create and reap their own scratch repositories using your existing
configured Git identity, without changing identity configuration. They exercise
failure recovery without touching this checkout's HEAD, index or tags.

Review the release commit/tag and dated changelog. When ready, push the branch and
the **specific** release tag yourself. Normal push/PR CI remains enabled. Only
pushed stable `vX.Y.Z` tags start binary builds after manifest/changelog validation;
other tags do not build or upload binaries. All six matrix builds must succeed
before the release job creates a draft, uploads the assets and publishes it.
If upload/publication fails, inspect any draft and assets before retrying; the
workflow intentionally does not overwrite an existing release automatically.

| Target | Hosted runner |
| --- | --- |
| `x86_64-unknown-linux-gnu` | `ubuntu-24.04` |
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` |
| `x86_64-unknown-linux-musl` | `ubuntu-24.04` |
| `aarch64-unknown-linux-musl` | `ubuntu-24.04-arm` |
| `x86_64-apple-darwin` | `macos-15-intel` |
| `aarch64-apple-darwin` | `macos-15` |

Linux GNU binaries use the runner's glibc; they are not promised to work on older
glibc systems. Musl builds explicitly install Ubuntu `musl-tools`/`musl-dev` and
configure the architecture-native musl GCC wrapper for both the tree-sitter C
build and Rust linker. Installing a Rust target alone is not enough. The workflow
smokes each native binary. Hosted six-platform execution must still be observed
before claiming a successful release; local preparation is not that evidence.

## Archive and source-package contract

At `https://github.com/Chrysent-AI/rust-bash-classify/releases/download/vX.Y.Z/`,
each target has `rust-bash-classify-<target>.tar.gz` and a matching `.tar.gz.sha256`.
The archive contains exactly:

```text
rust-bash-classify
LICENSE
NOTICE
README.md
vendor/LICENSE
vendor/NOTICE
```

The binary is at archive root. The manifest's `pkg-url`, `bin-dir` and `tgz`
format match this layout. SHA256 sidecars detect corruption; they are not
cryptographic publisher signatures. Verify with `sha256sum -c <archive>.sha256`
or `shasum -a 256 -c <archive>.sha256` from the download directory.

For local archive checks, build with `cargo build --release --locked`, then run
`bash scripts/package-release.sh <target> target/release/rust-bash-classify <output-dir>`.
Use the actual native target, extract into a temporary directory and run the binary.
The packager does not infer a binary's architecture; the build matrix supplies it.

Cargo uses an explicit source-package allowlist: Rust source, local command YAML,
schema, manifest/lockfile, licenses/notices and installation/release documentation.
Environment files (including the tracked local loader and any variants), hooks,
workflow scripts, test fixtures and local orchestration artifacts are not included.
Inspect `cargo package --list --locked` before publication. From a dirty preparation
checkout only, `--allow-dirty` permits local package verification without staging.
It is not part of the publication procedure.

## Manual crates.io publication

After the GitHub assets are published and verified, use a clean checkout of the
exact release tag. Verify the crates.io package name and ownership under the
intended authenticated **chrysent** account. `authors = ["Chrysent LLC"]` is
informational attribution, not registry ownership or proof of account access.
Account setup and ownership verification remain manual prerequisites.

Then, with explicit authorization, publish locally:

```bash
cargo publish --locked
```

The GitHub workflow uses its GitHub token only for repository release operations;
it never publishes to a Cargo registry. No registry credentials belong in this
workflow or these helpers. The local helpers do not source environment loaders,
inspect credentials or log in. Keep environment files out of packages and commits.

After both the registry version and official assets exist, consumers with
cargo-binstall can use:

```bash
cargo binstall rust-bash-classify
```

For an official-assets-only check, use
`cargo binstall --strategies crate-meta-data rust-bash-classify` so fallback
compilation or third-party binaries cannot mask a missing asset. This is a future
networked installation step, not a claim that 0.1.0 is published today.
