# Integration Examples

This page shows common patterns for integrating `token-privilege` into your Rust projects.

[TOC]

## Using from a `#![forbid(unsafe_code)]` Crate

The primary design goal of `token-privilege` is to let downstream consumers remain fully safe. Your crate can forbid unsafe code while still querying Windows process tokens:

```rust,ignore
// your_crate/src/lib.rs
#![forbid(unsafe_code)]

use token_privilege::{is_elevated, TokenPrivilegeError};

pub fn require_admin() -> Result<(), String> {
    match is_elevated() {
        Ok(true) => Ok(()),
        Ok(false) => Err("This operation requires Administrator privileges.".into()),
        Err(TokenPrivilegeError::UnsupportedPlatform) => {
            // Not on Windows -- skip the check or use a platform-specific alternative
            Ok(())
        }
        Err(e) => Err(format!("Failed to check elevation: {e}")),
    }
}
```

No `unsafe` block is needed in your code. All FFI interactions happen inside `token-privilege`'s internal `ffi.rs` module.

## Cross-Platform Applications

`token-privilege` compiles on all platforms. On non-Windows targets, every public function returns `Err(TokenPrivilegeError::UnsupportedPlatform)`. This means you can depend on the crate unconditionally and handle the error at runtime:

```rust,no_run
use token_privilege::{is_elevated, TokenPrivilegeError};

fn check_elevation() -> bool {
    match is_elevated() {
        Ok(elevated) => elevated,
        Err(TokenPrivilegeError::UnsupportedPlatform) => {
            // On Linux/macOS, fall back to a different check
            #[cfg(unix)]
            {
                // Example: check effective UID on Unix
                unsafe { libc::geteuid() == 0 }
            }
            #[cfg(not(unix))]
            {
                false
            }
        }
        Err(e) => {
            eprintln!("Elevation check failed: {e}");
            false
        }
    }
}
```

Alternatively, use compile-time `#[cfg]` gating if you want to avoid the runtime error path entirely:

```rust,no_run
fn is_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        token_privilege::is_elevated().unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}
```

## Privilege Guard Pattern

Create a guard that checks for required privileges before performing a sensitive operation:

```rust,ignore
use token_privilege::{is_privilege_enabled, has_privilege, privileges, TokenPrivilegeError};

#[derive(Debug)]
pub enum PrivilegeStatus {
    Enabled,
    PresentButDisabled,
    Missing,
}

pub fn check_privilege_status(name: &str) -> Result<PrivilegeStatus, TokenPrivilegeError> {
    if is_privilege_enabled(name)? {
        Ok(PrivilegeStatus::Enabled)
    } else if has_privilege(name)? {
        Ok(PrivilegeStatus::PresentButDisabled)
    } else {
        Ok(PrivilegeStatus::Missing)
    }
}

pub fn require_privilege(name: &str) -> Result<(), String> {
    match check_privilege_status(name) {
        Ok(PrivilegeStatus::Enabled) => Ok(()),
        Ok(PrivilegeStatus::PresentButDisabled) => {
            Err(format!("{name} is present but not enabled. Run as Administrator."))
        }
        Ok(PrivilegeStatus::Missing) => {
            Err(format!("{name} is not available on this token."))
        }
        Err(e) => Err(format!("Privilege check failed: {e}")),
    }
}
```

## Diagnostic Reporting

Enumerate all privileges and produce a diagnostic report, useful for troubleshooting or security audits:

```rust,no_run
use token_privilege::{enumerate_privileges, is_elevated};

fn print_token_report() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Process Token Report ===");
    println!("Elevated: {}", is_elevated()?);
    println!();

    let privileges = enumerate_privileges()?;
    println!("{:<40} {:>8} {:>10} {:>8}", "Privilege", "Enabled", "Default", "Removed");
    println!("{}", "-".repeat(70));

    for info in &privileges {
        println!(
            "{:<40} {:>8} {:>10} {:>8}",
            info.name, info.enabled, info.enabled_by_default, info.removed
        );
    }

    println!();
    println!("Total privileges: {}", privileges.len());
    let enabled_count = privileges.iter().filter(|p| p.enabled).count();
    println!("Enabled: {enabled_count}");

    Ok(())
}
# fn main() { let _ = print_token_report(); }
```

## Conditional Feature Gating

Use privilege checks to enable or disable features in your application:

```rust,ignore
use token_privilege::{is_elevated, is_privilege_enabled, privileges};

pub struct AppCapabilities {
    pub can_debug_processes: bool,
    pub can_backup_files: bool,
    pub is_admin: bool,
}

pub fn detect_capabilities() -> AppCapabilities {
    AppCapabilities {
        can_debug_processes: is_privilege_enabled(privileges::SE_DEBUG).unwrap_or(false),
        can_backup_files: is_privilege_enabled(privileges::SE_BACKUP).unwrap_or(false),
        is_admin: is_elevated().unwrap_or(false),
    }
}
```
