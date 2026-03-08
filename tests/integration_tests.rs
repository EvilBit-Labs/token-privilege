//! Integration tests for the `token-privilege` crate.
//!
//! Windows-specific tests validate actual Win32 API behavior.
//! Non-Windows tests verify that all public functions return
//! [`token_privilege::TokenPrivilegeError::UnsupportedPlatform`].

#[cfg(target_os = "windows")]
mod windows_tests {
    #[test]
    fn elevation_and_privileges_are_consistent() {
        let _elevated = token_privilege::is_elevated().expect("is_elevated should succeed");
        let privs =
            token_privilege::enumerate_privileges().expect("enumerate_privileges should succeed");

        // All Windows processes have at least SeChangeNotifyPrivilege
        assert!(!privs.is_empty(), "should have at least one privilege");

        let change_notify = privs.iter().find(|p| p.name == "SeChangeNotifyPrivilege");
        assert!(
            change_notify.is_some(),
            "should have SeChangeNotifyPrivilege"
        );
        assert!(
            change_notify.expect("found").enabled,
            "SeChangeNotifyPrivilege should be enabled"
        );
    }

    #[test]
    fn is_privilege_enabled_matches_enumeration() {
        let enabled_via_check =
            token_privilege::is_privilege_enabled(token_privilege::privileges::SE_CHANGE_NOTIFY)
                .expect("check should succeed");

        let privs = token_privilege::enumerate_privileges().expect("enumerate should succeed");
        let enabled_via_enum = privs
            .iter()
            .find(|p| p.name == token_privilege::privileges::SE_CHANGE_NOTIFY)
            .map_or(false, |p| p.enabled);

        assert_eq!(
            enabled_via_check, enabled_via_enum,
            "is_privilege_enabled and enumerate should agree"
        );
    }

    #[test]
    fn has_privilege_returns_true_for_change_notify() {
        let result = token_privilege::has_privilege(token_privilege::privileges::SE_CHANGE_NOTIFY);
        assert!(result.is_ok(), "should succeed");
        assert!(
            result.expect("checked"),
            "SeChangeNotifyPrivilege should be present"
        );
    }

    #[test]
    fn privilege_constants_are_valid_names() {
        let constants = [
            token_privilege::privileges::SE_CHANGE_NOTIFY,
            token_privilege::privileges::SE_SHUTDOWN,
            token_privilege::privileges::SE_UNDOCK,
        ];

        for name in constants {
            let result = token_privilege::has_privilege(name);
            assert!(
                result.is_ok(),
                "has_privilege({name}) should not error for valid privilege names"
            );
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod non_windows_tests {
    #[test]
    fn is_elevated_returns_unsupported() {
        assert!(matches!(
            token_privilege::is_elevated(),
            Err(token_privilege::TokenPrivilegeError::UnsupportedPlatform)
        ));
    }

    #[test]
    fn is_privilege_enabled_returns_unsupported() {
        assert!(matches!(
            token_privilege::is_privilege_enabled("SeDebugPrivilege"),
            Err(token_privilege::TokenPrivilegeError::UnsupportedPlatform)
        ));
    }

    #[test]
    fn has_privilege_returns_unsupported() {
        assert!(matches!(
            token_privilege::has_privilege("SeDebugPrivilege"),
            Err(token_privilege::TokenPrivilegeError::UnsupportedPlatform)
        ));
    }

    #[test]
    fn enumerate_privileges_returns_unsupported() {
        assert!(matches!(
            token_privilege::enumerate_privileges(),
            Err(token_privilege::TokenPrivilegeError::UnsupportedPlatform)
        ));
    }

    #[test]
    fn privilege_constants_exist() {
        // Verify constants are available even on non-Windows
        let _ = token_privilege::privileges::SE_DEBUG;
        let _ = token_privilege::privileges::SE_BACKUP;
        let _ = token_privilege::privileges::SE_CHANGE_NOTIFY;
        let _ = token_privilege::privileges::SE_SHUTDOWN;
        let _ = token_privilege::privileges::SE_SECURITY;
        let _ = token_privilege::privileges::SE_TAKE_OWNERSHIP;
        let _ = token_privilege::privileges::SE_LOAD_DRIVER;
        let _ = token_privilege::privileges::SE_IMPERSONATE;
        let _ = token_privilege::privileges::SE_CREATE_GLOBAL;
        let _ = token_privilege::privileges::SE_TCB;
    }
}
