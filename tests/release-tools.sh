#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Chrysent LLC
# SPDX-License-Identifier: MIT
# Runs only in owned scratch repositories; never commits/tags the source checkout.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
real_git=$(command -v git)
command -v gitleaks >/dev/null
export REAL_GIT="$real_git"
# Reuse configured identity without writing Git identity/configuration or printing it.
export GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL
GIT_AUTHOR_NAME=$(git -C "$root" config user.name)
GIT_AUTHOR_EMAIL=$(git -C "$root" config user.email)
GIT_COMMITTER_NAME=$GIT_AUTHOR_NAME
GIT_COMMITTER_EMAIL=$GIT_AUTHOR_EMAIL
scratch=$(mktemp -d "${TMPDIR:-/tmp}/rust-bash-classify-release-tests.XXXXXX")
cleanup() {
  status=$?
  trap - EXIT INT TERM
  cd "$root"
  rm -rf "$scratch"
  [[ ! -e $scratch ]] || status=1
  echo "Release fixture cleanup verified (status $status)."
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
export TMPDIR="$scratch/tmp"
mkdir -p "$TMPDIR" "$scratch/base/scripts" "$scratch/base/src" "$scratch/bin"
cp "$root/scripts/version.sh" "$scratch/base/scripts/"
cp "$root/Makefile" "$scratch/base/"
printf '# Changelog\n\n## [Unreleased]\n\n## [0.1.0] - Unreleased\n\n- Initial fixture release.\n' > "$scratch/base/CHANGELOG.md"
cd "$scratch/base"
printf '[package]\nname = "rust-bash-classify"\nversion = "0.1.0"\nedition = "2021"\n' > Cargo.toml
printf 'fn main() {}\n' > src/main.rs
cargo generate-lockfile --offline
printf 'unrelated\n' > other.txt
git init -q
git add -- Cargo.toml Cargo.lock CHANGELOG.md Makefile scripts/version.sh src/main.rs other.txt
git commit -qm 'Initial fixture'
base_head=$(git rev-parse HEAD)
count=0
fixture() {
  count=$((count + 1))
  git clone -q --local "$scratch/base" "$scratch/case-$count"
  cd "$scratch/case-$count"
  mkdir hooks
  cp "$root/.githooks/commit-msg" hooks/
  cp "$root/.gitleaks.toml" .
  git config core.hooksPath hooks
}
assert_version() {
  [[ $(make -s version) == "$1" ]]
  grep -Fx "version = \"$1\"" Cargo.lock
  grep -Eq "^## \[$1\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$" CHANGELOG.md
}
snapshot() {
  mkdir -p "$scratch/before"
  cp Cargo.toml Cargo.lock CHANGELOG.md "$scratch/before/"
  cp .git/index "$scratch/before/index"
  git rev-parse HEAD > "$scratch/before/head"
  git show-ref --tags > "$scratch/before/tags" || :
}
unchanged() {
  for file in Cargo.toml Cargo.lock CHANGELOG.md; do cmp "$file" "$scratch/before/$file"; done
  cmp .git/index "$scratch/before/index"
  [[ $(git rev-parse HEAD) == "$(< "$scratch/before/head")" ]]
  git show-ref --tags > "$scratch/after-tags" || :
  cmp "$scratch/after-tags" "$scratch/before/tags"
}
reject() {
  snapshot
  if "$@" > "$scratch/rejected.log" 2>&1; then echo 'Expected rejection' >&2; exit 1; fi
  unchanged
}
for bump in major minor patch; do
  fixture
  make bump-version BUMP="$bump"
  case "$bump" in major) expected=1.0.0;; minor) expected=0.2.0;; patch) expected=0.1.1;; esac
  assert_version "$expected"
  [[ $(git rev-parse HEAD) == "$base_head" && -z $(git tag -l) ]]
  git diff --cached --exit-code
done
fixture
make bump-version VERSION=2.3.4
assert_version 2.3.4
# Subsequent bump moves Unreleased notes under the next dated heading.
printf '\n' >> CHANGELOG.md
make bump-version BUMP=patch
assert_version 2.3.5
grep -F '## [2.3.4] - ' CHANGELOG.md
fixture
make release VERSION=0.1.0
assert_version 0.1.0
[[ $(git log -1 --format=%s) == 'chore(release): release v0.1.0' ]]
[[ $(git cat-file -t refs/tags/v0.1.0) == tag ]]
[[ $(git rev-list --count HEAD) == 2 ]]
[[ $(git diff-tree --no-commit-id --name-only -r HEAD) == CHANGELOG.md ]]
git diff --exit-code && git diff --cached --exit-code
# No helper may contact the remote (a local bare fixture makes pushes observable).
git init -q --bare "$scratch/remote.git"
git remote set-url origin "$scratch/remote.git"
make release BUMP=patch
[[ -z $(git --git-dir="$scratch/remote.git" for-each-ref) ]]
# Exercise the system Bash used by native macOS (3.2), not only a newer PATH Bash.
fixture
mkdir "$scratch/system-bin"
ln -s /bin/bash "$scratch/system-bin/bash"
PATH="$scratch/system-bin:$PATH" make release VERSION=0.1.0
assert_version 0.1.0
[[ $(git log -1 --format=%s) == 'chore(release): release v0.1.0' ]]
[[ $(git cat-file -t refs/tags/v0.1.0) == tag ]]
fixture
for version in 01.2.3 v1.2.3 1.2 1.2.3-beta 1.2.3+build 0.0.9 '1.2.3;false'; do
  reject make release VERSION="$version"
done
reject make bump-version BUMP=invalid
reject make release BUMP=patch VERSION=1.2.3
reject bash scripts/version.sh release unexpected
fixture
printf 'dirty\n' >> other.txt
reject make release
fixture
printf 'staged\n' >> other.txt
git add other.txt
# Also reject staged changes hidden by a worktree restored to HEAD.
git show HEAD:other.txt > other.txt
reject make release
fixture
git checkout -q --detach
reject make bump-version
fixture
git tag v0.1.1
reject make release
fixture
# Preserve distinct user bytes in the index and working trio on early failure.
printf '\nstaged notes\n' >> CHANGELOG.md
git add CHANGELOG.md
printf '\nunstaged notes\n' >> CHANGELOG.md
cat > "$scratch/bin/cargo" <<'STUB'
#!/usr/bin/env bash
exit 42
STUB
chmod +x "$scratch/bin/cargo"
reject env PATH="$scratch/bin:$PATH" make release
unlink "$scratch/bin/cargo"
fixture
printf '#!/usr/bin/env bash\nexit 1\n' > hooks/pre-commit
chmod +x hooks/pre-commit
printf '\nstaged notes\n' >> CHANGELOG.md
git add CHANGELOG.md
printf '\nunstaged notes\n' >> CHANGELOG.md
reject make release
fixture
# Inject the former malformed subject at Git argv, exercising the real hook
# and the helper rollback without changing the production script in the fixture.
cat > "$scratch/bin/git" <<'STUB'
#!/usr/bin/env bash
if [[ $1 == commit ]]; then
  exec "$REAL_GIT" commit -m 'Release without conventional type'
fi
exec "$REAL_GIT" "$@"
STUB
chmod +x "$scratch/bin/git"
reject env PATH="$scratch/bin:$PATH" make release
grep -q 'commit-msg: use type' "$scratch/rejected.log"
fixture
cat > "$scratch/bin/git" <<'STUB'
#!/usr/bin/env bash
if [[ $1 == tag && ${2:-} == -a ]]; then exit 43; fi
exec "$REAL_GIT" "$@"
STUB
chmod +x "$scratch/bin/git"
# The tag failure happens only after the real message hook accepts the commit.
if PATH="$scratch/bin:$PATH" make release > "$scratch/tag-failure.log" 2>&1; then exit 1; fi
[[ $(git rev-list --count HEAD) == 2 && -z $(git tag -l) ]]
assert_version 0.1.1
git diff --exit-code && git diff --cached --exit-code
grep -F 'Commit created but tag failed' "$scratch/tag-failure.log"
grep -F 'Release commit exists' "$scratch/tag-failure.log"
fixture
# Validate the exact check seam used by release CI.
reject bash scripts/version.sh check v0.1.0
make bump-version VERSION=0.1.0
bash scripts/version.sh check v0.1.0
reject bash scripts/version.sh check v0.2.0
printf '# Changelog\n\n## [0x1x0] - 2026-01-01\n' > CHANGELOG.md
reject bash scripts/version.sh check v0.1.0
# Archives use the actual production packager, with a tiny executable stub.
cd "$root"
printf '#!/usr/bin/env bash\nprintf "archive smoke\\n"\n' > "$scratch/stub"
chmod +x "$scratch/stub"
bash scripts/package-release.sh aarch64-apple-darwin "$scratch/stub" "$scratch/dist"
cd "$scratch/dist"
shasum -a 256 -c rust-bash-classify-aarch64-apple-darwin.tar.gz.sha256
mkdir "$scratch/extracted"
tar -xzf rust-bash-classify-aarch64-apple-darwin.tar.gz -C "$scratch/extracted"
[[ $("$scratch/extracted/rust-bash-classify") == 'archive smoke' ]]
for file in LICENSE NOTICE README.md vendor/LICENSE vendor/NOTICE; do cmp "$root/$file" "$scratch/extracted/$file"; done
[[ $(tar -tzf rust-bash-classify-aarch64-apple-darwin.tar.gz | wc -l | tr -d ' ') == 6 ]]
[[ -z $(find "$TMPDIR" -mindepth 1 -print) ]]
printf 'PASS: %s isolated version/release fixtures plus archive/checksum smoke.\n' "$count"
