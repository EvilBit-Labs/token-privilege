# Contributing to token-privilege

Thank you for your interest in contributing to token-privilege! This document provides guidelines and information for contributors.

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). Please be respectful and constructive in all interactions.

## Gotchas

Before working in a specific area, check [GOTCHAS.md](GOTCHAS.md) for hard-won lessons and edge cases organized by domain. It covers unsafe code rules, clippy/rustdoc pitfalls, CI quirks, pre-commit hook behavior, and platform-specific mmap limitations.

## Prerequisites

- **Rust 1.85+** (see `rust-toolchain.toml`)
- **Git** with commit signing configured
- **[mise](https://mise.jdx.dev/)** for toolchain management

## Setup

```bash
git clone https://github.com/EvilBit-Labs/token-privilege.git
cd token-privilege
mise install
cargo build
```

After `mise install`, the following tools are available: `just`, `cargo-nextest`, `cargo-llvm-cov`, `cargo-deny`, and `cargo-audit`.

## Development Commands

All commands are orchestrated through the `justfile`:

| Command               | Description                                  |
| --------------------- | -------------------------------------------- |
| `just ci-check`       | Full CI parity check (run before pushing)    |
| `just test`           | Run all tests via `cargo nextest`            |
| `just test-all`       | Run all tests including ignored/slow tests   |
| `just lint`           | Run all linters (fmt, clippy, actions, docs) |
| `just fmt`            | Format all code                              |
| `just coverage`       | Generate lcov coverage report                |
| `just coverage-check` | Coverage with `--fail-under-lines 85`        |
| `just audit`          | Run `cargo audit`                            |
| `just deny`           | Run `cargo deny check`                       |
| `just docs-build`     | Build rustdoc and mdBook                     |

Run a single test:

```bash
cargo nextest run -E 'test(test_name)'
```

Run clippy manually:

```bash
cargo clippy -- -D warnings
```

## Architecture

The crate has five source files in `src/`:

| File           | Role                                                           |
| -------------- | -------------------------------------------------------------- |
| `lib.rs`       | Public API, re-exports, module declarations, non-Windows stubs |
| `elevation.rs` | `is_elevated()` implementation                                 |
| `privilege.rs` | Privilege querying and enumeration functions                   |
| `error.rs`     | `TokenPrivilegeError` enum (uses `thiserror`)                  |
| `ffi.rs`       | All unsafe Win32 FFI, RAII handle wrapper                      |

**Layered design**: consumer crate -> public API (`lib.rs`) -> domain modules (`elevation.rs`, `privilege.rs`) -> FFI boundary (`ffi.rs`) -> `windows` crate -> Win32 kernel.

## Branching

Create branches from `main` using the following prefixes:

- `feat/` -- new features
- `fix/` -- bug fixes
- `docs/` -- documentation only
- `refactor/` -- code restructuring without behavior change
- `test/` -- adding or updating tests
- `chore/` -- maintenance (deps, CI config, tooling)
- `perf/` -- performance improvements
- `ci/` -- CI/CD pipeline changes

## Code Quality

Before submitting a PR, verify that:

- [ ] All tests pass (`just test`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code is formatted (`just fmt-check`)
- [ ] Documentation builds (`just docs-build`)
- [ ] Coverage is at least 85% (`just coverage-check`)

## Safety Rules

- All `unsafe` code MUST be confined to `ffi.rs`.
- Every `unsafe` block MUST have a `// SAFETY:` comment explaining the invariants.
- `undocumented_unsafe_blocks = "deny"` is enforced via Clippy configuration.
- Never introduce `unsafe` in any other module. If you need new FFI calls, add them to `ffi.rs` and expose a safe `pub(crate)` wrapper.

## Testing

- **Unit tests** live in each module as `#[cfg(test)] mod tests`.
- **Integration tests** live in `tests/`.
- Windows-specific tests are gated with `#[cfg(target_os = "windows")]`.
- Non-Windows stub tests verify that all public functions return `Err(TokenPrivilegeError::UnsupportedPlatform)`.
- `SeChangeNotifyPrivilege` is the reliable test privilege -- it is enabled on all Windows processes by default.
- Use `cargo-nextest` (not `cargo test`) for running tests.

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```text
<type>(<scope>): <description>
```

**Types**: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `ci`

**Scopes**: `lib`, `api`, `error`, `elevation`, `privilege`, `privileges`, `ffi`, `safety`, `security`, `docs`, `book`, `tests`, `ci`, `deps`, `release`

Special rules:

- Changes touching `unsafe` MUST use `ffi` or `safety` scope with safety invariant in the commit body.
- Cross-platform behavior changes must note Windows vs non-Windows impact.

## Pull Request Process

1. **Fork** the repository.
2. **Create a branch** from `main` using the naming convention above.
3. **Make your changes** with conventional commit messages.
4. **Sign your commits** with the DCO sign-off (`git commit -s`).
5. **Open a PR** against `main` with a clear description of the change.
6. **Ensure CI passes** -- the pipeline runs formatting, linting, tests, and coverage checks.
7. A maintainer will review your PR. Address any feedback and push additional commits (do not force-push during review).

All contributions require a [Developer Certificate of Origin](https://developercertificate.org/) sign-off. Use the `-s` flag when committing:

```bash
git commit -s -m "feat(privilege): add new privilege constant"
```

## Maintainers

- [@unclesp1d3r](https://github.com/unclesp1d3r)
- [@KryptoKat08](https://github.com/KryptoKat08)

## License

By contributing, you agree that your contributions will be dual licensed under MIT and Apache-2.0, without any additional terms or conditions.
