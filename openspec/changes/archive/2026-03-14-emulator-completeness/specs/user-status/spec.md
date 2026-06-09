## ADDED Requirements

### Requirement: Users can be disabled to block sign-in

`POST /v1/mgmt/user/status/disable` SHALL accept `{ "loginId": "<id>" }` and set `disabled: true` on the matching user. `POST /v1/mgmt/user/status/enable` SHALL set `disabled: false`. Both return `{ "user": UserResponse }`.

When a user is disabled, any auth endpoint that issues tokens — `password/signin`, `magiclink/verify`, `otp/verify/*`, `saml/exchange`, `auth/refresh` — SHALL return `403 Forbidden` with `{ "errorCode": "E011006", "errorDescription": "User is disabled" }`.

#### Scenario: Disabled user cannot sign in

- **WHEN** `POST /v1/mgmt/user/status/disable` is called for a user, then `POST /v1/auth/password/signin` is attempted
- **THEN** the sign-in SHALL return `403 Forbidden`

#### Scenario: Re-enabled user can sign in

- **WHEN** a disabled user is re-enabled via `POST /v1/mgmt/user/status/enable`, then sign-in is attempted
- **THEN** the sign-in SHALL succeed and return session tokens

#### Scenario: Disabling unknown user returns 404

- **WHEN** `POST /v1/mgmt/user/status/disable` is called for a non-existent `loginId`
- **THEN** the response SHALL be `404 Not Found`

### Requirement: A user can be removed from a tenant

`POST /v1/mgmt/user/tenant/remove` SHALL accept `{ "loginId": "<id>", "tenantId": "<id>" }` and remove the matching `UserTenant` entry from the user's `userTenants` array. Returns `{ "user": UserResponse }`. If the user is not in the tenant, the call is idempotent and returns the current user state.

#### Scenario: Tenant is removed from user

- **WHEN** `POST /v1/mgmt/user/tenant/remove` is called with a valid user and tenant
- **THEN** the user's `userTenants` array SHALL no longer contain an entry for that `tenantId`

#### Scenario: Remove from tenant is idempotent

- **WHEN** the endpoint is called for a tenant the user is not in
- **THEN** the response SHALL be `200 OK` with the user's current state unchanged

### Requirement: A user's roles within a tenant can be replaced

`POST /v1/mgmt/user/tenant/setRole` SHALL accept `{ "loginId": "<id>", "tenantId": "<id>", "roleNames": ["<role>"] }` and replace the `roleNames` on the matching `UserTenant` entry. Returns `{ "user": UserResponse }`. Fails with `404` if user is not in the specified tenant.

#### Scenario: Tenant roles are replaced

- **WHEN** `POST /v1/mgmt/user/tenant/setRole` is called with `roleNames: ["admin"]`
- **THEN** the user's `userTenants` entry for that tenant SHALL have `roleNames: ["admin"]`

#### Scenario: setRole fails if user is not in tenant

- **WHEN** the `tenantId` is not in the user's `userTenants`
- **THEN** the response SHALL be `404 Not Found`
