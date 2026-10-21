#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Chrysent LLC
# SPDX-License-Identifier: MIT
# Source payload and native archive checks; build the native release binary first.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/rust-bash-classify-package-tests.XXXXXX")
cleanup() {
  status=$?
  trap - EXIT INT TERM
  cd "$root"
  rm -rf "$scratch"
  [[ ! -e $scratch ]] || status=1
  echo "Package fixture cleanup verified (status $status)."
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
mkdir -p "$scratch/source/vendor" "$scratch/source/.githooks" "$scratch/source/sdlc" "$scratch/config" "$scratch/extracted"
# Explicit inputs only: never copy actual environment files into fixtures.
cp "$root/Cargo.toml" "$root/Cargo.lock" "$root/LICENSE" "$root/NOTICE" "$root/README.md" "$root/RELEASING.md" "$root/CHANGELOG.md" "$scratch/source/"
cp -R "$root/src" "$scratch/source/"
cp -R "$root/vendor/commands" "$root/vendor/schemas" "$scratch/source/vendor/"
cp "$root/vendor/LICENSE" "$root/vendor/NOTICE" "$scratch/source/vendor/"
cd "$scratch/source"
# Empty synthetic environment variants, including names under allowed globs.
touch .env .envrc .env.local .envrc.local src/.env.rs vendor/commands/.env.yaml .githooks/pre-commit sdlc/private.md
cargo package --list --locked --offline --allow-dirty > "$scratch/payload"
while IFS= read -r file; do
  case "$file" in
    */.env*|.env*|.githooks/*|sdlc/*) echo "Unexpected package file: $file" >&2; exit 1 ;;
    Cargo.toml|Cargo.toml.orig|Cargo.lock|LICENSE|NOTICE|README.md|CHANGELOG.md|RELEASING.md|src/*.rs|vendor/commands/*.yaml|vendor/schemas/command.schema.json|vendor/LICENSE|vendor/NOTICE) ;;
    *) echo "Unexpected package file: $file" >&2; exit 1 ;;
  esac
done < "$scratch/payload"
for file in vendor/commands/*.yaml vendor/schemas/command.schema.json LICENSE NOTICE vendor/LICENSE vendor/NOTICE; do
  grep -Fx "$file" "$scratch/payload" > /dev/null
done
printf '%s\n' 'PASS: source allowlist, environment variants, local YAML/schema and licenses.'
# Prove the real dependency graph stays byte-identical except for the root version.
mkdir scripts
cp "$root/scripts/version.sh" scripts/
export GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL
GIT_AUTHOR_NAME=$(git -C "$root" config user.name)
GIT_AUTHOR_EMAIL=$(git -C "$root" config user.email)
GIT_COMMITTER_NAME=$GIT_AUTHOR_NAME
GIT_COMMITTER_EMAIL=$GIT_AUTHOR_EMAIL
git init -q
git add -- Cargo.toml Cargo.lock CHANGELOG.md src/*.rs src/bin/*.rs vendor/commands/*.yaml vendor/schemas/*.json vendor/LICENSE vendor/NOTICE LICENSE NOTICE README.md RELEASING.md scripts/version.sh
git commit -qm 'Package fixture'
current=$(bash "$root/scripts/version.sh" version)
bash scripts/version.sh bump
awk -v current="$current" '/^name = "rust-bash-classify"$/ {root=1} root && /^version = / {$0="version = \"" current "\""; root=0} {print}' Cargo.lock > "$scratch/normalized.lock"
cmp "$root/Cargo.lock" "$scratch/normalized.lock"
printf '%s\n' 'PASS: real dependency lock entries preserved during bump.'
cd "$root"
target=$(rustc -vV | awk '/^host:/ {print $2}')
bash scripts/package-release.sh "$target" target/release/rust-bash-classify "$scratch/dist"
cd "$scratch/dist"
shasum -a 256 -c "rust-bash-classify-$target.tar.gz.sha256"
tar -xzf "rust-bash-classify-$target.tar.gz" -C "$scratch/extracted"
[[ $(tar -tzf "rust-bash-classify-$target.tar.gz" | wc -l | tr -d ' ') == 6 ]]
export BASH_CLASSIFY_CONFIG_DIR="$scratch/config"
"$scratch/extracted/rust-bash-classify" --version
printf '%s' 'git status' | "$scratch/extracted/rust-bash-classify" > "$scratch/smoke.json"
grep -F '"classification": "READONLY"' "$scratch/smoke.json"
for file in LICENSE NOTICE README.md vendor/LICENSE vendor/NOTICE; do cmp "$root/$file" "$scratch/extracted/$file"; done
# Literal metadata contract verified against cargo-binstall 1.6.3 SUPPORT.md.
grep -Fx 'pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ target }.tar.gz"' "$root/Cargo.toml"
grep -Fx 'bin-dir = "{ bin }{ binary-ext }"' "$root/Cargo.toml"
grep -Fx 'pkg-fmt = "tgz"' "$root/Cargo.toml"
printf 'PASS: native %s archive, checksum, root binary and classification.\n' "$target"
