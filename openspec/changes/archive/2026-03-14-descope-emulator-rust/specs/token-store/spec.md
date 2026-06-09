## ADDED Requirements

### Requirement: Single-use token storage

The token store SHALL store single-use tokens of four types: `magic` (magic link), `saml` (SAML auth code), `embedded` (embedded link), and `reset` (password reset). Each token entry SHALL record the `userId`, token `type`, and `created_at` timestamp.

#### Scenario: Token is stored and retrievable

- **WHEN** a token is stored with `insert(token, TokenEntry { type, userId, created_at })`
- **THEN** `get(token)` returns the entry

#### Scenario: Token is consumed on retrieval

- **WHEN** `consume(token)` is called for an existing token
- **THEN** the token is removed from the store and the entry is returned

#### Scenario: Consuming a non-existent token returns error

- **WHEN** `consume(token)` is called for a token that does not exist or was already consumed
- **THEN** the operation returns `Err(InvalidToken)`

---

### Requirement: Token generation

All tokens SHALL be 32 cryptographically random bytes, hex-encoded to a 64-character string, generated using the `rand` crate's `OsRng`.

#### Scenario: Generated tokens are unique

- **WHEN** two tokens are generated sequentially
- **THEN** the two token strings are different

#### Scenario: Generated token is 64 hex characters

- **WHEN** a token is generated
- **THEN** the token string has exactly 64 characters, all in `[0-9a-f]`
