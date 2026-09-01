#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Chrysent LLC
# SPDX-License-Identifier: MIT
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
source_dir=${BASH_CLASSIFY_SOURCE_DIR:-/home/chrysent/devel/bash-classify}
mode=sync

usage() {
  cat <<'USAGE'
Usage: scripts/vendor.sh [--check]

Syncs vendored command definitions, schemas, and SPEC.md from the Python
bash-classify repository. Set BASH_CLASSIFY_SOURCE_DIR to override the source
path (default: /home/chrysent/devel/bash-classify).

Policy: the Python repository remains the source of truth. Vendored files must
not be edited by hand. After any sync that changes behavior, regenerate golden
outputs before merging later parity milestones.

--check verifies that vendor/ matches the recorded source commit in
VENDOR.json, that the source checkout is clean, and that its HEAD is the
recorded commit. It never modifies the source repo.
USAGE
}

if [[ ${1:-} == "--help" || ${1:-} == "-h" ]]; then
  usage
  exit 0
elif [[ ${1:-} == "--check" ]]; then
  mode=check
elif [[ $# -gt 0 ]]; then
  usage >&2
  exit 2
fi

commands_src="$source_dir/src/bash_classify/commands"
schemas_src="$source_dir/schemas"
spec_src="$source_dir/SPEC.md"
license_src="$source_dir/LICENSE"

for required in "$commands_src" "$schemas_src" "$spec_src" "$license_src"; do
  if [[ ! -e "$required" ]]; then
    echo "missing source path: $required" >&2
    exit 1
  fi
done

source_commit=$(git -C "$source_dir" rev-parse HEAD)
if [[ -n $(git -C "$source_dir" status --porcelain) ]]; then
  echo "source working tree is dirty: $source_dir" >&2
  exit 1
fi
py_versions=$(python3 - "$source_dir/uv.lock" <<'PY'
import sys
from pathlib import Path
lock = Path(sys.argv[1]).read_text()
versions = {}
current = None
for line in lock.splitlines():
    if line.startswith('name = '):
        current = line.split('"', 2)[1]
    elif current in {'tree-sitter', 'tree-sitter-bash'} and line.startswith('version = '):
        versions[current] = line.split('"', 2)[1]
        current = None
print(versions.get('tree-sitter', 'unknown') + ' ' + versions.get('tree-sitter-bash', 'unknown'))
PY
)
python_tree_sitter=${py_versions%% *}
python_tree_sitter_bash=${py_versions##* }

if [[ "$mode" == "check" ]]; then
  if [[ ! -f "$repo_root/VENDOR.json" ]]; then
    echo "VENDOR.json is missing" >&2
    exit 1
  fi
  recorded_commit=$(python3 - "$repo_root/VENDOR.json" <<'PY'
import json, sys
with open(sys.argv[1], encoding='utf-8') as f:
    print(json.load(f)['source']['commit'])
PY
)
  if [[ "$recorded_commit" != "$source_commit" ]]; then
    echo "VENDOR.json commit $recorded_commit does not match source HEAD $source_commit" >&2
    exit 1
  fi
  recorded_py_bash=$(python3 - "$repo_root/VENDOR.json" <<'PY'
import json, sys
with open(sys.argv[1], encoding='utf-8') as f:
    print(json.load(f)['python']['tree_sitter_bash'])
PY
)
  if [[ "$recorded_py_bash" != "$python_tree_sitter_bash" ]]; then
    echo "VENDOR.json Python tree-sitter-bash $recorded_py_bash does not match source lock $python_tree_sitter_bash" >&2
    exit 1
  fi
  if ! diff -r "$commands_src" "$repo_root/vendor/commands"; then
    echo "vendored commands differ from recorded source commit $recorded_commit" >&2
    exit 1
  fi
  if ! diff -r "$schemas_src" "$repo_root/vendor/schemas"; then
    echo "vendored schemas differ from recorded source commit $recorded_commit" >&2
    exit 1
  fi
  if ! cmp -s "$spec_src" "$repo_root/vendor/SPEC.md"; then
    echo "vendored SPEC.md differs from recorded source commit $recorded_commit" >&2
    exit 1
  fi
  if ! cmp -s "$license_src" "$repo_root/vendor/LICENSE"; then
    echo "vendored LICENSE differs from recorded source commit $recorded_commit" >&2
    exit 1
  fi
  expected_notice="Files under this directory are vendored from https://github.com/fprochazka/bash-classify at the commit recorded in VENDOR.json and are Copyright (c) 2025 Filip Prochazka, MIT licensed (see LICENSE in this directory). They must remain byte-identical to upstream; sync only via scripts/vendor.sh. Chrysent LLC does not claim ownership of these files."
  if [[ ! -f "$repo_root/vendor/NOTICE" ]] || [[ $(<"$repo_root/vendor/NOTICE") != "$expected_notice" ]]; then
    echo "vendor/NOTICE is out of date; run scripts/vendor.sh" >&2
    exit 1
  fi
  echo "vendor check ok: $source_commit"
  exit 0
fi

python3 - "$source_dir" "$repo_root" <<'PY'
import shutil
import sys
from pathlib import Path

source = Path(sys.argv[1])
repo = Path(sys.argv[2])

pairs = [
    (source / 'src/bash_classify/commands', repo / 'vendor/commands', '*.yaml'),
    (source / 'schemas', repo / 'vendor/schemas', '*'),
]
for src_dir, dst_dir, pattern in pairs:
    dst_dir.mkdir(parents=True, exist_ok=True)
    wanted = {p.name for p in src_dir.glob(pattern) if p.is_file()}
    for stale in dst_dir.iterdir():
        if stale.is_file() and stale.name not in wanted:
            stale.unlink()
    for name in sorted(wanted):
        shutil.copy2(src_dir / name, dst_dir / name)
shutil.copy2(source / 'SPEC.md', repo / 'vendor/SPEC.md')
shutil.copy2(source / 'LICENSE', repo / 'vendor/LICENSE')
(repo / 'vendor/NOTICE').write_text(
    'Files under this directory are vendored from https://github.com/fprochazka/bash-classify at the commit recorded in VENDOR.json and are Copyright (c) 2025 Filip Prochazka, MIT licensed (see LICENSE in this directory). They must remain byte-identical to upstream; sync only via scripts/vendor.sh. Chrysent LLC does not claim ownership of these files.\n',
    encoding='utf-8',
)
PY

"$repo_root/scripts/gen_commands_data.sh"

rust_tree_sitter=$(python3 - "$repo_root/Cargo.lock" <<'PY'
import sys
from pathlib import Path
p = Path(sys.argv[1])
if not p.exists():
    print('unresolved')
    raise SystemExit
current = None
for line in p.read_text().splitlines():
    if line.startswith('name = '):
        current = line.split('"', 2)[1]
    elif current == 'tree-sitter' and line.startswith('version = '):
        print(line.split('"', 2)[1])
        raise SystemExit
print('unresolved')
PY
)

python3 - "$repo_root/VENDOR.json" "$source_dir" "$source_commit" "$python_tree_sitter" "$python_tree_sitter_bash" "$rust_tree_sitter" <<'PY'
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

out = Path(sys.argv[1])
source_dir, commit, py_ts, py_bash, rust_ts = sys.argv[2:7]
metadata = {
    "source": {
        "path": source_dir,
        "repository": "https://github.com/fprochazka/bash-classify",
        "commit": commit,
    },
    "python": {
        "tree_sitter": py_ts,
        "tree_sitter_bash": py_bash,
    },
    "rust": {
        "tree_sitter": rust_ts,
        "tree_sitter_bash": "0.25.1",
    },
    "synced_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace('+00:00', 'Z'),
    "policy": "Vendored payload files are byte-identical copies from the recorded source commit; vendor/LICENSE is copied from upstream; vendor/NOTICE is generated by scripts/vendor.sh; regenerate golden outputs after behavior-changing syncs.",
}
out.write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
PY

echo "vendor sync ok: $source_commit"
