# Safety Contract

All `unsafe` code in `token-privilege` is confined to a single module: `src/ffi.rs`. This page documents every unsafe block, its safety invariants, and the mechanisms that enforce correctness.

[TOC]

## Enforcement

The Clippy lint `undocumented_unsafe_blocks = "deny"` is set in `Cargo.toml`. Any `unsafe` block without a `// SAFETY:` comment is a compile-time error. This ensures every unsafe operation has an explicit justification in the source code.

## Unsafe Blocks

### `OwnedHandle::drop` -- `CloseHandle`

```rust,ignore
// SAFETY: `CloseHandle` is safe to call on a valid, open handle that
// we own. The RAII pattern ensures this is called exactly once, when
// the `OwnedHandle` is dropped. The `is_invalid()` guard skips the
// call for default-initialized or explicitly invalidated handles.
unsafe {
    let close_result = CloseHandle(self.0);
    debug_assert!(close_result.is_ok(), "CloseHandle failed: {close_result:?}");
}
```

**Invariant:** The `HANDLE` stored in `OwnedHandle` is either valid (opened by `OpenProcessToken`) or `INVALID_HANDLE_VALUE`. The `is_invalid()` check before the call prevents closing an invalid handle. The handle is never cloned or aliased, so double-close is impossible under normal usage. A `debug_assert!` verifies the close succeeds in debug builds.

---

### `open_current_process_token` -- `OpenProcessToken`

```rust,ignore
// SAFETY: `GetCurrentProcess()` returns a pseudo-handle that is always valid
// and does not need to be closed. `OpenProcessToken` writes to `handle` only
// on success; on failure we return the IO error.
unsafe {
    OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle)
        .map_err(|e| TokenPrivilegeError::OpenTokenFailed(io::Error::from(e)))?;
}
```

**Invariant:** `GetCurrentProcess()` always returns a valid pseudo-handle (`-1`). The `handle` output parameter is a local variable with exclusive access. On success, `OpenProcessToken` writes a valid token handle. On failure, the error is propagated and no handle is stored.

---

### `query_elevation` -- `GetTokenInformation`

```rust,ignore
// SAFETY: We pass a valid token handle and a correctly-sized buffer.
// `GetTokenInformation` writes at most `elevation_size` bytes into
// `elevation` and sets `return_length` to the actual bytes written.
unsafe {
    GetTokenInformation(
        token.0,
        TokenElevation,
        Some(std::ptr::from_mut(&mut elevation).cast()),
        elevation_size,
        &mut return_length,
    )
    .map_err(|e| TokenPrivilegeError::QueryFailed(io::Error::from(e)))?;
}
```

**Invariant:** The token handle is valid (obtained from `open_current_process_token` and wrapped in `OwnedHandle`). The buffer is a stack-allocated `TOKEN_ELEVATION` of known, fixed size. The `elevation_size` parameter matches `size_of::<TOKEN_ELEVATION>()`. The cast from `*mut TOKEN_ELEVATION` to `*mut c_void` is layout-compatible.

---

### `lookup_privilege_value` -- `LookupPrivilegeValueW`

```rust,ignore
// SAFETY: We pass a null-terminated wide string and a valid LUID pointer.
// `LookupPrivilegeValueW` writes the LUID on success.
unsafe {
    LookupPrivilegeValueW(None, PCWSTR(wide_name.as_ptr()), &mut luid)
        .map_err(|e| { /* error mapping */ })?;
}
```

**Invariant:** `wide_name` is constructed by encoding the input `&str` as UTF-16 and appending a null terminator. The `PCWSTR` wrapper points to a live `Vec<u16>` that outlives the call. The `luid` output is a local variable with exclusive access.

---

### `check_privilege_enabled` -- `PrivilegeCheck`

```rust,ignore
// SAFETY: We pass a valid token handle and a correctly initialized
// PRIVILEGE_SET with count=1. `PrivilegeCheck` writes the result.
unsafe {
    PrivilegeCheck(token.0, &mut privilege_set, &mut result)
        .map_err(|e| TokenPrivilegeError::CheckFailed(io::Error::from(e)))?;
}
```

**Invariant:** The token handle is valid. The `PRIVILEGE_SET` is stack-allocated with `PrivilegeCount = 1` and a single correctly-initialized `LUID_AND_ATTRIBUTES` entry. The `result` output is a local `i32`.

---

### `enumerate_token_privileges` -- `GetTokenInformation` (size query)

```rust,ignore
// SAFETY: First call with null buffer to query the required size.
// Expected to fail with ERROR_INSUFFICIENT_BUFFER, which we handle.
let size_result = unsafe {
    GetTokenInformation(token.0, TokenPrivileges, None, 0, &mut return_length)
};
```

**Invariant:** Passing `None` and size `0` is the documented pattern for querying the required buffer size. No data is written. The function is expected to fail with `ERROR_INSUFFICIENT_BUFFER`.

---

### `enumerate_token_privileges` -- `GetTokenInformation` (data query)

```rust,ignore
// SAFETY: We pass a buffer of exactly `return_length` bytes as reported
// by the previous call. `GetTokenInformation` will write TOKEN_PRIVILEGES
// data into this buffer.
unsafe {
    GetTokenInformation(
        token.0, TokenPrivileges,
        Some(buffer.as_mut_ptr().cast()),
        return_length, &mut return_length,
    )
    .map_err(|e| TokenPrivilegeError::QueryFailed(io::Error::from(e)))?;
}
```

**Invariant:** The buffer is a `Vec<u64>` heap-allocated with at least `return_length` bytes (using `div_ceil` to round up to whole `u64` elements). The `Vec<u64>` guarantees 8-byte alignment, which satisfies `TOKEN_PRIVILEGES` alignment requirements on all Windows platforms. The token handle has not been invalidated between calls.

---

### `enumerate_token_privileges` -- `TOKEN_PRIVILEGES` pointer cast

```rust,ignore
// SAFETY: The buffer was successfully filled with a TOKEN_PRIVILEGES struct.
// We read PrivilegeCount and then iterate over that many LUID_AND_ATTRIBUTES.
let token_privileges = unsafe { &*(buffer.as_ptr().cast::<TOKEN_PRIVILEGES>()) };
```

**Invariant:** `GetTokenInformation` succeeded, guaranteeing the buffer contains a valid `TOKEN_PRIVILEGES` structure. The buffer is a `Vec<u64>`, providing 8-byte alignment which exceeds the alignment requirement of `TOKEN_PRIVILEGES` (align 4 on 32-bit, align 8 on 64-bit Windows).

---

### `enumerate_token_privileges` -- `slice::from_raw_parts`

```rust,ignore
// SAFETY: The privileges array in TOKEN_PRIVILEGES is a variable-length
// array. We access `count` elements, which is what Windows wrote.
let privileges_slice = unsafe {
    std::slice::from_raw_parts(token_privileges.Privileges.as_ptr(), count)
};
```

**Invariant:** `count` is read from `PrivilegeCount`, which was written by Windows. The pointer comes from the `Privileges` field of the successfully populated `TOKEN_PRIVILEGES`. The buffer is large enough because its size was determined by the Windows API itself.

---

### `lookup_privilege_name` -- `LookupPrivilegeNameW` (size query and data query)

```rust,ignore
// SAFETY: First call with null buffer to get the required name length.
let _ = unsafe {
    LookupPrivilegeNameW(None, &luid, PWSTR::null(), &mut name_len)
};
```

```rust,ignore
// SAFETY: We pass a buffer of the size reported by the first call.
// `LookupPrivilegeNameW` writes the privilege name as a wide string.
unsafe {
    LookupPrivilegeNameW(None, &luid, PWSTR(name_buf.as_mut_ptr()), &mut name_len)
        .map_err(|e| TokenPrivilegeError::QueryFailed(io::Error::from(e)))?;
}
```

**Invariant:** The first call uses a null buffer to query the required size. The second call allocates a `Vec<u16>` of exactly that size. The LUID is valid because it was obtained from the same token enumeration. `name_len` is updated to reflect the actual characters written (excluding the null terminator).

## RAII Handle Wrapper

`OwnedHandle` is a `pub(crate)` newtype around `HANDLE` that implements `Drop` to call `CloseHandle`. This ensures handles are closed on all code paths:

- Normal return
- Early `?` propagation
- Panic unwinding

The handle is never `Copy` or `Clone`, preventing aliased access. It is never exposed outside `ffi.rs`.
