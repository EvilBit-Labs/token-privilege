# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability in `token-privilege`, please report it responsibly through one of the following channels:

- **GitHub Private Vulnerability Reporting**: [Report here](https://github.com/EvilBit-Labs/token-privilege/security/advisories/new)
- **Email**: [security@evilbitlabs.io](mailto:security@evilbitlabs.io)

Please do NOT open a public issue for security vulnerabilities.

Include the following in your report:

- A description of the vulnerability and its potential impact.
- Steps to reproduce or a proof-of-concept.
- The version(s) affected.
- Any suggested fix, if you have one.

## Scope

The following are considered in-scope for security reports:

- **Unsound unsafe code** -- any `unsafe` block in `ffi.rs` that violates Rust safety invariants or causes undefined behavior.
- **Privilege escalation bugs** -- any code path that could enable, disable, or modify privileges (the crate is strictly read-only by design).
- **Information leaks via error messages** -- error variants or messages that expose sensitive system state to unprivileged callers.

## Out of Scope

The following are NOT considered vulnerabilities in this project:

- **Windows kernel bugs** -- issues in the Windows kernel or Win32 API itself.
- **Physical access attacks** -- scenarios requiring physical access to the machine.
- **Upstream `windows` crate issues** -- bugs in the `windows` crate should be reported to [microsoft/windows-rs](https://github.com/microsoft/windows-rs).
- **Denial of service via expected API failures** -- the crate returns `Result` errors for all failure cases by design.

## Response Timeline

| Stage        | Target  |
| ------------ | ------- |
| Acknowledge  | 1 week  |
| Assess       | 2 weeks |
| Fix released | 90 days |

We will coordinate with you on disclosure timing. If a fix requires longer than 90 days, we will communicate the revised timeline.

## Security Features

`token-privilege` incorporates the following security measures:

- **Isolated unsafe boundary** -- all `unsafe` Win32 FFI is confined to a single internal module (`ffi.rs`) that is not publicly exported.
- **Strict linting** -- `undocumented_unsafe_blocks = "deny"`, `clippy::unwrap_used = "deny"`, and `clippy::panic = "deny"` are enforced crate-wide.
- **Read-only queries** -- the crate never enables, disables, or removes privileges. It only reads token state.
- **RAII handle management** -- Win32 `HANDLE` values are wrapped in a type that calls `CloseHandle` on `Drop`, preventing handle leaks.
- **Dependency auditing** -- `cargo-audit` and `cargo-deny` are run in CI to detect known vulnerabilities in dependencies.

## Safe Harbor

We consider security research conducted in good faith to be authorized. We will not pursue legal action against researchers who:

- Make a good-faith effort to avoid privacy violations, data destruction, and service disruption.
- Report vulnerabilities through the channels listed above.
- Allow reasonable time for the issue to be resolved before public disclosure.

We ask that you do not access or modify other users' data, and that you only interact with accounts you own or have explicit permission to test.
