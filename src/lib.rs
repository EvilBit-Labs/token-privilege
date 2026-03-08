//! Safe Rust wrapper for Windows process token privilege and elevation detection.
//!
//! All unsafe Win32 FFI calls are contained within the crate. Consumers use a fully
//! safe public API and can maintain `#![forbid(unsafe_code)]` in their own crates.
//!
//! On non-Windows platforms, all functions return
//! <code>Err([TokenPrivilegeError::UnsupportedPlatform])</code>.
//!
//! # Examples
//!
//! ```rust,no_run
//! use token_privilege::{is_elevated, is_privilege_enabled, privileges};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     if is_elevated()? {
//!         println!("Running as Administrator");
//!     }
//!
//!     if is_privilege_enabled(privileges::SE_DEBUG)? {
//!         println!("SeDebugPrivilege is enabled");
//!     }
//!
//!     Ok(())
//! }
//! ```

mod error;

#[cfg(target_os = "windows")]
mod elevation;
#[cfg(target_os = "windows")]
mod ffi;
#[cfg(target_os = "windows")]
mod privilege;

pub use error::TokenPrivilegeError;

/// Represents the status of a single Windows privilege.
///
/// Returned by [`enumerate_privileges`] to describe each privilege on the
/// current process token, including its name and current state.
///
/// # Examples
///
/// ```rust,no_run
/// use token_privilege::enumerate_privileges;
///
/// let privileges = enumerate_privileges()?;
/// for priv_info in &privileges {
///     println!("{}: enabled={}", priv_info.name, priv_info.enabled);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivilegeInfo {
    /// The privilege name (e.g., `"SeDebugPrivilege"`).
    pub name: String,
    /// Whether the privilege is currently enabled.
    pub enabled: bool,
    /// Whether the privilege is enabled by default.
    pub enabled_by_default: bool,
    /// Whether the privilege has been removed from the token.
    pub removed: bool,
}

/// Well-known Windows privilege name constants.
///
/// Use these with [`is_privilege_enabled`] and [`has_privilege`] to avoid
/// hard-coding privilege name strings.
pub mod privileges {
    /// Debug programs (`SeDebugPrivilege`).
    pub const SE_DEBUG: &str = "SeDebugPrivilege";
    /// Back up files and directories (`SeBackupPrivilege`).
    pub const SE_BACKUP: &str = "SeBackupPrivilege";
    /// Restore files and directories (`SeRestorePrivilege`).
    pub const SE_RESTORE: &str = "SeRestorePrivilege";
    /// Shut down the system (`SeShutdownPrivilege`).
    pub const SE_SHUTDOWN: &str = "SeShutdownPrivilege";
    /// Manage auditing and security log (`SeSecurityPrivilege`).
    pub const SE_SECURITY: &str = "SeSecurityPrivilege";
    /// Take ownership of files or other objects (`SeTakeOwnershipPrivilege`).
    pub const SE_TAKE_OWNERSHIP: &str = "SeTakeOwnershipPrivilege";
    /// Load and unload device drivers (`SeLoadDriverPrivilege`).
    pub const SE_LOAD_DRIVER: &str = "SeLoadDriverPrivilege";
    /// Profile system performance (`SeSystemProfilePrivilege`).
    pub const SE_SYSTEM_PROFILE: &str = "SeSystemProfilePrivilege";
    /// Change the system time (`SeSystemtimePrivilege`).
    pub const SE_SYSTEMTIME: &str = "SeSystemtimePrivilege";
    /// Bypass traverse checking (`SeChangeNotifyPrivilege`).
    pub const SE_CHANGE_NOTIFY: &str = "SeChangeNotifyPrivilege";
    /// Impersonate a client after authentication (`SeImpersonatePrivilege`).
    pub const SE_IMPERSONATE: &str = "SeImpersonatePrivilege";
    /// Create global objects (`SeCreateGlobalPrivilege`).
    pub const SE_CREATE_GLOBAL: &str = "SeCreateGlobalPrivilege";
    /// Adjust memory quotas for a process (`SeIncreaseQuotaPrivilege`).
    pub const SE_INCREASE_QUOTA: &str = "SeIncreaseQuotaPrivilege";
    /// Remove computer from docking station (`SeUndockPrivilege`).
    pub const SE_UNDOCK: &str = "SeUndockPrivilege";
    /// Perform volume maintenance tasks (`SeManageVolumePrivilege`).
    pub const SE_MANAGE_VOLUME: &str = "SeManageVolumePrivilege";
    /// Replace a process-level token (`SeAssignPrimaryTokenPrivilege`).
    pub const SE_ASSIGN_PRIMARY_TOKEN: &str = "SeAssignPrimaryTokenPrivilege";
    /// Increase scheduling priority (`SeIncreaseBasePriorityPrivilege`).
    pub const SE_INCREASE_BASE_PRIORITY: &str = "SeIncreaseBasePriorityPrivilege";
    /// Create a pagefile (`SeCreatePagefilePrivilege`).
    pub const SE_CREATE_PAGEFILE: &str = "SeCreatePagefilePrivilege";
    /// Act as part of the operating system (`SeTcbPrivilege`).
    pub const SE_TCB: &str = "SeTcbPrivilege";
    /// Force shutdown from a remote system (`SeRemoteShutdownPrivilege`).
    pub const SE_REMOTE_SHUTDOWN: &str = "SeRemoteShutdownPrivilege";
}

// ─── Windows implementation (delegates to domain modules) ───

/// Check if the current process is running with elevated (Administrator) privileges.
///
/// Returns `true` if the process token has elevation type `TokenElevationTypeFull`
/// (elevated via UAC) or `TokenElevationTypeDefault` (UAC disabled, user is admin).
///
/// # Errors
///
/// Returns [`TokenPrivilegeError::UnsupportedPlatform`] on non-Windows.
/// Returns an error if the process token cannot be opened or queried.
#[cfg(target_os = "windows")]
pub fn is_elevated() -> Result<bool, TokenPrivilegeError> {
    elevation::is_elevated()
}

/// Check if a specific named privilege is present and enabled on the current process token.
///
/// # Arguments
///
/// * `privilege_name` — The Windows privilege name (e.g., `"SeDebugPrivilege"`).
///   Use constants from the [`privileges`] module.
///
/// # Errors
///
/// Returns [`TokenPrivilegeError::UnsupportedPlatform`] on non-Windows.
/// Returns an error if the privilege name is invalid or the check fails.
#[cfg(target_os = "windows")]
pub fn is_privilege_enabled(privilege_name: &str) -> Result<bool, TokenPrivilegeError> {
    privilege::is_privilege_enabled(privilege_name)
}

/// Check if a specific named privilege is present on the current process token,
/// regardless of whether it is currently enabled.
///
/// # Errors
///
/// Returns [`TokenPrivilegeError::UnsupportedPlatform`] on non-Windows.
/// Returns an error if the privilege name is invalid.
#[cfg(target_os = "windows")]
pub fn has_privilege(privilege_name: &str) -> Result<bool, TokenPrivilegeError> {
    privilege::has_privilege(privilege_name)
}

/// Enumerate all privileges on the current process token.
///
/// Returns a list of [`PrivilegeInfo`] describing each privilege, its name,
/// and its current status.
///
/// # Errors
///
/// Returns [`TokenPrivilegeError::UnsupportedPlatform`] on non-Windows.
/// Returns an error if the process token cannot be opened or enumerated.
#[cfg(target_os = "windows")]
pub fn enumerate_privileges() -> Result<Vec<PrivilegeInfo>, TokenPrivilegeError> {
    privilege::enumerate_privileges()
}

// ─── Non-Windows stubs ───

/// Check if the current process is running with elevated (Administrator) privileges.
///
/// # Platform Support
///
/// This is a non-Windows stub that always returns an error. On Windows, this
/// queries the process token for UAC elevation status.
///
/// # Errors
///
/// Always returns [`TokenPrivilegeError::UnsupportedPlatform`] on non-Windows.
#[cfg(not(target_os = "windows"))]
pub const fn is_elevated() -> Result<bool, TokenPrivilegeError> {
    Err(TokenPrivilegeError::UnsupportedPlatform)
}

/// Check if a specific named privilege is present and enabled on the current process token.
///
/// # Platform Support
///
/// This is a non-Windows stub that always returns an error. On Windows, this
/// looks up the privilege by name and checks if it is enabled.
///
/// # Errors
///
/// Always returns [`TokenPrivilegeError::UnsupportedPlatform`] on non-Windows.
#[cfg(not(target_os = "windows"))]
pub const fn is_privilege_enabled(_privilege_name: &str) -> Result<bool, TokenPrivilegeError> {
    Err(TokenPrivilegeError::UnsupportedPlatform)
}

/// Check if a specific named privilege is present on the current process token,
/// regardless of whether it is currently enabled.
///
/// # Platform Support
///
/// This is a non-Windows stub that always returns an error. On Windows, this
/// enumerates token privileges and checks for the named privilege.
///
/// # Errors
///
/// Always returns [`TokenPrivilegeError::UnsupportedPlatform`] on non-Windows.
#[cfg(not(target_os = "windows"))]
pub const fn has_privilege(_privilege_name: &str) -> Result<bool, TokenPrivilegeError> {
    Err(TokenPrivilegeError::UnsupportedPlatform)
}

/// Enumerate all privileges on the current process token.
///
/// # Platform Support
///
/// This is a non-Windows stub that always returns an error. On Windows, this
/// returns a [`Vec<PrivilegeInfo>`] with each privilege's name and status.
///
/// # Errors
///
/// Always returns [`TokenPrivilegeError::UnsupportedPlatform`] on non-Windows.
#[cfg(not(target_os = "windows"))]
pub const fn enumerate_privileges() -> Result<Vec<PrivilegeInfo>, TokenPrivilegeError> {
    Err(TokenPrivilegeError::UnsupportedPlatform)
}

#[cfg(not(target_os = "windows"))]
#[cfg(test)]
mod stub_tests {
    use super::*;

    #[test]
    fn is_elevated_returns_unsupported() {
        assert!(matches!(
            is_elevated(),
            Err(TokenPrivilegeError::UnsupportedPlatform)
        ));
    }

    #[test]
    fn is_privilege_enabled_returns_unsupported() {
        assert!(matches!(
            is_privilege_enabled("SeDebugPrivilege"),
            Err(TokenPrivilegeError::UnsupportedPlatform)
        ));
    }

    #[test]
    fn has_privilege_returns_unsupported() {
        assert!(matches!(
            has_privilege("SeDebugPrivilege"),
            Err(TokenPrivilegeError::UnsupportedPlatform)
        ));
    }

    #[test]
    fn enumerate_privileges_returns_unsupported() {
        assert!(matches!(
            enumerate_privileges(),
            Err(TokenPrivilegeError::UnsupportedPlatform)
        ));
    }
}
