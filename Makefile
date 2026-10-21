# SPDX-FileCopyrightText: 2026 Chrysent LLC
# SPDX-License-Identifier: MIT
export BUMP VERSION
.PHONY: version bump-version release
version:
	@bash scripts/version.sh version
bump-version:
	@bash scripts/version.sh bump
release:
	@bash scripts/version.sh release
