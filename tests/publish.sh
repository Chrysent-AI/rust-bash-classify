#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Chrysent LLC
# SPDX-License-Identifier: MIT
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
make_bin=$(command -v make)
scratch=$(mktemp -d /tmp/rust-bash-classify-publish.XXXXXX)
completed=false
cleanup() {
    status=$?
    trap - EXIT INT TERM
    # Bash 3.2 can reach EXIT with status zero after a nounset expansion error.
    if [[ $completed != true && $status == 0 ]]; then status=1; fi
    cd "$root"
    rm -rf "$scratch"
    test ! -e "$scratch" || status=1
    echo "Publish fixture cleanup verified (status $status)."
    exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
mkdir "$scratch/bin"
cat > "$scratch/bin/cargo" <<'STUB'
#!/bin/sh
set -eu
# Record invocation without recording any environment or argument values.
touch invoked
test "$#" -eq 4
test "$1" = publish
test "$2" = --registry
test "$3" = crates-io
test "$4" = --locked
test "${CARGO_REGISTRY_TOKEN:-}" = synthetic-publish-test-value
test "${CARGO_REGISTRY_CREDENTIAL_PROVIDER:-}" = cargo:token
touch validated
exit "${STUB_STATUS:-0}"
STUB
chmod +x "$scratch/bin/cargo"
cd "$scratch"

run_make() {
    # A fresh environment prevents real credentials and caller shell configuration
    # from entering the fixture. Cargo can only resolve to the owned stub.
    env -i PATH="$scratch/bin:/usr/bin:/bin" "$@" \
        "$make_bin" -f "$root/Makefile" publish > output 2>&1
}
check_no_token_output() {
    if grep -Fq synthetic-publish-test-value output; then
        echo 'publish exposed the synthetic token' >&2
        exit 1
    fi
}
for state in unset empty; do
    set --
    if [[ $state == empty ]]; then set -- CARGO_REGISTRY_TOKEN=; fi
    if run_make "$@"; then
        echo "publish accepted $state token" >&2
        exit 1
    fi
    test ! -e invoked
    grep -q 'CARGO_REGISTRY_TOKEN must be exported and nonempty' output
    check_no_token_output
done
for provider in default overridden; do
    args=(CARGO_REGISTRY_TOKEN=synthetic-publish-test-value)
    if [[ $provider == overridden ]]; then
        args+=(CARGO_REGISTRY_CREDENTIAL_PROVIDER=unwanted-provider)
    fi
    run_make "${args[@]}"
    test -e validated
    check_no_token_output
    rm invoked validated
done
if run_make CARGO_REGISTRY_TOKEN=synthetic-publish-test-value STUB_STATUS=42; then
    echo 'publish hid Cargo failure' >&2
    exit 1
fi
test -e validated
check_no_token_output
printf '%s\n' 'PASS: publish tests (stub Cargo only)'
completed=true
