#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Chrysent LLC
# SPDX-License-Identifier: MIT
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
real_git=$(command -v git)
command -v gitleaks >/dev/null
scratch=$(mktemp -d "${TMPDIR:-/tmp}/rust-bash-classify-message-tests.XXXXXX")
cleanup() {
  status=$?
  trap - EXIT INT TERM
  cd "$root"
  rm -rf "$scratch"
  [[ ! -e $scratch ]] || status=1
  echo "Message fixture cleanup verified (status $status)."
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
mkdir "$scratch/repo" "$scratch/bin"
cp "$root/.gitleaks.toml" "$scratch/repo/"
cd "$scratch/repo"
git init -q
git config core.hooksPath "$root/.githooks"
count=0
check() {
  local expected=$1 actual=0
  shift
  cp "$scratch/message" "$scratch/original"
  "$real_git" hook run commit-msg -- "$scratch/message" > "$scratch/result" 2>&1 || actual=$?
  [[ $actual == "$expected" ]] || { echo "Unexpected hook status in case $count: $actual" >&2; exit 1; }
  cmp "$scratch/message" "$scratch/original"
  count=$((count + 1))
}
for message in 'feat: add feature' 'fix: correct behavior' 'docs: explain setup' \
  'custom-type: custom description' 'FIX(core)!: compatibility change' \
  'Build(core/api): build artifacts' 'chore: parentheses (in description)' \
  $'feat: feature\n\nFree-form body\nAnother line\n\nBREAKING CHANGE: explanation' \
  $'fix(scope): repair\r\n\r\nBody\r\n' $'docs: text\n'; do
  printf '%s' "$message" > "$scratch/message"
  check 0
done
for message in '' 'plain title' 'Merge branch topic' 'Revert change' 'fixup! fix: change' \
  'fix:' 'fix: ' 'fix:    ' 'fix:missing space' 'fix(): description' 'fix( ): description' \
  'fix((scope)): description' ': description' 'fix!: ' $'fix: good\nBody without separator' \
  $'fix: good\r\nBody without separator\r\n'; do
  printf '%s' "$message" > "$scratch/message"
  check 1
  grep -q 'commit-msg: use type' "$scratch/result"
done
# Construct synthetic samples so repository scans do not find fixture literals.
token="ghp_"
token+="7kR9mQ2vL8nX4pT6wY3bC5dF1hJ0sA9eU2zW"
pii="message-fixture@"
pii+="gmail.com"
for sample in "$token" "$pii"; do
  for placement in title body; do
    if [[ $placement == title ]]; then
      printf 'fix: %s\n' "$sample" > "$scratch/message"
    else
      printf 'fix: clean title\n\n%s\n' "$sample" > "$scratch/message"
    fi
    check 1
    if [[ $sample == "$token" ]]; then rule=github-pat; else rule=consumer-email; fi
    grep -q "$rule" "$scratch/result"
    if grep -Fq "$sample" "$scratch/result"; then echo 'Unredacted finding' >&2; exit 1; fi
  done
done
# A scanner failure is not mistaken for a clean result.
printf '#!/bin/sh\nexit 42\n' > "$scratch/bin/gitleaks"
chmod +x "$scratch/bin/gitleaks"
printf 'fix: clean title\n' > "$scratch/message"
PATH="$scratch/bin:$PATH" check 42
# Minimal PATH deliberately excludes gitleaks, but keeps the actual hook tools.
unlink "$scratch/bin/gitleaks"
for tool in git awk; do ln -s "$(command -v "$tool")" "$scratch/bin/$tool"; done
# check() also uses these utilities to assert immutability.
for tool in cp cmp; do ln -s "$(command -v "$tool")" "$scratch/bin/$tool"; done
PATH="$scratch/bin" check 0
grep -q 'skipping commit-message scan' "$scratch/result"
printf 'plain title\n' > "$scratch/message"
PATH="$scratch/bin" check 1
grep -q 'commit-msg: use type' "$scratch/result"
if "$real_git" hook run commit-msg > "$scratch/result" 2>&1; then exit 1; fi
grep -q 'no commit message file argument' "$scratch/result"
[[ -z $(git status --porcelain --untracked-files=no) ]]
printf 'PASS: %s effective commit-msg cases plus missing argument.\n' "$count"
