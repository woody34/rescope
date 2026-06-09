## Why

The emulator's user management API is missing several endpoints that the Descope SDK uses:
- `POST /v1/mgmt/user/update/role/add` — append roles without replacing
- `POST /v1/mgmt/user/update/picture` — update profile picture URL
- `POST /v1/mgmt/user/update/customAttribute` — update a single custom attribute
- The status endpoint rejects `invited` — a valid Descope status

## What

### Capability: invited-status
Accept `invited` as a valid status in `POST /v1/mgmt/user/update/status`.

### Capability: add-role-endpoint
Implement `POST /v1/mgmt/user/update/role/add` — appends roles to existing set without replacing.

### Capability: update-picture-endpoint
Implement `POST /v1/mgmt/user/update/picture` — sets the user's profile picture URL.

### Capability: update-custom-attribute-endpoint
Implement `POST /v1/mgmt/user/update/customAttribute` — updates a single key in the user's custom attributes map.
