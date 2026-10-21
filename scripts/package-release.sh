#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Chrysent LLC
# SPDX-License-Identifier: MIT
set -euo pipefail
[[ $# == 3 ]] || { echo "Usage: $0 TARGET BINARY OUTPUT_DIRECTORY" >&2; exit 1; }
target=$1
case "$target" in
  x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|x86_64-unknown-linux-musl|aarch64-unknown-linux-musl|x86_64-apple-darwin|aarch64-apple-darwin) ;;
  *) echo "Unsupported release target: $target" >&2; exit 1 ;;
esac
[[ -x $2 ]] || { echo 'Binary must be executable' >&2; exit 1; }
binary=$(cd "$(dirname "$2")" && pwd)/$(basename "$2")
mkdir -p "$3"
out=$(cd "$3" && pwd)
cd "$(dirname "$0")/.."
stage=$(mktemp -d "${TMPDIR:-/tmp}/rust-bash-classify-archive.XXXXXX")
cleanup() {
  status=$?
  trap - EXIT INT TERM
  rm -rf "$stage"
  [[ ! -e $stage ]] || status=1
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
mkdir "$stage/vendor"
cp "$binary" "$stage/rust-bash-classify"
cp LICENSE NOTICE README.md "$stage/"
cp vendor/LICENSE vendor/NOTICE "$stage/vendor/"
archive="rust-bash-classify-$target.tar.gz"
# Avoid local user names and macOS metadata in distributed archives.
owner=(--owner=0 --group=0)
if tar --version | grep -q bsdtar; then owner=(--uid 0 --gid 0 --uname root --gname root); fi
COPYFILE_DISABLE=1 tar --format=ustar "${owner[@]}" -czf "$out/$archive" -C "$stage" rust-bash-classify LICENSE NOTICE README.md vendor/LICENSE vendor/NOTICE
cd "$out"
if command -v sha256sum >/dev/null; then
  sha256sum "$archive" > "$archive.sha256"
else
  shasum -a 256 "$archive" > "$archive.sha256"
fi
printf '%s\n' "$archive" "$archive.sha256"
