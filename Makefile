# SPDX-FileCopyrightText: 2026 Chrysent LLC
# SPDX-License-Identifier: MIT
export BUMP VERSION
.PHONY: version bump-version release publish
version:
	@bash scripts/version.sh version
bump-version:
	@bash scripts/version.sh bump
release:
	@bash scripts/version.sh release
publish:
	@test -n "$${CARGO_REGISTRY_TOKEN:-}" || { echo 'CARGO_REGISTRY_TOKEN must be exported and nonempty' >&2; exit 1; }
	@CARGO_REGISTRY_CREDENTIAL_PROVIDER=cargo:token cargo publish --registry crates-io --locked
