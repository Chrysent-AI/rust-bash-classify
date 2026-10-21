<!-- SPDX-FileCopyrightText: 2026 Chrysent LLC -->
<!-- SPDX-License-Identifier: MIT -->

# Repository secret and privacy checks

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
- `commit-msg` scans the proposed commit message with the same rules.
- Both extend gitleaks' default secret detection with generic home-directory,
  UNC-share, and consumer-email patterns. Noreply GitHub identities are allowed.
  Only reviewed synthetic home-path fixtures have rule-specific, exact-path
  exceptions in named source and golden files; secret and email rules still apply.
- If gitleaks is missing, the hooks print a notice and allow the operation.
  These are local safeguards, not an enforced CI or server-side policy.

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
