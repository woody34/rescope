## ADDED Requirements

### Requirement: Refresh token revocation

The revocation store SHALL maintain a `HashSet<String>` of invalidated refresh JWT strings. Any refresh JWT added via `revoke(token)` SHALL be rejected by all subsequent operations that check the revocation set.

#### Scenario: Revoked token is detected

- **WHEN** `revoke(token)` is called and then `is_revoked(token)` is checked
- **THEN** `is_revoked` returns `true`

#### Scenario: Non-revoked token passes check

- **WHEN** `is_revoked(token)` is called for a token that has not been revoked
- **THEN** `is_revoked` returns `false`

---

### Requirement: Revocation set is cleared on reset

The revocation store SHALL be completely cleared when `POST /emulator/reset` is called.

#### Scenario: Reset clears revocation set

- **WHEN** tokens are revoked and then `reset()` is called on the store
- **THEN** `is_revoked` returns `false` for all previously revoked tokens
