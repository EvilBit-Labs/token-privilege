# Testing

This page describes the testing strategy, tools, and conventions used in `token-privilege`.

[TOC]

## Test Runner

Tests are run with [cargo-nextest](https://nexte.st/) instead of the built-in `cargo test`. nextest provides better output, parallel execution, and per-test timeouts.

```bash
just test                # Run all tests
just test-all            # Include ignored/slow tests
```

Run a single test by name:

```bash
cargo nextest run -E 'test(test_name)'
```

## Test Organization

### Unit Tests Per Module

Each source module has `#[cfg(test)] mod tests` at the bottom with unit tests for its internal logic:

- **`error.rs`** -- display formatting for each error variant, `Send + Sync` bounds.
- **`elevation.rs`** -- (Windows only) `is_elevated()` returns `Ok`, result is consistent across calls.
- **`privilege.rs`** -- (Windows only) `SeChangeNotifyPrivilege` is enabled, invalid names return errors, enumeration is non-empty.
- **`ffi.rs`** -- (Windows only) handle open/close cycle, elevation query, privilege lookup (valid and invalid), privilege check, enumeration.

### Stub Tests

`lib.rs` contains stub tests gated with `#[cfg(not(target_os = "windows"))]` that verify all public functions return `Err(TokenPrivilegeError::UnsupportedPlatform)` on non-Windows platforms:

```rust,ignore
#[cfg(not(target_os = "windows"))]
#[cfg(test)]
mod stub_tests {
    #[test]
    fn is_elevated_returns_unsupported() { /* ... */ }
    #[test]
    fn is_privilege_enabled_returns_unsupported() { /* ... */ }
    #[test]
    fn has_privilege_returns_unsupported() { /* ... */ }
    #[test]
    fn enumerate_privileges_returns_unsupported() { /* ... */ }
}
```

### Platform-Gated Tests

Windows-specific tests in `elevation.rs`, `privilege.rs`, and `ffi.rs` are compiled only when `target_os = "windows"` because those modules are conditionally compiled. This means:

- **Linux/macOS CI runners** execute the stub tests and `error.rs` tests.
- **Windows CI runners** execute the full FFI test suite.

### Reliable Test Privilege

`SeChangeNotifyPrivilege` is used as the canonical test privilege because it is enabled by default on all Windows process tokens. Tests relying on this privilege do not require Administrator elevation to pass.

## Property-Based Testing

The crate includes [proptest](https://proptest-rs.github.io/proptest/) as a dev dependency for property-based testing. Proptest generates random inputs and verifies that invariants hold across many cases. This is particularly useful for testing error handling with arbitrary privilege name strings.

## Coverage

Coverage is measured with [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) and reported in LCOV format.

### Running Coverage Locally

```bash
just coverage            # Generate lcov.info
just coverage-check      # Fail if line coverage < 85%
just coverage-report     # HTML report, opens in browser
just coverage-summary    # Print per-file summary
```

### Coverage Target

The project enforces a minimum of **85% line coverage**, checked by:

```bash
just coverage-check
```

This threshold is also enforced in the `ci-check` recipe and the CI pipeline.

### Coverage in CI

The CI pipeline generates coverage on Ubuntu, uploads the LCOV report to [Codecov](https://codecov.io/), and reports results on pull requests.

## CI Test Matrix

The CI workflow runs tests across multiple platforms:

| Runner           | Platform | What Runs                              |
| ---------------- | -------- | -------------------------------------- |
| `ubuntu-latest`  | Linux    | Stub tests, error tests, quality gate. |
| `ubuntu-22.04`   | Linux    | Compatibility check on older Ubuntu.   |
| `macos-latest`   | macOS    | Stub tests, error tests.               |
| `windows-latest` | Windows  | Full FFI test suite.                   |

The pipeline structure is:

1. **quality** -- formatting and Clippy checks.
2. **test** -- run tests on Ubuntu.
3. **test-cross-platform** -- run tests on all platform runners (depends on **test**).
4. **coverage** -- generate and upload coverage (depends on **test**).

## Writing New Tests

When adding a new test:

1. Place it in the `#[cfg(test)] mod tests` block of the relevant module.
2. Gate Windows-specific tests with `#[cfg(target_os = "windows")]` (handled automatically since the module is already conditionally compiled).
3. Use `SeChangeNotifyPrivilege` when you need a privilege that is reliably present and enabled.
4. Return `Result` from tests when possible to avoid `.unwrap()` (which is denied by Clippy in non-test code but allowed in test modules).
5. Run `just coverage-check` to verify the 85% threshold is maintained.
