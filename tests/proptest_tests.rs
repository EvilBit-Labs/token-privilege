//! Property-based tests for the `token-privilege` crate.
//!
//! Uses `proptest` to generate random and adversarial privilege name strings,
//! verifying the crate never panics and always returns a well-formed `Result`.

use proptest::prelude::*;

/// Strategy for generating arbitrary strings including edge cases:
/// empty, whitespace, Unicode, very long, null bytes.
fn privilege_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Completely arbitrary Unicode strings
        "\\PC*",
        // ASCII-only strings (privilege names are ASCII in practice)
        "[a-zA-Z0-9_]{0,100}",
        // Empty string
        Just(String::new()),
        // Strings with embedded null bytes
        "[a-zA-Z]*\0[a-zA-Z]*",
        // Very long strings
        "[a-zA-Z]{200,500}",
        // Strings resembling real privilege names but slightly wrong
        Just("Se".to_owned()),
        Just("SeNotARealPrivilege".to_owned()),
        Just("SeDebugPrivilege\0".to_owned()),
        Just("SE_DEBUG_PRIVILEGE".to_owned()),
    ]
}

proptest! {
    #[test]
    fn is_privilege_enabled_never_panics(name in privilege_name_strategy()) {
        // Must return Ok or Err, never panic
        let _result = token_privilege::is_privilege_enabled(&name);
    }

    #[test]
    fn has_privilege_never_panics(name in privilege_name_strategy()) {
        let _result = token_privilege::has_privilege(&name);
    }

    #[test]
    fn is_privilege_enabled_returns_result(name in privilege_name_strategy()) {
        let result = token_privilege::is_privilege_enabled(&name);
        // On non-Windows, must always be UnsupportedPlatform
        #[cfg(not(target_os = "windows"))]
        prop_assert!(matches!(
            result,
            Err(token_privilege::TokenPrivilegeError::UnsupportedPlatform)
        ));
        // On Windows, must be Ok(bool) or a typed error, never a panic
        #[cfg(target_os = "windows")]
        {
            let _ = result;
        }
    }

    #[test]
    fn has_privilege_returns_result(name in privilege_name_strategy()) {
        let result = token_privilege::has_privilege(&name);
        #[cfg(not(target_os = "windows"))]
        prop_assert!(matches!(
            result,
            Err(token_privilege::TokenPrivilegeError::UnsupportedPlatform)
        ));
        #[cfg(target_os = "windows")]
        {
            let _ = result;
        }
    }
}

#[test]
fn enumerate_privileges_never_panics() {
    let _result = token_privilege::enumerate_privileges();
}

#[test]
fn is_elevated_never_panics() {
    let _result = token_privilege::is_elevated();
}
