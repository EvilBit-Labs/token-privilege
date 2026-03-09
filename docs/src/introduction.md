# Introduction

`token-privilege` is a safe Rust crate that wraps the Windows Win32 APIs for querying process token privileges and elevation status. It provides a fully safe public API so that downstream consumers can maintain `#![forbid(unsafe_code)]` in their own crates while still accessing low-level Windows security information.

## Why This Crate Exists

Querying Windows process tokens requires multiple Win32 FFI calls involving unsafe handle management, raw pointer casting, and variable-length buffer allocation. Getting any of these steps wrong can lead to undefined behavior, resource leaks, or incorrect security decisions.

`token-privilege` encapsulates all of that complexity behind four simple functions and a set of well-known privilege name constants.

## Key Features

- **Safe public API** -- all `unsafe` code is confined to a single internal module (`ffi.rs`).
- **RAII handle management** -- Win32 `HANDLE` values are wrapped in a drop guard that calls `CloseHandle` automatically.
- **Cross-platform friendly** -- on non-Windows platforms, every public function returns `Err(TokenPrivilegeError::UnsupportedPlatform)`, allowing unconditional dependency without `#[cfg]` at the call site.
- **Read-only** -- the crate never modifies token privileges; it only queries them.
- **Strict linting** -- `clippy::unwrap_used` and `clippy::panic` are denied; every `unsafe` block requires a `// SAFETY:` comment.

## Quick Example

```rust,no_run
use token_privilege::{is_elevated, is_privilege_enabled, privileges};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if is_elevated()? {
        println!("Running as Administrator");
    }

    if is_privilege_enabled(privileges::SE_DEBUG)? {
        println!("SeDebugPrivilege is enabled");
    }

    Ok(())
}
```

## License

Dual-licensed under [MIT](https://opensource.org/licenses/MIT) or [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) at your option.

## Repository

Source code and issue tracker: <https://github.com/EvilBit-Labs/token-privilege>
