<!-- SPDX-FileCopyrightText: 2026 Chrysent LLC -->
<!-- SPDX-License-Identifier: MIT -->

# Repository commit-message, secret and privacy checks

Install gitleaks (8.25 or newer for this configuration; verified with 8.30.1). From the repository
root, activate these hooks for this checkout only:

```sh
git config --local core.hooksPath .githooks
git hook run pre-commit
```

Check your existing hook configuration before activation: this setting replaces
hook routing for this repository. Integrate existing hooks rather than silently
disabling them. No global configuration is changed. Git clones do not activate
hooks automatically; each contributor must opt in.

- `pre-commit` scans staged changes with `.gitleaks.toml`.
- `commit-msg` validates Conventional Commit structure and scans the entire
  original message (subject, body and trailers) with the same rules. Neither
  check rewrites the message.
- Both extend gitleaks' default secret detection with generic home-directory,
  UNC-share, and consumer-email patterns. Noreply GitHub identities are allowed.
  Only reviewed synthetic home-path fixtures have rule-specific, exact-path
  exceptions in named source and golden files; secret and email rules still apply.
- If gitleaks is missing, the hooks print a notice and skip scanning, but
  `commit-msg` still enforces Conventional Commit syntax.
  These are local safeguards, not an enforced CI or server-side policy.

## Commit-message syntax

Use Conventional Commits 1.0.0: https://www.conventionalcommits.org/en/v1.0.0/

```text
feat(parser): recognize another command
fix!: reject ambiguous input
chore(release): release vX.Y.Z
```

The header is `type(scope)!: description`: scope and `!` are optional; the colon
must be followed by a space and a nonblank description. Types are noun-like
identifiers (a letter followed by letters, digits, underscores or hyphens).
Custom types and case variants are accepted, with no fixed type list or subject
length limit. A scope must contain meaningful, nonblank text, without nested
parentheses or control characters. A nonempty body or footer needs an empty line
after the header. LF, CRLF and a final line without a newline are supported.

Bodies and trailers are free-form. The hook checks structure, not whether `feat`,
`fix` or a breaking-change claim is semantically true, and does not parse footer
semantics. Follow the specification's uppercase `BREAKING CHANGE` convention.
Automatic `Merge`, `Revert` and `fixup!` titles are not exempt: rewrite them into
conventional form (for example, `revert: undo parser change`). No accepted title
bypasses the full-message secret/PII scan when gitleaks is available.

Run `bash tests/commit-msg.sh` for isolated effective-hook regression tests
(requires Git and gitleaks).

A rejection identifies the matched rule. Remove sensitive content; for a
confirmed synthetic example, use a narrowly scoped exception rather than
excluding a whole directory. Do not put personal literals in the shared config.
Generic patterns do not detect every personal identifier or verify Git author
and committer identities. History and commit messages need separate audits.

To disable this checkout's routing (only if it still points to `.githooks`):

```sh
git config --local --unset core.hooksPath
```

This restores the previously inherited/default hook routing, not any earlier
local override. Restore a previous local value if you had one. To remove the
shared setup entirely, also remove these hook files and `.gitleaks.toml`.
