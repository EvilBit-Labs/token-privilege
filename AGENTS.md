# AGENTS.md

This file provides guidance to AI coding assistants when working with code in this repository.

@GOTCHAS.md

## Project Overview

`token-privilege` is a safe Rust crate wrapping Windows process token privilege and elevation detection APIs. All `unsafe` Win32 FFI is confined to a single internal module (`ffi.rs`) so downstream consumers can use `#![forbid(unsafe_code)]`. On non-Windows platforms, all public functions return `Err(TokenPrivilegeError::UnsupportedPlatform)`.

Dual-licensed MIT / Apache-2.0. Rust 2024 edition, MSRV 1.85.

## Build & Development Commands

Tooling is managed via [mise](https://mise.jdx.dev/) (`mise.toml`) and orchestrated with [just](https://just.systems/) (`justfile`). Run `just setup` first to install all tools.

```bash
just build              # cargo build --workspace
just test               # cargo nextest run (all workspace, with output)
just test-all           # includes ignored/slow tests
just fmt                # cargo fmt --all
just fmt-check          # cargo fmt --all --check
just lint-rust          # fmt-check + clippy (pedantic, all features, -D warnings)
just lint               # lint-rust + lint-actions + lint-docs + lint-justfile
just fix                # cargo clippy --fix
just check              # pre-commit + lint (full local quality gate)
just ci-check           # full CI parity: pre-commit, fmt, clippy, test, build-release, audit, coverage, docs
just coverage           # cargo llvm-cov (lcov output)
just coverage-check     # coverage with --fail-under-lines 85
just coverage-report    # HTML coverage report, opens in browser
just audit              # cargo audit
just deny               # cargo deny check
just docs-build         # rustdoc + mdBook
just docs-serve         # mdBook with live reload
```

Run a single test: `cargo nextest run -E 'test(test_name)'`

## Architecture

```text
src/
  lib.rs          — Public API, re-exports, module declarations, non-Windows stubs
  elevation.rs    — is_elevated() implementation
  privilege.rs    — is_privilege_enabled(), has_privilege(), enumerate_privileges()
  error.rs        — TokenPrivilegeError (uses thiserror)
  ffi.rs          — All unsafe Win32 FFI (pub(crate) only, RAII handle wrapper)
```

**Layered design**: Consumer crate → public API (lib.rs) → domain modules (elevation.rs, privilege.rs) → FFI boundary (ffi.rs) → `windows` crate → Win32 kernel.

### Public API

- `is_elevated() -> Result<bool, TokenPrivilegeError>` — UAC elevation check
- `is_privilege_enabled(name: &str) -> Result<bool, TokenPrivilegeError>` — check if privilege is enabled
- `has_privilege(name: &str) -> Result<bool, TokenPrivilegeError>` — check if privilege is present (enabled or not)
- `enumerate_privileges() -> Result<Vec<PrivilegeInfo>, TokenPrivilegeError>` — list all token privileges
- `privileges::*` — well-known privilege name constants (e.g., `SE_DEBUG`, `SE_BACKUP`)

### Key design constraints

- `unsafe_code` is NOT forbidden at crate level — this crate IS the unsafe boundary
- All `unsafe` blocks MUST have `// SAFETY:` comments (`undocumented_unsafe_blocks = "deny"`)
- `clippy::panic` is denied, `clippy::unwrap_used` is denied — use `Result` returns
- RAII pattern for Win32 `HANDLE` (calls `CloseHandle` on `Drop`)
- Read-only: crate never modifies privileges, only queries them

## Linting

Clippy is configured aggressively in `Cargo.toml` under `[workspace.lints.clippy]`:

- `pedantic`, `nursery`, `cargo` groups enabled as warnings
- `correctness` group denied
- Security-focused lints: `unwrap_used = "deny"`, `panic = "deny"`, `undocumented_unsafe_blocks = "deny"`
- `rustfmt.toml`: edition 2024, style_edition 2024

## Commit Conventions

Conventional Commits format: `<type>(<scope>): <description>`

Scopes: `lib`, `api`, `error`, `elevation`, `privilege`, `privileges`, `ffi`, `safety`, `security`, `docs`, `book`, `tests`, `ci`, `deps`, `release`

Special rules:

- Changes touching `unsafe` MUST use `ffi` or `safety` scope with safety invariant in body
- Cross-platform behavior changes must note Windows vs non-Windows impact
- See `.github/commit-instructions.md` for full details and examples

## Testing

- Tests use `cargo-nextest` (not `cargo test`)
- Windows-specific tests gated with `#[cfg(target_os = "windows")]`
- Non-Windows stub tests verify `Err(UnsupportedPlatform)` on all platforms
- `SeChangeNotifyPrivilege` is the reliable test privilege (enabled on all Windows processes by default)
- Coverage target: 85% line coverage (enforced by `just coverage-check`)
- Dev dependencies: `proptest` for property-based testing, `tempfile`

## CI

CI runs on push to `main` and PRs. Pipeline: quality (fmt + clippy) → test → cross-platform (Linux, macOS, Windows) → coverage (Codecov). Windows runner is where actual Win32 FFI tests execute.

## Platform Targets

`x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`
