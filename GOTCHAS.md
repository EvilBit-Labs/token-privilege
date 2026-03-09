# GOTCHAS.md

Hard-won lessons, edge cases, and "watch out for" patterns. Organized by domain.

Referenced from [AGENTS.md](AGENTS.md) and [CONTRIBUTING.md](CONTRIBUTING.md) -- read the relevant section before working in that area.

## Testing Strategy: proptest over cargo-fuzz

We use `proptest` for property-based testing instead of `cargo-fuzz` (libFuzzer). Reasons:

1. **Windows support** — `cargo-fuzz` relies on libFuzzer, which has limited Windows support. This crate's real functionality only runs on Windows, so fuzzing on Linux would only exercise the `UnsupportedPlatform` stubs.
2. **Narrow attack surface** — The crate accepts `&str` privilege names and calls Win32 APIs. There are no parsers, deserializers, or complex input formats that benefit from coverage-guided fuzzing.
3. **proptest integrates with `cargo nextest`** — Property tests run as regular `#[test]` functions, no separate toolchain or CI job needed.
4. **Deterministic reproduction** — proptest generates regression files for failing cases, making bugs reproducible without a corpus directory.

Use proptest to generate random/adversarial privilege name strings and verify the crate handles them gracefully (returns errors, never panics).

## Platform Conditional Compilation

- All Win32 FFI and domain logic is gated with `#[cfg(target_os = "windows")]`.
- Non-Windows builds get `const fn` stubs returning `Err(UnsupportedPlatform)`.
- `SeChangeNotifyPrivilege` is the reliable test privilege — enabled on every Windows process by default.
- CI runs primarily on Windows. A single Linux job validates stub behavior.

## Avoiding `expect`/`unwrap` Under Denied Lints

`clippy::panic` and `clippy::unwrap_used` are denied in production code. For compile-time-known values (e.g., `size_of::<T>()`), don't use `try_from(...).expect(...)`. Instead:

1. Add a module-level `const _: () = assert!(...)` to verify the assumption at compile time.
2. Use `const X: u32 = size_of::<T>() as u32` — a `const` item cannot panic, satisfying both lints.

## MSRV

MSRV is 1.91. All dependencies (including `thiserror` 2.x and `windows` 0.62) support 1.91. Don't bump without checking the full dependency tree with `cargo metadata`.
