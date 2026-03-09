//! All unsafe Win32 FFI calls live in this module.
//!
//! This is the ONLY module that contains `unsafe` code. Every unsafe block
//! has a `// SAFETY:` comment documenting its invariants.

#![allow(unsafe_code)]

use std::io;

use windows::Win32::Foundation::{
    CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_NO_SUCH_PRIVILEGE, HANDLE, LUID,
};
use windows::Win32::Security::{
    GetTokenInformation, LUID_AND_ATTRIBUTES, LookupPrivilegeNameW, LookupPrivilegeValueW,
    PRIVILEGE_SET, PrivilegeCheck, SE_PRIVILEGE_ENABLED, SE_PRIVILEGE_ENABLED_BY_DEFAULT,
    SE_PRIVILEGE_REMOVED, TOKEN_ELEVATION, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::BOOL;

use crate::PrivilegeInfo;
use crate::error::TokenPrivilegeError;

/// `PRIVILEGE_SET_ALL_NECESSARY` — all privileges in the set must be held.
/// Defined in the Windows SDK but not exposed by the `windows` crate.
const PRIVILEGE_SET_ALL_NECESSARY: u32 = 1;

// Compile-time guarantee that `TOKEN_ELEVATION` fits in a `u32` size parameter.
#[allow(clippy::as_conversions)] // Compile-time const, safe
const _: () = assert!(
    std::mem::size_of::<TOKEN_ELEVATION>() <= u32::MAX as usize,
    "TOKEN_ELEVATION size must fit in u32"
);

/// RAII wrapper for Win32 `HANDLE` that calls `CloseHandle` on drop.
pub struct OwnedHandle(HANDLE);

impl std::fmt::Debug for OwnedHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedHandle").finish_non_exhaustive()
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // SAFETY: `CloseHandle` is safe to call on a valid, open handle that
            // we own. The RAII pattern ensures this is called exactly once, when
            // the `OwnedHandle` is dropped. The `is_invalid()` guard skips the
            // call for default-initialized or explicitly invalidated handles.
            unsafe {
                let close_result = CloseHandle(self.0);
                debug_assert!(close_result.is_ok(), "CloseHandle failed: {close_result:?}");
            }
        }
    }
}

/// Open the current process token with `TOKEN_QUERY` access.
pub fn open_current_process_token() -> Result<OwnedHandle, TokenPrivilegeError> {
    let mut handle = HANDLE::default();

    // SAFETY: `GetCurrentProcess()` returns a pseudo-handle that is always valid
    // and does not need to be closed. `OpenProcessToken` writes to `handle` only
    // on success; on failure we return the IO error.
    unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut handle)
            .map_err(|e| TokenPrivilegeError::OpenTokenFailed(io::Error::from(e)))?;
    }

    Ok(OwnedHandle(handle))
}

/// Query whether the token is elevated (UAC elevation).
pub fn query_elevation(token: &OwnedHandle) -> Result<bool, TokenPrivilegeError> {
    // Safe: compile-time assertion above guarantees this fits in u32.
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    const ELEVATION_SIZE: u32 = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

    let mut elevation = TOKEN_ELEVATION::default();
    let mut return_length = 0_u32;

    // SAFETY: We pass a valid token handle and a correctly-sized buffer.
    // `GetTokenInformation` writes at most `elevation_size` bytes into
    // `elevation` and sets `return_length` to the actual bytes written.
    unsafe {
        GetTokenInformation(
            token.0,
            windows::Win32::Security::TokenElevation,
            Some(std::ptr::from_mut(&mut elevation).cast()),
            ELEVATION_SIZE,
            &raw mut return_length,
        )
        .map_err(|e| TokenPrivilegeError::QueryFailed(io::Error::from(e)))?;
    }

    Ok(elevation.TokenIsElevated != 0)
}

/// Look up a privilege LUID by name.
pub fn lookup_privilege_value(name: &str) -> Result<LUID, TokenPrivilegeError> {
    let wide_name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut luid = LUID::default();

    // SAFETY: We pass a null-terminated wide string and a valid LUID pointer.
    // `LookupPrivilegeValueW` writes the LUID on success.
    unsafe {
        LookupPrivilegeValueW(
            None,
            windows::core::PCWSTR(wide_name.as_ptr()),
            &raw mut luid,
        )
        .map_err(|e| {
            if e.code() == ERROR_NO_SUCH_PRIVILEGE.to_hresult() {
                TokenPrivilegeError::InvalidPrivilegeName {
                    name: name.to_owned(),
                }
            } else {
                TokenPrivilegeError::LookupFailed {
                    name: name.to_owned(),
                    source: io::Error::from(e),
                }
            }
        })?;
    }

    Ok(luid)
}

/// Check if a specific privilege (by LUID) is enabled on the token.
pub fn check_privilege_enabled(
    token: &OwnedHandle,
    luid: LUID,
) -> Result<bool, TokenPrivilegeError> {
    let mut privilege_set = PRIVILEGE_SET {
        PrivilegeCount: 1,
        Control: PRIVILEGE_SET_ALL_NECESSARY,
        Privilege: [LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };
    let mut result = BOOL::default();

    // SAFETY: We pass a valid token handle and a correctly initialized
    // PRIVILEGE_SET with count=1. `PrivilegeCheck` writes the result.
    unsafe {
        PrivilegeCheck(token.0, &raw mut privilege_set, &raw mut result)
            .map_err(|e| TokenPrivilegeError::CheckFailed(io::Error::from(e)))?;
    }

    Ok(result.as_bool())
}

/// Enumerate all privileges on the token.
pub fn enumerate_token_privileges(
    token: &OwnedHandle,
) -> Result<Vec<PrivilegeInfo>, TokenPrivilegeError> {
    // First call to get required buffer size
    let mut return_length = 0_u32;

    // SAFETY: First call with null buffer to query the required size.
    // Expected to fail with ERROR_INSUFFICIENT_BUFFER, which we handle.
    let size_result = unsafe {
        GetTokenInformation(
            token.0,
            windows::Win32::Security::TokenPrivileges,
            None,
            0,
            &raw mut return_length,
        )
    };

    // Expected failure — we need the buffer size
    match size_result {
        Ok(()) => {
            return Err(TokenPrivilegeError::QueryFailed(io::Error::other(
                "GetTokenInformation unexpectedly succeeded with null buffer",
            )));
        }
        Err(ref e) if e.code() == ERROR_INSUFFICIENT_BUFFER.to_hresult() => {
            // Expected: buffer was too small, return_length now holds the required size
        }
        Err(e) => {
            return Err(TokenPrivilegeError::QueryFailed(io::Error::from(e)));
        }
    }
    if return_length == 0 {
        return Err(TokenPrivilegeError::QueryFailed(io::Error::other(
            "GetTokenInformation returned zero required length",
        )));
    }

    // Allocate with proper alignment for TOKEN_PRIVILEGES.
    // We use Vec<u64> to guarantee at least 8-byte alignment, which satisfies
    // TOKEN_PRIVILEGES alignment requirements on all Windows platforms.
    #[allow(clippy::as_conversions)] // u32 -> usize safe on all Windows platforms
    let byte_len = return_length as usize;
    let u64_len = byte_len.div_ceil(size_of::<u64>());
    let mut buffer = vec![0_u64; u64_len];

    // SAFETY: We pass a buffer of at least `return_length` bytes as reported
    // by the previous call. `GetTokenInformation` will write TOKEN_PRIVILEGES
    // data into this buffer.
    unsafe {
        GetTokenInformation(
            token.0,
            windows::Win32::Security::TokenPrivileges,
            Some(buffer.as_mut_ptr().cast()),
            return_length,
            &raw mut return_length,
        )
        .map_err(|e| TokenPrivilegeError::QueryFailed(io::Error::from(e)))?;
    }

    // SAFETY: Buffer was filled by GetTokenInformation with TOKEN_PRIVILEGES data.
    // The cast is safe because Vec<u64> guarantees 8-byte alignment, which
    // satisfies TOKEN_PRIVILEGES alignment requirements (align 4 on 32-bit,
    // align 8 on 64-bit Windows).
    let token_privileges = unsafe { &*(buffer.as_ptr().cast::<TOKEN_PRIVILEGES>()) };
    #[allow(clippy::as_conversions)] // u32 -> usize safe on all Windows platforms
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
    let mut name_len = 0_u32;

    // SAFETY: First call with null buffer to get the required name length.
    let size_result =
        unsafe { LookupPrivilegeNameW(None, &raw const luid, None, &raw mut name_len) };

    if name_len == 0 {
        return Err(TokenPrivilegeError::QueryFailed(
            size_result.err().map_or_else(
                || io::Error::other("LookupPrivilegeNameW returned zero length without error"),
                io::Error::from,
            ),
        ));
    }

    #[allow(clippy::as_conversions)] // u32 -> usize safe on all Windows platforms
    let mut name_buf = vec![0_u16; name_len as usize];

    // SAFETY: We pass a buffer of the size reported by the first call.
    // `LookupPrivilegeNameW` writes the privilege name as a wide string.
    unsafe {
        LookupPrivilegeNameW(
            None,
            &raw const luid,
            Some(windows::core::PWSTR(name_buf.as_mut_ptr())),
            &raw mut name_len,
        )
        .map_err(|e| TokenPrivilegeError::QueryFailed(io::Error::from(e)))?;
    }

    // name_len now holds the length WITHOUT the null terminator
    #[allow(clippy::as_conversions)] // u32 -> usize safe on all Windows platforms
    let len = name_len as usize;
    let name_slice = name_buf.get(..len).ok_or_else(|| {
        TokenPrivilegeError::QueryFailed(io::Error::other("name buffer indexing failed"))
    })?;
    String::from_utf16(name_slice).map_err(|_utf16_err| {
        TokenPrivilegeError::QueryFailed(io::Error::other(
            "privilege name contained invalid UTF-16",
        ))
    })
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
        let token = open_current_process_token();
        assert!(token.is_ok(), "should open current process token");
        if let Ok(tok) = token {
            let result = query_elevation(&tok);
            assert!(result.is_ok(), "should query elevation");
        }
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
        let token = open_current_process_token();
        assert!(token.is_ok(), "should open current process token");
        let luid = lookup_privilege_value("SeChangeNotifyPrivilege");
        assert!(luid.is_ok(), "SeChangeNotifyPrivilege should exist");
        if let (Ok(tok), Ok(l)) = (token, luid) {
            let result = check_privilege_enabled(&tok, l);
            assert!(result.is_ok(), "check should succeed");
            assert!(
                matches!(result, Ok(true)),
                "SeChangeNotifyPrivilege should be enabled"
            );
        }
    }

    #[test]
    fn enumerate_privileges_non_empty() {
        let token = open_current_process_token();
        assert!(token.is_ok(), "should open current process token");
        if let Ok(tok) = token {
            let result = enumerate_token_privileges(&tok);
            assert!(result.is_ok(), "enumeration should succeed");
            if let Ok(list) = result {
                assert!(!list.is_empty(), "should have at least one privilege");
            }
        }
    }
}
