# Development Setup

This page covers how to set up a local development environment for working on `token-privilege`.

[TOC]

## Prerequisites

- **Rust 1.91+** -- the crate uses the 2024 edition
- **Git**
- **[mise](https://mise.jdx.dev/)** -- manages all development tool versions (Rust toolchain, cargo extensions, formatters, linters)
- **[just](https://just.systems/)** -- command runner (installed automatically by mise)

## Initial Setup

Clone the repository and install all tools:

```bash
git clone https://github.com/EvilBit-Labs/token-privilege.git
cd token-privilege
just setup
```

`just setup` runs `mise install`, which reads `mise.toml` and installs all required tool versions. This includes `cargo-nextest`, `cargo-llvm-cov`, `cargo-audit`, `cargo-deny`, `mdbook`, and other development tools.

## Common Commands

All development tasks are orchestrated through the justfile. Run `just` (with no arguments) to see all available recipes.

### Building

| Command              | Description                    |
| -------------------- | ------------------------------ |
| `just build`         | Build the workspace (debug).   |
| `just build-release` | Build the workspace (release). |

### Testing

| Command         | Description                         |
| --------------- | ----------------------------------- |
| `just test`     | Run all tests with `cargo nextest`. |
| `just test-all` | Include ignored and slow tests.     |

Run a single test by name:

```bash
cargo nextest run -E 'test(test_name)'
```

### Formatting and Linting

| Command          | Description                                      |
| ---------------- | ------------------------------------------------ |
| `just fmt`       | Format all Rust code.                            |
| `just fmt-check` | Check formatting without modifying files.        |
| `just lint`      | Run all linters (Rust, actions, docs, justfile). |
| `just lint-rust` | Format check + Clippy (pedantic, all features).  |
| `just fix`       | Auto-fix Clippy warnings.                        |

### Quality Checks

| Command         | Description                                                                          |
| --------------- | ------------------------------------------------------------------------------------ |
| `just check`    | Pre-commit hooks + all linters (local quality gate).                                 |
| `just ci-check` | Full CI parity: pre-commit, fmt, clippy, test, build-release, audit, coverage, docs. |

### Coverage

| Command                 | Description                             |
| ----------------------- | --------------------------------------- |
| `just coverage`         | Generate LCOV coverage report.          |
| `just coverage-check`   | Coverage with `--fail-under-lines 85`.  |
| `just coverage-report`  | HTML coverage report, opens in browser. |
| `just coverage-summary` | Print coverage summary by file.         |

### Security

| Command      | Description                                             |
| ------------ | ------------------------------------------------------- |
| `just audit` | Run `cargo audit` for known vulnerabilities.            |
| `just deny`  | Run `cargo deny check` for license and advisory checks. |

### Documentation

| Command           | Description                               |
| ----------------- | ----------------------------------------- |
| `just docs-build` | Build mdBook and rustdoc.                 |
| `just docs-serve` | Serve docs locally with live reload.      |
| `just docs-check` | Validate rustdoc links and mdBook build.  |
| `just docs-clean` | Remove generated documentation artifacts. |

## Editor Configuration

The repository includes an `.editorconfig` file. Ensure your editor respects it or configure the following manually:

- Indent with 4 spaces (Rust files)
- UTF-8 encoding
- LF line endings
- Trailing whitespace trimmed

## Pre-Commit Hooks

Pre-commit hooks are configured via `.pre-commit-config.yaml`. They run automatically on `git commit` and can be run manually with:

```bash
just check
```

## Windows Development

The justfile supports Windows via PowerShell. All `just` commands work on both Windows and Unix. Windows is the primary platform for actually running the FFI tests -- Linux and macOS only exercise the non-Windows stub paths.
