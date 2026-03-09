# API Reference

This page documents every public item exported by `token-privilege`. For the auto-generated rustdoc, see <https://docs.rs/token-privilege>.

[TOC]

## Functions

### `is_elevated`

```rust,ignore
pub fn is_elevated() -> Result<bool, TokenPrivilegeError>
```

Check if the current process is running with elevated (Administrator) privileges.

Returns `true` if the process token has `TokenElevationTypeFull` (elevated via UAC) or `TokenElevationTypeDefault` (UAC disabled and user is an admin).

**Errors:**

- `TokenPrivilegeError::UnsupportedPlatform` on non-Windows.
- `TokenPrivilegeError::OpenTokenFailed` if the process token cannot be opened.
- `TokenPrivilegeError::QueryFailed` if the elevation query fails.

**Example:**

```rust,no_run
use token_privilege::is_elevated;

let elevated = is_elevated()?;
println!("Elevated: {elevated}");
# Ok::<(), Box<dyn std::error::Error>>(())
```

---

### `is_privilege_enabled`

```rust,ignore
pub fn is_privilege_enabled(privilege_name: &str) -> Result<bool, TokenPrivilegeError>
```

Check if a specific named privilege is **present and enabled** on the current process token.

**Arguments:**

- `privilege_name` -- the Windows privilege name (e.g., `"SeDebugPrivilege"`). Use constants from the [`privileges`](#privileges-module) module.

**Errors:**

- `TokenPrivilegeError::UnsupportedPlatform` on non-Windows.
- `TokenPrivilegeError::InvalidPrivilegeName` if the name is not recognized.
- `TokenPrivilegeError::LookupFailed` if the OS-level lookup fails.
- `TokenPrivilegeError::CheckFailed` if `PrivilegeCheck` fails.

**Example:**

```rust,no_run
use token_privilege::{is_privilege_enabled, privileges};

if is_privilege_enabled(privileges::SE_CHANGE_NOTIFY)? {
    println!("SeChangeNotifyPrivilege is enabled");
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

---

### `has_privilege`

```rust,ignore
pub fn has_privilege(privilege_name: &str) -> Result<bool, TokenPrivilegeError>
```

Check if a specific named privilege is **present** on the current process token, regardless of whether it is currently enabled.

This function enumerates all token privileges and checks whether the named privilege appears in the list.

**Arguments:**

- `privilege_name` -- the Windows privilege name.

**Errors:**

- `TokenPrivilegeError::UnsupportedPlatform` on non-Windows.
- `TokenPrivilegeError::InvalidPrivilegeName` if the name is not recognized.
- `TokenPrivilegeError::QueryFailed` if token enumeration fails.

**Example:**

```rust,no_run
use token_privilege::{has_privilege, privileges};

if has_privilege(privileges::SE_BACKUP)? {
    println!("SeBackupPrivilege is on the token");
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

---

### `enumerate_privileges`

```rust,ignore
pub fn enumerate_privileges() -> Result<Vec<PrivilegeInfo>, TokenPrivilegeError>
```

Enumerate all privileges on the current process token.

Returns a `Vec<PrivilegeInfo>` describing each privilege, its name, and its current status flags.

**Errors:**

- `TokenPrivilegeError::UnsupportedPlatform` on non-Windows.
- `TokenPrivilegeError::OpenTokenFailed` if the process token cannot be opened.
- `TokenPrivilegeError::QueryFailed` if privilege enumeration fails.

**Example:**

```rust,no_run
use token_privilege::enumerate_privileges;

for info in enumerate_privileges()? {
    println!("{}: enabled={}", info.name, info.enabled);
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

---

## Types

### `PrivilegeInfo`

```rust,ignore
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PrivilegeInfo {
    pub name: String,
    pub enabled: bool,
    pub enabled_by_default: bool,
    pub removed: bool,
}
```

Represents the status of a single Windows privilege on the process token.

| Field                | Type     | Description                                            |
| -------------------- | -------- | ------------------------------------------------------ |
| `name`               | `String` | The privilege name (e.g., `"SeDebugPrivilege"`).       |
| `enabled`            | `bool`   | Whether the privilege is currently enabled.            |
| `enabled_by_default` | `bool`   | Whether the privilege is enabled by default.           |
| `removed`            | `bool`   | Whether the privilege has been removed from the token. |

---

### `TokenPrivilegeError`

```rust,ignore
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TokenPrivilegeError {
    OpenTokenFailed(std::io::Error),
    QueryFailed(std::io::Error),
    InvalidPrivilegeName { name: String },
    LookupFailed { name: String, source: std::io::Error },
    CheckFailed(std::io::Error),
    UnsupportedPlatform,
}
```

All functions in this crate return this error type. The enum is `#[non_exhaustive]` -- always include a wildcard arm when matching.

| Variant                | When It Occurs                                      |
| ---------------------- | --------------------------------------------------- |
| `OpenTokenFailed`      | `OpenProcessToken` fails.                           |
| `QueryFailed`          | `GetTokenInformation` fails.                        |
| `InvalidPrivilegeName` | The privilege name is not recognized by Windows.    |
| `LookupFailed`         | `LookupPrivilegeValueW` fails for OS-level reasons. |
| `CheckFailed`          | `PrivilegeCheck` fails.                             |
| `UnsupportedPlatform`  | Called on a non-Windows platform.                   |

---

## `privileges` Module

The `privileges` module provides well-known Windows privilege name constants. Use these instead of hard-coding string literals.

| Constant                    | Value                               | Description                                |
| --------------------------- | ----------------------------------- | ------------------------------------------ |
| `SE_DEBUG`                  | `"SeDebugPrivilege"`                | Debug programs.                            |
| `SE_BACKUP`                 | `"SeBackupPrivilege"`               | Back up files and directories.             |
| `SE_RESTORE`                | `"SeRestorePrivilege"`              | Restore files and directories.             |
| `SE_SHUTDOWN`               | `"SeShutdownPrivilege"`             | Shut down the system.                      |
| `SE_SECURITY`               | `"SeSecurityPrivilege"`             | Manage auditing and security log.          |
| `SE_TAKE_OWNERSHIP`         | `"SeTakeOwnershipPrivilege"`        | Take ownership of files or other objects.  |
| `SE_LOAD_DRIVER`            | `"SeLoadDriverPrivilege"`           | Load and unload device drivers.            |
| `SE_SYSTEM_PROFILE`         | `"SeSystemProfilePrivilege"`        | Profile system performance.                |
| `SE_SYSTEMTIME`             | `"SeSystemtimePrivilege"`           | Change the system time.                    |
| `SE_CHANGE_NOTIFY`          | `"SeChangeNotifyPrivilege"`         | Bypass traverse checking.                  |
| `SE_IMPERSONATE`            | `"SeImpersonatePrivilege"`          | Impersonate a client after authentication. |
| `SE_CREATE_GLOBAL`          | `"SeCreateGlobalPrivilege"`         | Create global objects.                     |
| `SE_INCREASE_QUOTA`         | `"SeIncreaseQuotaPrivilege"`        | Adjust memory quotas for a process.        |
| `SE_UNDOCK`                 | `"SeUndockPrivilege"`               | Remove computer from docking station.      |
| `SE_MANAGE_VOLUME`          | `"SeManageVolumePrivilege"`         | Perform volume maintenance tasks.          |
| `SE_ASSIGN_PRIMARY_TOKEN`   | `"SeAssignPrimaryTokenPrivilege"`   | Replace a process-level token.             |
| `SE_INCREASE_BASE_PRIORITY` | `"SeIncreaseBasePriorityPrivilege"` | Increase scheduling priority.              |
| `SE_CREATE_PAGEFILE`        | `"SeCreatePagefilePrivilege"`       | Create a pagefile.                         |
| `SE_TCB`                    | `"SeTcbPrivilege"`                  | Act as part of the operating system.       |
| `SE_REMOTE_SHUTDOWN`        | `"SeRemoteShutdownPrivilege"`       | Force shutdown from a remote system.       |

**Example:**

```rust,no_run
use token_privilege::{is_privilege_enabled, privileges};

let debug_enabled = is_privilege_enabled(privileges::SE_DEBUG)?;
let backup_enabled = is_privilege_enabled(privileges::SE_BACKUP)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```
