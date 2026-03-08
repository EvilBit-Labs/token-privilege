# Getting Started

## Requirements

- Rust 1.85 or later (the crate uses the 2024 edition)
- Windows for actual privilege queries (Linux and macOS are supported but all functions return `Err(TokenPrivilegeError::UnsupportedPlatform)`)

## Installation

Add `token-privilege` to your project:

```bash
cargo add token-privilege
```

Or add it manually to your `Cargo.toml`:

```toml
[dependencies]
token-privilege = "0.1"
```

## Basic Usage

### Check Elevation Status

Determine whether the current process is running with Administrator privileges (elevated via UAC):

```rust,no_run
use token_privilege::is_elevated;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match is_elevated()? {
        true  => println!("Process is elevated (Administrator)"),
        false => println!("Process is NOT elevated"),
    }
    Ok(())
}
```

### Check a Specific Privilege

Use `is_privilege_enabled` with a constant from the `privileges` module to check whether a specific privilege is currently enabled on the process token:

```rust,no_run
use token_privilege::{is_privilege_enabled, privileges};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if is_privilege_enabled(privileges::SE_DEBUG)? {
        println!("SeDebugPrivilege is enabled -- process can debug other processes");
    } else {
        println!("SeDebugPrivilege is NOT enabled");
    }
    Ok(())
}
```

### Check Privilege Presence

`has_privilege` checks whether a privilege exists on the token regardless of whether it is currently enabled:

```rust,no_run
use token_privilege::{has_privilege, privileges};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if has_privilege(privileges::SE_BACKUP)? {
        println!("SeBackupPrivilege is present on the token (may or may not be enabled)");
    }
    Ok(())
}
```

### Enumerate All Privileges

List every privilege on the current process token along with its status:

```rust,no_run
use token_privilege::enumerate_privileges;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let privileges = enumerate_privileges()?;
    for info in &privileges {
        println!(
            "{}: enabled={}, enabled_by_default={}, removed={}",
            info.name, info.enabled, info.enabled_by_default, info.removed
        );
    }
    Ok(())
}
```

## Error Handling

All functions return `Result<T, TokenPrivilegeError>`. The error type is `#[non_exhaustive]`, so always include a wildcard arm when matching:

```rust
use token_privilege::{is_elevated, TokenPrivilegeError};

fn check() {
    match is_elevated() {
        Ok(true)  => println!("Elevated"),
        Ok(false) => println!("Not elevated"),
        Err(TokenPrivilegeError::UnsupportedPlatform) => {
            println!("Not running on Windows");
        }
        Err(e) => eprintln!("Unexpected error: {e}"),
    }
}
```
