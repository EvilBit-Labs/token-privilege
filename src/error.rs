/// Errors that can occur when querying token privileges.
///
/// This enum is `#[non_exhaustive]` to allow adding new error variants in
/// future minor releases without breaking downstream code. Always include
/// a wildcard arm when matching.
///
/// # Examples
///
/// ```rust
/// use token_privilege::TokenPrivilegeError;
///
/// fn handle_error(err: TokenPrivilegeError) {
///     match err {
///         TokenPrivilegeError::UnsupportedPlatform => {
///             eprintln!("This feature requires Windows");
///         }
///         other => eprintln!("Token error: {other}"),
///     }
/// }
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TokenPrivilegeError {
    /// Failed to open the process token.
    ///
    /// This typically occurs if the process lacks sufficient permissions
    /// to query its own token (very rare in practice).
    #[error("failed to open process token: {0}")]
    OpenTokenFailed(std::io::Error),

    /// Failed to query token information.
    ///
    /// Returned when `GetTokenInformation` fails after the token
    /// has been successfully opened.
    #[error("failed to query token information: {0}")]
    QueryFailed(std::io::Error),

    /// The specified privilege name is invalid or not recognized.
    ///
    /// The Windows privilege name (e.g., `"SeDebugPrivilege"`) was not
    /// found in the system's privilege database.
    #[error("invalid privilege name: {name}")]
    InvalidPrivilegeName {
        /// The privilege name that was not recognized.
        name: String,
    },

    /// Failed to look up the privilege value for the given name.
    ///
    /// Unlike [`InvalidPrivilegeName`](Self::InvalidPrivilegeName), this
    /// indicates an OS-level failure during the lookup rather than an
    /// unknown privilege name.
    #[error("privilege lookup failed for '{name}': {source}")]
    LookupFailed {
        /// The privilege name that was being looked up.
        name: String,
        /// The underlying OS error.
        source: std::io::Error,
    },

    /// Failed to check privilege status.
    ///
    /// Returned when `PrivilegeCheck` fails after the privilege LUID
    /// has been successfully resolved.
    #[error("privilege check failed: {0}")]
    CheckFailed(std::io::Error),

    /// This functionality is only available on Windows.
    ///
    /// All public functions in this crate return this error on non-Windows
    /// platforms. This allows downstream crates to depend on `token-privilege`
    /// unconditionally and handle the non-Windows case gracefully.
    #[error("token privilege operations are only supported on Windows")]
    UnsupportedPlatform,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_platform_display() {
        let err = TokenPrivilegeError::UnsupportedPlatform;
        let msg = err.to_string();
        assert!(
            msg.contains("only supported on Windows"),
            "unexpected message: {msg}"
        );
    }

    #[test]
    fn open_token_failed_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = TokenPrivilegeError::OpenTokenFailed(io_err);
        let msg = err.to_string();
        assert!(
            msg.contains("open process token"),
            "unexpected message: {msg}"
        );
    }

    #[test]
    fn invalid_privilege_name_display() {
        let err = TokenPrivilegeError::InvalidPrivilegeName {
            name: "FakePrivilege".to_owned(),
        };
        let msg = err.to_string();
        assert!(msg.contains("FakePrivilege"), "unexpected message: {msg}");
    }

    #[test]
    fn query_failed_display() {
        let io_err = std::io::Error::other("query error");
        let err = TokenPrivilegeError::QueryFailed(io_err);
        let msg = err.to_string();
        assert!(
            msg.contains("query token information"),
            "unexpected message: {msg}"
        );
    }

    #[test]
    fn lookup_failed_display() {
        let io_err = std::io::Error::other("not found");
        let err = TokenPrivilegeError::LookupFailed {
            name: "SeDebugPrivilege".to_owned(),
            source: io_err,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("SeDebugPrivilege"),
            "unexpected message: {msg}"
        );
    }

    #[test]
    fn check_failed_display() {
        let io_err = std::io::Error::other("check error");
        let err = TokenPrivilegeError::CheckFailed(io_err);
        let msg = err.to_string();
        assert!(
            msg.contains("privilege check failed"),
            "unexpected message: {msg}"
        );
    }

    #[test]
    fn error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TokenPrivilegeError>();
    }
}
