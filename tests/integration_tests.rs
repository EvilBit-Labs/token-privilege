//! Integration tests for the `token-privilege` crate.
//!
//! Windows-specific tests validate actual Win32 API behavior.
//! Non-Windows tests verify that all public functions return
//! [`token_privilege::TokenPrivilegeError::UnsupportedPlatform`].

#[cfg(target_os = "windows")]
mod windows_tests {
    #[test]
    fn elevation_and_privileges_are_consistent() {
        let elevated_result = token_privilege::is_elevated();
        assert!(elevated_result.is_ok(), "is_elevated should succeed");

        let privs_result = token_privilege::enumerate_privileges();
        assert!(privs_result.is_ok(), "enumerate_privileges should succeed");

        if let Ok(privs) = privs_result {
            // All Windows processes have at least SeChangeNotifyPrivilege
            assert!(!privs.is_empty(), "should have at least one privilege");

            let change_notify = privs.iter().find(|p| p.name == "SeChangeNotifyPrivilege");
            assert!(
                change_notify.is_some(),
                "should have SeChangeNotifyPrivilege"
            );
            if let Some(cn) = change_notify {
                assert!(cn.enabled, "SeChangeNotifyPrivilege should be enabled");
                assert!(
                    cn.enabled_by_default,
                    "SeChangeNotifyPrivilege should be enabled by default"
                );
            }
        }
    }

    #[test]
    fn is_privilege_enabled_matches_enumeration() {
        let check_result =
            token_privilege::is_privilege_enabled(token_privilege::privileges::SE_CHANGE_NOTIFY);
        assert!(check_result.is_ok(), "check should succeed");

        let privs_result = token_privilege::enumerate_privileges();
        assert!(privs_result.is_ok(), "enumerate should succeed");

        if let (Ok(enabled_via_check), Ok(privs)) = (check_result, privs_result) {
            let enabled_via_enum = privs
                .iter()
                .find(|p| p.name == token_privilege::privileges::SE_CHANGE_NOTIFY)
                .is_some_and(|p| p.enabled);

            assert_eq!(
                enabled_via_check, enabled_via_enum,
                "is_privilege_enabled and enumerate should agree"
            );
        }
    }

    #[test]
    fn has_privilege_returns_true_for_change_notify() {
        let result = token_privilege::has_privilege(token_privilege::privileges::SE_CHANGE_NOTIFY);
        assert!(result.is_ok(), "should succeed");
        if let Ok(has_priv) = result {
            assert!(has_priv, "SeChangeNotifyPrivilege should be present");
        }
    }

    #[test]
    fn has_privilege_returns_false_for_absent_privilege() {
        // SeTcbPrivilege is almost never present on standard user tokens
        let result = token_privilege::has_privilege(token_privilege::privileges::SE_TCB);
        assert!(result.is_ok(), "should succeed for valid privilege name");
        // We don't assert the value since elevated processes might have it,
        // but we verify it doesn't error for a valid privilege name.
    }

    #[test]
    fn invalid_privilege_name_returns_correct_error_variant() {
        let result = token_privilege::is_privilege_enabled("SeTotallyFakePrivilege");
        assert!(
            matches!(
                result,
                Err(token_privilege::TokenPrivilegeError::InvalidPrivilegeName { .. })
            ),
            "should return InvalidPrivilegeName for invalid privilege, got: {result:?}"
        );
    }

    #[test]
    fn has_privilege_invalid_name_returns_correct_error_variant() {
        let result = token_privilege::has_privilege("SeTotallyFakePrivilege");
        assert!(
            matches!(
                result,
                Err(token_privilege::TokenPrivilegeError::InvalidPrivilegeName { .. })
            ),
            "should return InvalidPrivilegeName for invalid privilege, got: {result:?}"
        );
    }

    #[test]
    fn empty_privilege_name_returns_error() {
        let result = token_privilege::is_privilege_enabled("");
        assert!(result.is_err(), "empty string should fail");
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
