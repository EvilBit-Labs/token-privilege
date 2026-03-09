# Cross-platform justfile using OS annotations
# Windows uses PowerShell, Unix uses bash

set windows-shell := ["powershell.exe", "-c"]
set shell := ["bash", "-c"]
set dotenv-load := true
set ignore-comments := true

# Use mise to manage all dev tools
# See mise.toml for tool versions

mise_exec := "mise exec --"

# =============================================================================
# GENERAL COMMANDS
# =============================================================================

default:
    @just --list

# =============================================================================
# SETUP AND INITIALIZATION
# =============================================================================

# Development setup - mise handles all tool installation via mise.toml
setup:
    mise install

# =============================================================================
# FORMATTING AND LINTING
# =============================================================================

alias format-rust := fmt
alias format-md := format-docs
alias format-just := fmt-justfile

# Main format recipe - calls all formatters
format: fmt format-json-yaml format-docs fmt-justfile

format-json-yaml:
    @{{ mise_exec }} prettier --write "**/*.{json,yaml,yml}"

format-docs:
    @{{ mise_exec }} mdformat --exclude "target/*" --exclude "node_modules/*" .

fmt:
    @{{ mise_exec }} cargo fmt --all

fmt-check:
    @{{ mise_exec }} cargo fmt --all --check

lint-rust: fmt-check
    @{{ mise_exec }} cargo clippy --workspace --all-targets --all-features -- -D warnings -A clippy::multiple_crate_versions

lint-rust-min:
    @{{ mise_exec }} cargo clippy --workspace --all-targets --no-default-features -- -D warnings -A clippy::multiple_crate_versions

# Format justfile
fmt-justfile:
    @just --fmt --unstable

# Lint justfile formatting
lint-justfile:
    @just --fmt --check --unstable

# Main lint recipe - calls all sub-linters
lint: lint-rust lint-actions lint-docs lint-justfile

lint-actions:
    @{{ mise_exec }} actionlint .github/workflows/audit.yml .github/workflows/ci.yml .github/workflows/compat.yml .github/workflows/docs.yml .github/workflows/release-plz.yml .github/workflows/scorecard.yml .github/workflows/security.yml

lint-docs:
    @{{ mise_exec }} markdownlint-cli2 docs/**/*.md README.md
    @{{ mise_exec }} lychee docs/**/*.md README.md

alias lint-just := lint-justfile

# Run clippy with fixes
fix:
    @{{ mise_exec }} cargo clippy --fix --allow-dirty --allow-staged

# Quick development check
check: pre-commit-run lint

[private]
pre-commit-run:
    @{{ mise_exec }} pre-commit run -a

# Format a single file (for pre-commit hooks)
format-files +FILES:
    @{{ mise_exec }} prettier --write --config .prettierrc.json {{ FILES }}

# =============================================================================
# BUILDING AND TESTING
# =============================================================================

build:
    @{{ mise_exec }} cargo build --workspace

build-release:
    @{{ mise_exec }} cargo build --workspace --release

test:
    @{{ mise_exec }} cargo nextest run --workspace --no-capture

test-ci:
    @{{ mise_exec }} cargo nextest run --workspace --no-capture

# Run all tests including ignored/slow tests across workspace
test-all:
    @{{ mise_exec }} cargo nextest run --workspace --no-capture -- --ignored

# =============================================================================
# SECURITY AND AUDITING
# =============================================================================

audit:
    @{{ mise_exec }} cargo audit

deny:
    @{{ mise_exec }} cargo deny check

# =============================================================================
# CI AND QUALITY ASSURANCE
# =============================================================================

# Private helper: run cargo llvm-cov with proper setup
[private]
[unix]
_coverage +args:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf target/llvm-cov-target
    RUSTFLAGS="--cfg coverage" {{ mise_exec }} cargo llvm-cov --workspace --lcov --output-path lcov.info {{ args }}

[private]
[windows]
_coverage +args:
    Remove-Item -Recurse -Force target/llvm-cov-target -ErrorAction SilentlyContinue
    $env:RUSTFLAGS = "--cfg coverage"; {{ mise_exec }} cargo llvm-cov --workspace --lcov --output-path lcov.info {{ args }}

coverage:
    @just _coverage

coverage-check:
    @just _coverage --fail-under-lines 85

# Generate HTML coverage report for local viewing
[unix]
coverage-report:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf target/llvm-cov-target
    RUSTFLAGS="--cfg coverage" {{ mise_exec }} cargo llvm-cov --workspace --html --open

[windows]
coverage-report:
    $env:RUSTFLAGS = "--cfg coverage"; {{ mise_exec }} cargo llvm-cov --workspace --html --open

# Show coverage summary by file
[unix]
coverage-summary:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf target/llvm-cov-target
    RUSTFLAGS="--cfg coverage" {{ mise_exec }} cargo llvm-cov --workspace

[windows]
coverage-summary:
    $env:RUSTFLAGS = "--cfg coverage"; {{ mise_exec }} cargo llvm-cov --workspace

# Full local CI parity check
ci-check: pre-commit-run fmt-check lint-rust lint-rust-min test-ci build-release audit coverage-check docs-check

# =============================================================================
# DOCUMENTATION
# =============================================================================

# Build complete documentation (mdBook + rustdoc)
[unix]
docs-build:
    #!/usr/bin/env bash
    set -euo pipefail
    # Build rustdoc
    {{ mise_exec }} cargo doc --no-deps --document-private-items --target-dir docs/book/api-temp
    # Move rustdoc output to final location
    mkdir -p docs/book/api
    cp -r docs/book/api-temp/doc/* docs/book/api/
    rm -rf docs/book/api-temp
    # Build mdBook
    cd docs && {{ mise_exec }} mdbook build

[windows]
docs-build:
    # Build rustdoc
    {{ mise_exec }} cargo doc --no-deps --document-private-items --target-dir docs/book/api-temp
    # Move rustdoc output to final location
    New-Item -ItemType Directory -Force -Path docs/book/api | Out-Null; Copy-Item -Recurse -Force docs/book/api-temp/doc/* docs/book/api/; Remove-Item -Recurse -Force docs/book/api-temp
    # Build mdBook
    Set-Location docs; {{ mise_exec }} mdbook build

# Serve documentation locally with live reload
[unix]
docs-serve:
    cd docs && {{ mise_exec }} mdbook serve --open

[windows]
docs-serve:
    Set-Location docs; {{ mise_exec }} mdbook serve --open

# Clean documentation artifacts
[unix]
docs-clean:
    rm -rf docs/book target/doc

[windows]
docs-clean:
    Remove-Item -Recurse -Force docs/book, target/doc -ErrorAction SilentlyContinue

# Check documentation (rustdoc link validation + mdBook build)
[unix]
docs-check:
    @{{ mise_exec }} cargo doc --no-deps --document-private-items
    cd docs && {{ mise_exec }} mdbook build

[windows]
docs-check:
    {{ mise_exec }} cargo doc --no-deps --document-private-items
    Set-Location docs; {{ mise_exec }} mdbook build

# Generate and serve documentation
[unix]
docs: docs-build docs-serve

[windows]
docs: docs-build docs-serve

# =============================================================================
# THIRD-PARTY NOTICES
# =============================================================================

# Regenerate THIRD_PARTY_NOTICES.md from current dependencies
third-party-notices:
    @{{ mise_exec }} cargo about generate about.hbs -o THIRD_PARTY_NOTICES.md

# =============================================================================
# CHANGELOG
# =============================================================================

# Generate changelog
[group('docs')]
changelog:
    @{{ mise_exec }} git-cliff --output CHANGELOG.md

# Generate changelog for a specific version
[group('docs')]
changelog-version version:
    @{{ mise_exec }} git-cliff --tag {{ version }} --output CHANGELOG.md

# Generate changelog for unreleased changes only
[group('docs')]
changelog-unreleased:
    @{{ mise_exec }} git-cliff --unreleased --output CHANGELOG.md

# =============================================================================
# RELEASE MANAGEMENT
# =============================================================================

release:
    @{{ mise_exec }} cargo release

release-dry-run:
    @{{ mise_exec }} cargo release --dry-run

release-patch:
    @{{ mise_exec }} cargo release patch

release-minor:
    @{{ mise_exec }} cargo release minor

release-major:
    @{{ mise_exec }} cargo release major
