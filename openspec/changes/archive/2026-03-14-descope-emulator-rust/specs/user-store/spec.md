## ADDED Requirements

### Requirement: Multi-index in-memory user storage

The user store SHALL maintain four indices for O(1) lookup: by `loginId`, by `userId`, by `email`, and by `phone`. All indices SHALL be updated atomically on every write operation.

#### Scenario: User is findable by all indices after creation

- **WHEN** a user is created with loginId, userId, email, and phone
- **THEN** the user is retrievable by each of those four values

---

### Requirement: User creation

The user store SHALL create users with a generated UUID `userId`, a `loginIds` array containing the primary `loginId` plus any `additionalLoginIds`, default `status: "enabled"`, and `createdTime` set to the current Unix timestamp.

#### Scenario: User is created with all provided fields

- **WHEN** `create(loginId, options)` is called with email, phone, displayName, customAttributes, and userTenants
- **THEN** the stored user contains all provided fields and a generated `userId`

#### Scenario: Duplicate loginId is rejected

- **WHEN** `create(loginId, options)` is called with a `loginId` that already exists
- **THEN** the operation returns `Err(UserAlreadyExists)`

#### Scenario: Test user is flagged

- **WHEN** `create_test_user(loginId, options)` is called
- **THEN** the stored user has `_is_test_user: true`

---

### Requirement: User retrieval

The user store SHALL support loading users by `loginId` (any entry in the `loginIds` array) and by `userId`.

#### Scenario: Load by loginId finds user

- **WHEN** `load(loginId)` is called for an existing user
- **THEN** the user is returned

#### Scenario: Load by additional loginId finds user

- **WHEN** a user has `additionalLoginIds` and `load(additionalLoginId)` is called
- **THEN** the user is returned

#### Scenario: Load non-existent user returns error

- **WHEN** `load(loginId)` is called for a loginId that does not exist
- **THEN** the operation returns `Err(UserNotFound)`

#### Scenario: Load by userId finds user

- **WHEN** `load_by_user_id(userId)` is called for an existing user
- **THEN** the user is returned

---

### Requirement: User search

The user store SHALL support filtering users by `emails`, `phones`, `customAttributes` (partial key-value match), and `withTestUser` flag. Empty search criteria returns all matching users. Test users are excluded by default.

#### Scenario: Search by email returns matching users

- **WHEN** `search({ emails: ["a@example.com"] })` is called
- **THEN** only users whose `email` matches are returned

#### Scenario: Search by customAttribute returns matching users

- **WHEN** `search({ customAttributes: { uid: "abc" } })` is called
- **THEN** only users whose `customAttributes.uid == "abc"` are returned

#### Scenario: Test users excluded by default

- **WHEN** `search({})` is called without `withTestUser: true`
- **THEN** test users are not included in results

#### Scenario: Test users included when flag is set

- **WHEN** `search({ withTestUser: true })` is called
- **THEN** test users are included in results

---

### Requirement: User update (full replace)

The `update(loginId, options)` operation SHALL replace all mutable fields on the user. Fields not provided in `options` SHALL be cleared to their zero/empty values.

#### Scenario: Update clears unprovided fields

- **WHEN** a user with `phone` is updated without providing `phone` in options
- **THEN** the stored user's `phone` is empty string

---

### Requirement: User patch (partial update)

The `patch(loginId, options)` operation SHALL merge provided fields into the existing user. Fields not provided in `options` SHALL remain unchanged.

#### Scenario: Patch preserves unprovided fields

- **WHEN** a user with `phone` is patched without providing `phone` in options
- **THEN** the stored user's `phone` is unchanged

---

### Requirement: User deletion

The user store SHALL support deletion by `loginId`, by `userId`, and bulk deletion of all test users. Deleting a non-existent user SHALL succeed (idempotent).

#### Scenario: Deleted user is not findable

- **WHEN** a user is deleted by loginId
- **THEN** subsequent `load(loginId)` returns `Err(UserNotFound)`

#### Scenario: Delete non-existent user succeeds

- **WHEN** `delete(loginId)` is called for a loginId that does not exist
- **THEN** the operation succeeds without error

#### Scenario: Delete all test users only removes test users

- **WHEN** `delete_all_test_users()` is called with a mix of regular and test users
- **THEN** only users with `_is_test_user: true` are removed

---

### Requirement: Password storage

The user store SHALL store bcrypt-hashed passwords for users. Passwords SHALL be hashed using cost factor 10 via `tokio::task::spawn_blocking` to avoid blocking the async executor.

#### Scenario: Password is stored as bcrypt hash

- **WHEN** `set_password(loginId, plaintext)` is called
- **THEN** the stored value passes `bcrypt::verify(plaintext, stored_hash)`

#### Scenario: Wrong password fails verification

- **WHEN** `verify_password(loginId, wrongPassword)` is called
- **THEN** the operation returns `Err(InvalidCredentials)`
