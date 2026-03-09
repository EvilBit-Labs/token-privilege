//! Privilege querying and enumeration for the current process token.

use crate::PrivilegeInfo;
use crate::error::TokenPrivilegeError;
use crate::ffi;

/// Check if a specific named privilege is present and enabled on the current process token.
///
/// # Arguments
///
/// * `privilege_name` — The Windows privilege name (e.g., `"SeDebugPrivilege"`).
///
/// # Errors
///
/// Returns an error if the process token cannot be opened, the privilege name
/// is invalid, or the privilege check fails.
pub fn is_privilege_enabled(privilege_name: &str) -> Result<bool, TokenPrivilegeError> {
    let token = ffi::open_current_process_token()?;
    let luid = ffi::lookup_privilege_value(privilege_name)?;
    ffi::check_privilege_enabled(&token, luid)
}

/// Check if a specific named privilege is present on the current process token,
/// regardless of whether it is currently enabled.
///
/// # Errors
///
/// Returns an error if the process token cannot be opened or the privilege
/// name is invalid.
pub fn has_privilege(privilege_name: &str) -> Result<bool, TokenPrivilegeError> {
    let token = ffi::open_current_process_token()?;
    // Validate the privilege name exists before enumerating
    let _luid = ffi::lookup_privilege_value(privilege_name)?;

    let privileges = ffi::enumerate_token_privileges(&token)?;
    Ok(privileges.iter().any(|p| p.name == privilege_name))
}

/// Enumerate all privileges on the current process token.
///
/// Returns a list of [`PrivilegeInfo`] describing each privilege, its name,
/// and its current status.
///
/// # Errors
///
/// Returns an error if the process token cannot be opened or privileges
/// cannot be enumerated.
pub fn enumerate_privileges() -> Result<Vec<PrivilegeInfo>, TokenPrivilegeError> {
    let token = ffi::open_current_process_token()?;
    ffi::enumerate_token_privileges(&token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_notify_is_enabled() {
        let result = is_privilege_enabled("SeChangeNotifyPrivilege");
        assert!(result.is_ok(), "should succeed");
        if let Ok(is_enabled) = result {
            assert!(is_enabled, "SeChangeNotifyPrivilege should be enabled");
        }
    }

    #[test]
    fn has_change_notify_privilege() {
        let result = has_privilege("SeChangeNotifyPrivilege");
        assert!(result.is_ok(), "should succeed");
        if let Ok(has_priv) = result {
            assert!(has_priv, "SeChangeNotifyPrivilege should be present");
        }
    }

    #[test]
    fn invalid_privilege_name_returns_error() {
        let result = is_privilege_enabled("SeTotallyFakePrivilege");
        assert!(result.is_err(), "should fail for invalid privilege");
    }

    #[test]
    fn enumerate_privileges_non_empty() {
        let result = enumerate_privileges();
        assert!(result.is_ok(), "enumeration should succeed");
        if let Ok(privs) = result {
            assert!(!privs.is_empty(), "should have at least one privilege");
        }
    }

    #[test]
    fn enumerate_contains_change_notify() {
        let result = enumerate_privileges();
        assert!(result.is_ok(), "enumeration should succeed");
        if let Ok(privs) = result {
            let found = privs.iter().any(|p| p.name == "SeChangeNotifyPrivilege");
            assert!(found, "should contain SeChangeNotifyPrivilege");
        }
    }

    #[test]
    fn change_notify_is_enabled_in_enumeration() {
        let result = enumerate_privileges();
        assert!(result.is_ok(), "enumeration should succeed");
        if let Ok(privs) = result {
            let cn = privs.iter().find(|p| p.name == "SeChangeNotifyPrivilege");
            assert!(cn.is_some(), "should find SeChangeNotifyPrivilege");
            if let Some(change_notify) = cn {
                assert!(
                    change_notify.enabled,
                    "SeChangeNotifyPrivilege should be enabled"
                );
            }
        }
    }
}
