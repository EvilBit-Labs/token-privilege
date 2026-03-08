# Security Assurance

`token-privilege` is a security-sensitive crate. This page documents the threat model, security properties, and the measures in place to maintain them.

[TOC]

## Threat Model

### In Scope

- **Incorrect privilege reporting.** A bug that reports a privilege as enabled when it is not (or vice versa) could lead to incorrect authorization decisions by consuming applications.
- **Resource leaks.** Failing to close a token handle could leak kernel resources.
- **Memory safety violations.** Incorrect FFI usage could lead to buffer overflows, use-after-free, or other undefined behavior.
- **Denial of service.** Pathological inputs (e.g., extremely long privilege names) should not cause panics or unbounded allocations.

### Out of Scope

- **Privilege escalation.** This crate is read-only; it never calls `AdjustTokenPrivileges` or any API that modifies the process token. It cannot elevate, enable, disable, or remove privileges.
- **Malicious OS behavior.** If the Windows kernel itself returns incorrect data, that is outside the crate's control.
- **Side-channel attacks.** Timing differences between privilege checks are not considered a threat for this use case.

## Security Properties

### Read-Only Access

The crate opens process tokens with `TOKEN_QUERY` access only. No write operations are performed. The token state is never modified.

### Unsafe Code Isolation

All `unsafe` blocks are confined to `src/ffi.rs`. The public API is entirely safe. Consumers can use `#![forbid(unsafe_code)]` in their own crates.

See the [Safety Contract](safety-contract.md) for a detailed audit of every unsafe block.

### No Panics, No Unwraps

The following Clippy lints are set to `deny` in `Cargo.toml`:

- `clippy::panic` -- no `panic!()`, `todo!()`, or `unimplemented!()` in non-test code.
- `clippy::unwrap_used` -- no `.unwrap()` calls; all fallible operations use `Result` propagation.

This ensures the crate returns structured errors instead of aborting the process.

### Documented Unsafe Blocks

The lint `clippy::undocumented_unsafe_blocks = "deny"` requires every `unsafe` block to have a `// SAFETY:` comment explaining why the operation is sound.

### Non-Exhaustive Error Type

`TokenPrivilegeError` is `#[non_exhaustive]`, allowing new error variants to be added in future releases without breaking downstream match arms. This prevents consumers from assuming they handle all possible errors.

## Static Analysis

### Clippy Configuration

The crate enables aggressive Clippy lint groups:

| Group         | Level | Purpose                           |
| ------------- | ----- | --------------------------------- |
| `correctness` | deny  | Catch definite bugs.              |
| `pedantic`    | warn  | Catch subtle issues.              |
| `nursery`     | warn  | Catch emerging patterns.          |
| `suspicious`  | warn  | Catch code that looks like a bug. |
| `cargo`       | warn  | Catch packaging issues.           |

Additional security-focused lints include `as_conversions`, `cast_ptr_alignment`, `indexing_slicing`, and `arithmetic_side_effects`.

### Formatting

`rustfmt` is configured for the 2024 edition and style. Formatting is checked in CI via `just fmt-check`.

## Dependency Auditing

### `cargo-audit`

`cargo audit` checks for known vulnerabilities in the dependency tree. Run locally with:

```bash
just audit
```

This is also enforced in CI via the `audit.yml` workflow.

### `cargo-deny`

`cargo deny check` validates licenses, bans problematic crates, and checks for security advisories. Run locally with:

```bash
just deny
```

### Minimal Dependencies

The crate has a single runtime dependency: `thiserror` for error derivation. The `windows` crate is only pulled in on Windows targets via `[target.'cfg(windows)'.dependencies]`.

Dev dependencies (`proptest`, `tempfile`) are not included in production builds.

## Test Coverage

The project targets 85% line coverage, enforced by:

```bash
just coverage-check
```

Coverage is generated with `cargo-llvm-cov` and reported to Codecov in CI. See the [Testing](testing.md) page for details on the testing strategy.

## CI Pipeline

Every push to `main` and every pull request runs through:

1. **Quality gate** -- `rustfmt` check and Clippy with `-D warnings`.
2. **Test suite** -- `cargo nextest run` on Ubuntu.
3. **Cross-platform tests** -- Linux, macOS, and Windows runners.
4. **Coverage** -- `cargo-llvm-cov` with Codecov upload.
5. **Audit** -- `cargo audit` for known vulnerabilities.
6. **Scorecard** -- OpenSSF Scorecard for supply chain security.
7. **Security scanning** -- Additional security workflow checks.

## Reporting Security Issues

If you discover a security vulnerability, please report it responsibly via GitHub's private vulnerability reporting feature on the repository: <https://github.com/EvilBit-Labs/token-privilege/security>
