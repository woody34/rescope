## ADDED Requirements

### Requirement: User display name can be updated via management API
`POST /v1/mgmt/user/update/name` SHALL accept `{ loginId, name }` and patch the user's `name` field. Returns `{ user }`.

#### Scenario: Display name is updated
- **WHEN** `POST /v1/mgmt/user/update/name` is called with a valid loginId and name
- **THEN** user's name is updated and response returns `{ user }` with updated name

### Requirement: User phone can be updated via management API
`POST /v1/mgmt/user/update/phone` SHALL accept `{ loginId, phone, verified? }` and update the user's phone field and index. Returns `{ user }`.

#### Scenario: Phone is updated
- **WHEN** `POST /v1/mgmt/user/update/phone` is called with a valid loginId and phone
- **THEN** user's phone is updated and response returns `{ user }`

### Requirement: User loginId can be renamed
`POST /v1/mgmt/user/update/loginid` SHALL accept `{ loginId, newLoginId }`. It SHALL remove the old loginId from `loginIds` and `by_login_id` index, add the new one, and return `{ user }`.

#### Scenario: LoginId is renamed
- **WHEN** `POST /v1/mgmt/user/update/loginid` is called with valid old and new loginIds
- **THEN** user can be loaded by the new loginId and old loginId returns 404

#### Scenario: New loginId already taken returns conflict
- **WHEN** `POST /v1/mgmt/user/update/loginid` is called and newLoginId already exists
- **THEN** response is 409

### Requirement: User global roles can be set
`POST /v1/mgmt/user/update/role/set` SHALL accept `{ loginId, roleNames }` and fully replace the user's `role_names`. Returns `{ user }`.

#### Scenario: Roles are set
- **WHEN** `POST /v1/mgmt/user/update/role/set` is called with roles `["admin"]`
- **THEN** user's `roleNames` is `["admin"]`

### Requirement: User global roles can be removed
`POST /v1/mgmt/user/update/role/remove` SHALL accept `{ loginId, roleNames }` and remove the specified roles from the user's `role_names`, leaving unlisted roles unchanged.

#### Scenario: Specified roles are removed
- **WHEN** user has roles `["admin","viewer"]` and `remove` is called with `["admin"]`
- **THEN** user's `roleNames` is `["viewer"]`
