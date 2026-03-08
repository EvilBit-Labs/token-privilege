# Commit Message Style for token-privilege

Use Conventional Commits:

`<type>(<scope>): <description>`

- **Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`
- **Scopes** (required):
  - Core/API: `lib`, `api`, `error`, `elevation`, `privilege`, `privileges`
  - Unsafe boundary: `ffi`, `safety`, `security`
  - Project/tooling: `docs`, `book`, `tests`, `ci`, `deps`, `release`
- **Description**: imperative, capitalized, ≤72 chars, no period
- **Body** (optional): blank line, bullet list, explain what/why
- **Footer** (optional): blank line, issue refs (`Closes #123`) or `BREAKING CHANGE:`
- **Breaking changes**: add `!` after type/scope or use `BREAKING CHANGE:`

## Repository-specific expectations

- Keep public API changes explicit in commit messages (`is_elevated`, `is_privilege_enabled`, `has_privilege`, `enumerate_privileges`, `PrivilegeInfo`, `TokenPrivilegeError`).
- Any change involving `unsafe` must use `ffi` or `safety` scope and mention the safety invariant in the body.
- Cross-platform behavior changes should call out Windows vs non-Windows behavior (especially `UnsupportedPlatform`).
- Test-related commits should note platform gating when relevant (e.g., Windows-only assertions vs non-Windows stubs).

## Examples

- `feat(elevation): Add TokenElevationTypeFull handling in is_elevated`
- `feat(privileges): Add SE_MANAGE_VOLUME constant and docs`
- `fix(privilege): Return InvalidPrivilegeName for empty privilege input`
- `fix(ffi): Ensure token handle closes on all GetTokenInformation paths`
- `refactor(safety): Document RAII handle invariants for CloseHandle calls`
- `test(privilege): Add SeChangeNotifyPrivilege enumeration coverage`
- `ci(ci): Run nextest on windows-latest in cross-platform matrix`
- `docs(book): Add safety contract section for unsafe FFI boundaries`
- `chore(deps): Bump windows crate features for token APIs`

## Suggested commit body template

- What changed
- Why it changed
- Safety impact (if touching `ffi.rs` or unsafe code)
- Platform impact (Windows / non-Windows)
- Verification performed (`just ci-check`, `cargo nextest`, etc.)
