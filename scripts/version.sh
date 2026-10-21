#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Chrysent LLC
# SPDX-License-Identifier: MIT
set -euo pipefail
cd "$(dirname "$0")/.."
fail() { printf '%s\n' "$*" >&2; exit 1; }
# Keep integer arithmetic portable and reject prerelease/build/leading-zero forms.
stable='^(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})$'
current=$(awk '/^\[package\]$/ {p=1; next} /^\[/ {p=0} p && /^version = / {gsub(/"/, "", $3); print $3}' Cargo.toml)
[[ $current =~ $stable ]] || fail "Invalid package version"
mode=${1:-version}
case "$mode" in
  version) [[ $# -le 1 ]] || fail "Usage: $0 version"; printf '%s\n' "$current"; exit ;;
  check)
    [[ $# == 2 && $2 == "v$current" ]] || fail "Tag must match v$current"
    escaped=${current//./\\.}
    grep -Eq "^## \[$escaped\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$" CHANGELOG.md || fail "Missing dated changelog entry"
    exit ;;
  bump|release) [[ $# == 1 ]] || fail "Use BUMP=major|minor|patch or VERSION=x.y.z" ;;
  *) fail "Usage: $0 version|bump|release|check TAG" ;;
esac
[[ -z ${BUMP:-} || -z ${VERSION:-} ]] || fail "Choose BUMP or VERSION, not both"
if [[ -n ${VERSION:-} ]]; then
  next=$VERSION
else
  IFS=. read -r major minor patch <<< "$current"
  case ${BUMP:-patch} in
    major) next="$((major + 1)).0.0" ;;
    minor) next="$major.$((minor + 1)).0" ;;
    patch) next="$major.$minor.$((patch + 1))" ;;
    *) fail "BUMP must be major, minor or patch" ;;
  esac
fi
[[ $next =~ $stable ]] || fail "VERSION must be stable x.y.z (components 0..999999999)"
IFS=. read -r a b c <<< "$current"
IFS=. read -r x y z <<< "$next"
(( x > a || (x == a && y > b) || (x == a && y == b && z >= c) )) || fail "Version must not decrease"
[[ -z ${GIT_INDEX_FILE:-} ]] || fail "Alternate Git index is not supported"
git symbolic-ref -q HEAD >/dev/null || fail "Detached HEAD; switch to a branch first"
head=$(git rev-parse HEAD)
git show-ref --verify --quiet "refs/tags/v$next" && fail "Tag v$next already exists"
files=(Cargo.toml Cargo.lock CHANGELOG.md)
for file in "${files[@]}"; do
  [[ -f $file && ! -L $file ]] || fail "Expected regular file: $file"
  git ls-files --error-unmatch "$file" >/dev/null || fail "Commit $file before using this helper"
done
for state in worktree index; do
  args=(); [[ $state != index ]] || args=(--cached)
  git diff "${args[@]}" --quiet -- . ':!Cargo.toml' ':!Cargo.lock' ':!CHANGELOG.md' || fail "Unrelated tracked or staged changes; commit or stash them first"
done
[[ -z $(git ls-files -u) ]] || fail "Resolve merge conflicts first"
index=$(git rev-parse --git-path index)
[[ -f $index ]] || fail "Missing Git index"
backup=$(mktemp -d "${TMPDIR:-/tmp}/rust-bash-classify-version.XXXXXX")
finished=false
snapshotted=false
cleanup() {
  status=$?
  trap - EXIT INT TERM
  if [[ $finished == false && $snapshotted == true ]]; then
    if [[ $(git rev-parse HEAD) == "$head" ]]; then
      for file in "${files[@]}"; do cp -p "$backup/$file" "$file"; done
      cp -p "$backup/index" "$index"
      printf '%s\n' 'No commit created; restored version files and index.' >&2
    else
      printf '%s\n' "Release commit exists; leaving files/index intact. Inspect HEAD and tag v$next before continuing; nothing was pushed." >&2
    fi
  fi
  rm -rf "$backup"
  [[ ! -e $backup ]] || { printf '%s\n' 'Temporary backup cleanup failed' >&2; status=1; }
  exit "$status"
}
# A failed snapshot only removes the incomplete backup, never restores it.
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
for file in "${files[@]}"; do cp -p "$file" "$backup/$file"; done
cp -p "$index" "$backup/index"
snapshotted=true
awk -v v="$next" '
  /^\[package\]$/ {p=1}
  /^\[/ && $0 != "[package]" {p=0}
  p && /^version = / {$0="version = \"" v "\""; n++}
  {print}
  END {if(n!=1) exit 1}
' Cargo.toml > "$backup/manifest.new"
awk -v v="$next" '
  /^\[\[package\]\]$/ {p=0}
  /^name = "rust-bash-classify"$/ {p=1}
  p && /^version = / {$0="version = \"" v "\""; n++}
  {print}
  END {if(n!=1) exit 1}
' Cargo.lock > "$backup/lock.new"
cp "$backup/manifest.new" Cargo.toml
cp "$backup/lock.new" Cargo.lock
# Validate the surgically refreshed root entry without re-resolving dependency versions.
cargo metadata --offline --locked --format-version 1 > /dev/null
awk -v old="$current" -v new="$next" -v date="$(date -u +%F)" '
  NR==FNR {if($0=="## [" old "] - Unreleased") initial=1; next}
  $0=="## [" old "] - Unreleased" {$0="## [" new "] - " date; n++}
  !initial && old!=new && $0=="## [Unreleased]" {print; $0="\n## [" new "] - " date; n++}
  {print}
  END {if(n!=1 && old!=new) exit 1}
' CHANGELOG.md CHANGELOG.md > "$backup/changelog.new"
cp "$backup/changelog.new" CHANGELOG.md
bash scripts/version.sh check "v$next"
if [[ $mode == release ]]; then
  git add -- "${files[@]}"
  git diff --cached --quiet && fail "No release changes to commit; use an unreleased entry or a new version"
  printf 'Release v%s\n' "$next" > "$backup/message"
  git commit -F "$backup/message"
  # A failed tag must never roll back a successful commit (including hook effects).
  git tag -a "v$next" -F "$backup/message" || fail "Commit created but tag failed; inspect HEAD and create the tag manually after resolving the failure"
  printf 'Created local release commit and annotated tag v%s; nothing pushed or published.\n' "$next"
else
  printf 'Prepared %s; no commit or tag created.\n' "$next"
fi
finished=true
