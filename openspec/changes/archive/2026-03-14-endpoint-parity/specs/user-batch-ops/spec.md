## ADDED Requirements

### Requirement: Users can be created in batch
`POST /v1/mgmt/user/create/batch` SHALL accept `{ users: [...] }` where each entry matches the single `user/create` schema. It SHALL create each user in order. On first conflict it SHALL return 409 without rolling back already-created users.

#### Scenario: All users created successfully
- **WHEN** `POST /v1/mgmt/user/create/batch` is called with 3 new users
- **THEN** all 3 users exist and response returns `{ users: [...] }`

#### Scenario: Partial failure on duplicate stops processing
- **WHEN** `POST /v1/mgmt/user/create/batch` is called and the second user already exists
- **THEN** response is 409 and the first user was still created

### Requirement: Users can be deleted in batch
`DELETE /v1/mgmt/user/delete/batch` SHALL accept `{ loginIds: [...] }` and call `delete_by_login_id` for each. Non-existent loginIds are silently ignored.

#### Scenario: Multiple users deleted
- **WHEN** `DELETE /v1/mgmt/user/delete/batch` is called with 2 existing loginIds
- **THEN** both users are removed

#### Scenario: Non-existent loginIds are ignored
- **WHEN** `DELETE /v1/mgmt/user/delete/batch` includes a loginId that doesn't exist
- **THEN** response is `{ ok: true }` without error
