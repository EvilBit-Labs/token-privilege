//! All unsafe Win32 FFI calls live in this module.
//!
//! This is the ONLY module that contains `unsafe` code. Every unsafe block
//! has a `// SAFETY:` comment documenting its invariants.

#![allow(unsafe_code)]

use std::io;

use windows::Win32::Foundation::{CloseHandle, HANDLE, LUID};
use windows::Win32::Security::{
    GetTokenInformation, LUID_AND_ATTRIBUTES, LookupPrivilegeNameW, LookupPrivilegeValueW,
    PRIVILEGE_SET, PrivilegeCheck, SE_PRIVILEGE_ENABLED, SE_PRIVILEGE_ENABLED_BY_DEFAULT,
    SE_PRIVILEGE_REMOVED, TOKEN_ELEVATION, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

use crate::PrivilegeInfo;
use crate::error::TokenPrivilegeError;

/// RAII wrapper for Win32 `HANDLE` that calls `CloseHandle` on drop.
pub(crate) struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // SAFETY: `CloseHandle` is safe to call on a valid, open handle.
            // After this call the handle is invalidated. Calling `CloseHandle`
            // on an already-closed handle is benign (returns an error we ignore).
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

/// Open the current process token with `TOKEN_QUERY` access.
pub(crate) fn open_current_process_token() -> Result<OwnedHandle, TokenPrivilegeError> {
    let mut handle = HANDLE::default();

    // SAFETY: `GetCurrentProcess()` returns a pseudo-handle that is always valid
    // and does not need to be closed. `OpenProcessToken` writes to `handle` only
    // on success; on failure we return the IO error.
    unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle)
            .map_err(|e| TokenPrivilegeError::OpenTokenFailed(io::Error::from(e)))?;
    }

    Ok(OwnedHandle(handle))
}

/// Query whether the token is elevated (UAC elevation).
pub(crate) fn query_elevation(token: &OwnedHandle) -> Result<bool, TokenPrivilegeError> {
    let mut elevation = TOKEN_ELEVATION::default();
    let mut return_length = 0u32;
    let elevation_size = u32::try_from(std::mem::size_of::<TOKEN_ELEVATION>())
        .expect("TOKEN_ELEVATION size fits in u32");

    // SAFETY: We pass a valid token handle and a correctly-sized buffer.
    // `GetTokenInformation` writes at most `elevation_size` bytes into
    // `elevation` and sets `return_length` to the actual bytes written.
    unsafe {
        GetTokenInformation(
            token.0,
            windows::Win32::Security::TokenElevation,
            Some(std::ptr::from_mut(&mut elevation).cast()),
            elevation_size,
            &mut return_length,
        )
        .map_err(|e| TokenPrivilegeError::QueryFailed(io::Error::from(e)))?;
    }

    Ok(elevation.TokenIsElevated != 0)
}

/// Look up a privilege LUID by name.
pub(crate) fn lookup_privilege_value(name: &str) -> Result<LUID, TokenPrivilegeError> {
    let wide_name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut luid = LUID::default();

    // SAFETY: We pass a null-terminated wide string and a valid LUID pointer.
    // `LookupPrivilegeValueW` writes the LUID on success.
    unsafe {
        LookupPrivilegeValueW(None, windows::core::PCWSTR(wide_name.as_ptr()), &mut luid).map_err(
            |e| {
                let io_err = io::Error::from(e);
                if io_err.raw_os_error() == Some(1313) {
                    // ERROR_NO_SUCH_PRIVILEGE
                    TokenPrivilegeError::InvalidPrivilegeName {
                        name: name.to_owned(),
                    }
                } else {
                    TokenPrivilegeError::LookupFailed {
                        name: name.to_owned(),
                        source: io_err,
                    }
                }
            },
        )?;
    }

    Ok(luid)
}

/// Check if a specific privilege (by LUID) is enabled on the token.
pub(crate) fn check_privilege_enabled(
    token: &OwnedHandle,
    luid: LUID,
) -> Result<bool, TokenPrivilegeError> {
    let mut privilege_set = PRIVILEGE_SET {
        PrivilegeCount: 1,
        Control: 1, // PRIVILEGE_SET_ALL_NECESSARY
        Privilege: [LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };
    let mut result = 0i32;

    // SAFETY: We pass a valid token handle and a correctly initialized
    // PRIVILEGE_SET with count=1. `PrivilegeCheck` writes the result.
    unsafe {
        PrivilegeCheck(token.0, &mut privilege_set, &mut result)
            .map_err(|e| TokenPrivilegeError::CheckFailed(io::Error::from(e)))?;
    }

    Ok(result != 0)
}

/// Enumerate all privileges on the token.
pub(crate) fn enumerate_token_privileges(
    token: &OwnedHandle,
) -> Result<Vec<PrivilegeInfo>, TokenPrivilegeError> {
    // First call to get required buffer size
    let mut return_length = 0u32;

    // SAFETY: First call with null buffer to query the required size.
    // Expected to fail with ERROR_INSUFFICIENT_BUFFER, which we handle.
    let size_result = unsafe {
        GetTokenInformation(
            token.0,
            windows::Win32::Security::TokenPrivileges,
            None,
            0,
            &mut return_length,
        )
    };

    // Expected failure — we need the buffer size
    if size_result.is_ok() || return_length == 0 {
        return Err(TokenPrivilegeError::QueryFailed(io::Error::new(
            io::ErrorKind::Other,
            "unexpected success or zero length from GetTokenInformation size query",
        )));
    }

    let mut buffer = vec![0u8; return_length as usize];

    // SAFETY: We pass a buffer of exactly `return_length` bytes as reported
    // by the previous call. `GetTokenInformation` will write TOKEN_PRIVILEGES
    // data into this buffer.
    unsafe {
        GetTokenInformation(
            token.0,
            windows::Win32::Security::TokenPrivileges,
            Some(buffer.as_mut_ptr().cast()),
            return_length,
            &mut return_length,
        )
        .map_err(|e| TokenPrivilegeError::QueryFailed(io::Error::from(e)))?;
    }

    // SAFETY: The buffer was successfully filled with a TOKEN_PRIVILEGES struct.
    // We read PrivilegeCount and then iterate over that many LUID_AND_ATTRIBUTES.
    let token_privileges = unsafe { &*(buffer.as_ptr().cast::<TOKEN_PRIVILEGES>()) };
    let count = token_privileges.PrivilegeCount as usize;

    // SAFETY: The privileges array in TOKEN_PRIVILEGES is a variable-length
    // array. We access `count` elements, which is what Windows wrote.
    let privileges_slice =
        unsafe { std::slice::from_raw_parts(token_privileges.Privileges.as_ptr(), count) };

    let mut result = Vec::with_capacity(count);
    for attr in privileges_slice {
        let name = lookup_privilege_name(attr.Luid)?;
        let attributes = attr.Attributes;

        result.push(PrivilegeInfo {
            name,
            enabled: (attributes & SE_PRIVILEGE_ENABLED).0 != 0,
            enabled_by_default: (attributes & SE_PRIVILEGE_ENABLED_BY_DEFAULT).0 != 0,
            removed: (attributes & SE_PRIVILEGE_REMOVED).0 != 0,
        });
    }

    Ok(result)
}

/// Look up the name of a privilege by its LUID.
fn lookup_privilege_name(luid: LUID) -> Result<String, TokenPrivilegeError> {
    let mut name_len = 0u32;

    // SAFETY: First call with null buffer to get the required name length.
    let _ =
        unsafe { LookupPrivilegeNameW(None, &luid, windows::core::PWSTR::null(), &mut name_len) };

    if name_len == 0 {
        return Err(TokenPrivilegeError::QueryFailed(io::Error::last_os_error()));
    }

    let mut name_buf = vec![0u16; name_len as usize];

    // SAFETY: We pass a buffer of the size reported by the first call.
    // `LookupPrivilegeNameW` writes the privilege name as a wide string.
    unsafe {
        LookupPrivilegeNameW(
            None,
            &luid,
            windows::core::PWSTR(name_buf.as_mut_ptr()),
            &mut name_len,
        )
        .map_err(|e| TokenPrivilegeError::QueryFailed(io::Error::from(e)))?;
    }

    // name_len now holds the length WITHOUT the null terminator
    Ok(String::from_utf16_lossy(&name_buf[..name_len as usize]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_and_drop_token_handle() {
        let token = open_current_process_token();
        assert!(token.is_ok(), "should open current process token");
        // OwnedHandle drops here — verifies RAII doesn't panic
    }

    #[test]
    fn query_elevation_returns_bool() {
        let token = open_current_process_token().expect("open token");
        let result = query_elevation(&token);
        assert!(result.is_ok(), "should query elevation");
    }

    #[test]
    fn lookup_known_privilege() {
        let result = lookup_privilege_value("SeChangeNotifyPrivilege");
        assert!(result.is_ok(), "SeChangeNotifyPrivilege should exist");
    }

    #[test]
    fn lookup_invalid_privilege() {
        let result = lookup_privilege_value("SeTotallyFakePrivilege");
        assert!(result.is_err(), "fake privilege should fail");
    }

    #[test]
    fn check_change_notify_enabled() {
        let token = open_current_process_token().expect("open token");
        let luid = lookup_privilege_value("SeChangeNotifyPrivilege").expect("lookup");
        let enabled = check_privilege_enabled(&token, luid);
        assert!(enabled.is_ok(), "check should succeed");
        assert!(
            enabled.expect("checked"),
            "SeChangeNotifyPrivilege should be enabled"
        );
    }

    #[test]
    fn enumerate_privileges_non_empty() {
        let token = open_current_process_token().expect("open token");
        let privs = enumerate_token_privileges(&token);
        assert!(privs.is_ok(), "enumeration should succeed");
        let list = privs.expect("enumerated");
        assert!(!list.is_empty(), "should have at least one privilege");
    }
}
