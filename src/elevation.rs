//! Elevation detection for the current process.

use crate::error::TokenPrivilegeError;
use crate::ffi;

/// Check if the current process is running with elevated (Administrator) privileges.
///
/// Returns `true` if the process token indicates elevation.
///
/// # Errors
///
/// Returns an error if the process token cannot be opened or queried.
pub fn is_elevated() -> Result<bool, TokenPrivilegeError> {
    let token = ffi::open_current_process_token()?;
    ffi::query_elevation(&token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_elevated_returns_result() {
        let result = is_elevated();
        assert!(result.is_ok(), "is_elevated should return Ok on Windows");
    }

    #[test]
    fn is_elevated_is_consistent() {
        let first = is_elevated().expect("first call");
        let second = is_elevated().expect("second call");
        assert_eq!(first, second, "elevation status should be stable");
    }
}
