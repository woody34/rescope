/// OpenAPI 3.0 specification for the Rescope emulator.
/// Returns the spec as a serde_json::Value — zero external dependencies.
use serde_json::{json, Map, Value};

/// Build the full OpenAPI 3.0.3 spec covering all endpoints.
pub fn build_openapi_spec() -> Value {
    let mut spec = Map::new();
    spec.insert("openapi".into(), json!("3.0.3"));
    spec.insert("info".into(), build_info());
    spec.insert("servers".into(), build_servers());
    spec.insert("tags".into(), build_tags());
    spec.insert("paths".into(), build_paths());
    Value::Object(spec)
}

fn build_info() -> Value {
    json!({
        "title": "Rescope — Descope Emulator API",
        "version": "0.1.0",
        "description": "Local emulator for the Descope identity platform. Drop-in replacement for the Descope API during development and testing.\n\nSupports user management, authentication flows (password, OTP, magic link, SAML/SSO), session management, tenant/role/permission RBAC, JWT customization, and more.\n\n## Quick Start\n1. `GET /health` — verify emulator is running\n2. `POST /emulator/reset` — clean state for tests\n3. `POST /v1/auth/password/signup` — create a user and get JWTs\n4. `GET /v1/auth/me` — verify the session works"
    })
}

fn build_servers() -> Value {
    json!([
        { "url": "http://localhost:4600", "description": "Local emulator (default)" },
    ])
}

fn build_tags() -> Value {
    json!([
        { "name": "Lifecycle", "description": "Health check and state management" },
        { "name": "Auth: Password", "description": "Password-based sign-up and sign-in" },
        { "name": "Auth: OTP", "description": "One-time password (email/SMS) flows" },
        { "name": "Auth: Magic Link", "description": "Magic link (email/SMS) flows" },
        { "name": "Auth: SAML/SSO", "description": "SAML and SSO authentication" },
        { "name": "Auth: Session", "description": "Session refresh, logout, and introspection" },
        { "name": "Mgmt: Users", "description": "User CRUD and search" },
        { "name": "Mgmt: User Roles", "description": "Project-level role assignment" },
        { "name": "Mgmt: User Tenants", "description": "Tenant membership and tenant-scoped roles" },
        { "name": "Mgmt: Tenants", "description": "Tenant (organization) CRUD" },
        { "name": "Mgmt: Roles", "description": "Role definitions with permissions" },
        { "name": "Mgmt: Permissions", "description": "Permission definitions" },
        { "name": "Mgmt: Passwords", "description": "Admin password management" },
        { "name": "Mgmt: Access Keys", "description": "Machine-to-machine access keys" },
        { "name": "Mgmt: Custom Attributes", "description": "Custom attribute schema definitions" },
        { "name": "Mgmt: Test Helpers", "description": "Generate OTP/magic-link/enchanted-link for test users" },
        { "name": "Mgmt: Embedded Links", "description": "Server-side authentication tokens" },
        { "name": "Emulator Helpers", "description": "Emulator-only endpoints (not in real Descope API)" },
        { "name": "JWKS", "description": "JSON Web Key Set for JWT validation" }
    ])
}

fn build_paths() -> Value {
    let mut paths = Map::new();
    add_lifecycle(&mut paths);
    add_auth_password(&mut paths);
    add_auth_otp(&mut paths);
    add_auth_magic_link(&mut paths);
    add_auth_session(&mut paths);
    add_mgmt_users(&mut paths);
    add_mgmt_user_roles(&mut paths);
    add_mgmt_user_tenants(&mut paths);
    add_mgmt_tenants(&mut paths);
    add_mgmt_permissions(&mut paths);
    add_mgmt_roles(&mut paths);
    add_mgmt_passwords(&mut paths);
    add_mgmt_access_keys(&mut paths);
    add_mgmt_custom_attrs(&mut paths);
    add_mgmt_test_helpers(&mut paths);
    add_emulator_helpers(&mut paths);
    add_jwks(&mut paths);
    Value::Object(paths)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn get_op(id: &str, summary: &str, desc: &str, tag: &str) -> Value {
    json!({ "get": {
        "operationId": id, "summary": summary, "description": desc, "tags": [tag],
        "responses": { "200": { "description": "OK" }, "400": { "description": "Bad request" }, "401": { "description": "Unauthorized" } }
    }})
}

fn post_op(id: &str, summary: &str, desc: &str, tag: &str, body: &str) -> Value {
    json!({ "post": {
        "operationId": id, "summary": summary, "description": desc, "tags": [tag],
        "requestBody": { "description": body, "required": true,
            "content": { "application/json": { "schema": { "type": "object" } } } },
        "responses": { "200": { "description": "OK" }, "400": { "description": "Bad request" }, "401": { "description": "Unauthorized" } }
    }})
}

fn merge_into(paths: &mut Map<String, Value>, path: &str, val: Value) {
    if let Value::Object(map) = val {
        let entry = paths.entry(path.to_string()).or_insert_with(|| json!({}));
        if let Value::Object(existing) = entry {
            for (k, v) in map {
                existing.insert(k, v);
            }
        }
    }
}

// ─── Section builders ─────────────────────────────────────────────────────────

fn add_lifecycle(p: &mut Map<String, Value>) {
    merge_into(p, "/health", get_op("health", "Health check",
        "Returns `{ \"status\": \"ok\" }`. Poll this to verify the emulator is running before tests.", "Lifecycle"));
    merge_into(p, "/emulator/reset", post_op("reset", "Reset emulator state",
        "Clears all runtime state (users, sessions, OTPs, tokens). Config stores preserved.\n\n**Example:**\n```js\nawait fetch('http://localhost:4600/emulator/reset', { method: 'POST' });\n```",
        "Lifecycle", "No body required"));
}

fn add_auth_password(p: &mut Map<String, Value>) {
    let t = "Auth: Password";
    merge_into(p, "/v1/auth/password/signup", post_op("passwordSignup", "Sign up with password",
        "Creates a new user and signs them in. Returns session + refresh JWTs.\n\n**Request:** `{ \"loginId\": \"user@example.com\", \"password\": \"S3cure!Pass\" }`", t, "{ loginId, password }"));
    merge_into(p, "/v1/auth/password/signin", post_op("passwordSignin", "Sign in with password",
        "Authenticates an existing user. Returns session + refresh JWTs.\n\n**Request:** `{ \"loginId\": \"user@example.com\", \"password\": \"S3cure!Pass\" }`", t, "{ loginId, password }"));
    merge_into(p, "/v1/auth/password/replace", post_op("passwordReplace", "Replace password",
        "Replaces old password with new. Requires correct old password.\n\n**Request:** `{ \"loginId\": \"...\", \"oldPassword\": \"old\", \"newPassword\": \"new\" }`", t, "{ loginId, oldPassword, newPassword }"));
    merge_into(
        p,
        "/v1/auth/password/policy",
        get_op(
            "passwordPolicy",
            "Get password policy",
            "Returns password policy configuration.",
            t,
        ),
    );
}

fn add_auth_otp(p: &mut Map<String, Value>) {
    let t = "Auth: OTP";
    merge_into(p, "/v1/auth/otp/signup/email", post_op("otpSignupEmail", "OTP signup via email",
        "Sends OTP to new user's email. Creates user if needed.\n\n**Emulator:** Code printed to console + `GET /emulator/otp/:loginId`.", t, "{ loginId }"));
    merge_into(
        p,
        "/v1/auth/otp/signin/email",
        post_op(
            "otpSigninEmail",
            "OTP sign-in via email",
            "Sends OTP to existing user's email.",
            t,
            "{ loginId }",
        ),
    );
    merge_into(p, "/v1/auth/otp/verify/email", post_op("otpVerifyEmail", "Verify OTP (email)",
        "Verifies OTP code. Returns session + refresh JWTs.\n\n**Request:** `{ \"loginId\": \"...\", \"code\": \"123456\" }`", t, "{ loginId, code }"));
}

fn add_auth_magic_link(p: &mut Map<String, Value>) {
    let t = "Auth: Magic Link";
    merge_into(p, "/v1/auth/magiclink/signup/email", post_op("magicLinkSignupEmail", "Magic link signup via email",
        "Sends magic link to new user. Creates user if needed.\n\n**Emulator:** Token printed to console.", t, "{ loginId, URI }"));
    merge_into(p, "/v1/auth/magiclink/verify", post_op("magicLinkVerify", "Verify magic link token",
        "Completes auth by verifying magic link token. Returns JWTs.\n\n**Request:** `{ \"token\": \"<token>\" }`", t, "{ token }"));
}

fn add_auth_session(p: &mut Map<String, Value>) {
    let t = "Auth: Session";
    merge_into(p, "/v1/auth/refresh", post_op("refresh", "Refresh session",
        "Exchanges refresh JWT for new session JWT. Token is rotated.\n\n**Request:** `{ \"refreshJwt\": \"<token>\" }`", t, "{ refreshJwt }"));
    merge_into(
        p,
        "/v1/auth/logout",
        post_op(
            "logout",
            "Logout",
            "Invalidates the current session.",
            t,
            "Empty body",
        ),
    );
    merge_into(
        p,
        "/v1/auth/logoutall",
        post_op(
            "logoutAll",
            "Logout all sessions",
            "Invalidates all sessions for the user.",
            t,
            "Empty body",
        ),
    );
    merge_into(
        p,
        "/v1/auth/me",
        get_op(
            "me",
            "Get current user",
            "Returns authenticated user's profile. Requires valid session JWT.",
            t,
        ),
    );
    merge_into(
        p,
        "/v1/auth/validate",
        post_op(
            "validate",
            "Validate session",
            "Validates a session JWT.",
            t,
            "{ sessionJwt }",
        ),
    );
    merge_into(
        p,
        "/v1/auth/tenant/select",
        post_op(
            "tenantSelect",
            "Select tenant",
            "Switches session to a tenant context.",
            t,
            "{ tenant }",
        ),
    );
}

fn add_mgmt_users(p: &mut Map<String, Value>) {
    let t = "Mgmt: Users";
    merge_into(p, "/v1/mgmt/user/create", post_op("createUser", "Create user",
        "Creates a new user. loginId is required.\n\n**Request:**\n```json\n{ \"loginId\": \"jane@startup.com\", \"email\": \"jane@startup.com\", \"name\": \"Jane Smith\" }\n```",
        t, "{ loginId, email?, phone?, name?, roleNames?, userTenants?, customAttributes?, picture? }"));
    merge_into(p, "/v1/mgmt/user/create/test", post_op("createTestUser", "Create test user",
        "Creates a test user (flagged `_isTestUser: true`). Can be bulk-deleted. Excluded from search by default.", t, "Same as create user"));
    merge_into(p, "/v1/mgmt/user", get_op("loadUser", "Load user by login ID",
        "Loads a single user. Query param: `loginid`.\n\n**Example:** `GET /v1/mgmt/user?loginid=jane@startup.com`", t));
    merge_into(p, "/v2/mgmt/user/search", post_op("searchUsers", "Search users",
        "Returns users matching filters. All filters AND-composed; within-list OR.\n\n**Filters:** `loginIds`, `statuses`, `tenantIds`, `roleNames`, `text`, `sort`, `createdAfter`, `createdBefore`.\n\n**Text search:** case-insensitive substring across loginId, name, email, phone.\n\n**Sort:** `[{ \"field\": \"name\", \"desc\": false }]`. Fields: name, email, phone, status, loginId, createdTime.",
        t, "{ loginIds?, statuses?, tenantIds?, roleNames?, text?, sort?, limit?, page? }"));
    merge_into(p, "/v1/mgmt/project/snapshot/export", post_op("exportProjectSnapshot", "Export project snapshot",
        "Exports project configuration (roles, permissions, connectors, JWT templates, custom attributes, auth method config) as a Descope-style `{ \"files\": { ... } }` map. Users and tenants are excluded, matching cloud Descope (they are exported via their own APIs).",
        t, "{} (no request body required)"));
    merge_into(
        p,
        "/v1/mgmt/user/update",
        post_op(
            "updateUser",
            "Update user (full replace)",
            "Replaces user fields wholesale.",
            t,
            "{ loginId, email?, phone?, name?, roleNames?, customAttributes?, picture? }",
        ),
    );
    merge_into(p, "/v1/mgmt/user/update/status", post_op("updateStatus", "Update user status",
        "Valid values: `enabled`, `disabled`, `invited`.\n\n**Request:** `{ \"loginId\": \"...\", \"status\": \"disabled\" }`", t, "{ loginId, status }"));
    merge_into(
        p,
        "/v1/mgmt/user/update/name",
        post_op(
            "updateName",
            "Update display name",
            "**Request:** `{ \"loginId\": \"...\", \"name\": \"New Name\" }`",
            t,
            "{ loginId, name }",
        ),
    );
    merge_into(
        p,
        "/v1/mgmt/user/update/email",
        post_op(
            "updateEmail",
            "Update email",
            "**Request:** `{ \"loginId\": \"...\", \"email\": \"new@email.com\" }`",
            t,
            "{ loginId, email }",
        ),
    );
    merge_into(
        p,
        "/v1/mgmt/user/update/phone",
        post_op(
            "updatePhone",
            "Update phone",
            "**Request:** `{ \"loginId\": \"...\", \"phone\": \"+1555...\" }`",
            t,
            "{ loginId, phone }",
        ),
    );
    merge_into(
        p,
        "/v1/mgmt/user/update/picture",
        post_op(
            "updatePicture",
            "Update profile picture",
            "**Request:** `{ \"loginId\": \"...\", \"picture\": \"https://cdn/avatar.png\" }`",
            t,
            "{ loginId, picture }",
        ),
    );
    merge_into(p, "/v1/mgmt/user/update/loginid", post_op("updateLoginId", "Update login ID",
        "Renames login ID. Old ID becomes invalid.\n\n**Request:** `{ \"loginId\": \"old@ex.com\", \"newLoginId\": \"new@ex.com\" }`", t, "{ loginId, newLoginId }"));
    merge_into(p, "/v1/mgmt/user/update/customAttribute", post_op("updateCustomAttribute", "Update single custom attribute",
        "Sets one key in user's custom attributes. Others preserved.\n\n**Request:** `{ \"loginId\": \"...\", \"attributeKey\": \"plan\", \"attributeValue\": \"enterprise\" }`", t, "{ loginId, attributeKey, attributeValue }"));
    merge_into(
        p,
        "/v1/mgmt/user/delete",
        post_op(
            "deleteUser",
            "Delete user",
            "Permanently deletes a user.\n\n**Request:** `{ \"loginId\": \"...\" }`",
            t,
            "{ loginId }",
        ),
    );
    merge_into(p, "/v1/mgmt/user/create/batch", post_op("batchCreateUsers", "Batch create users",
        "Creates multiple users.\n\n**Request:** `{ \"users\": [{ \"loginId\": \"a@test.com\" }, ...] }`", t, "{ users }"));
    merge_into(p, "/v1/mgmt/user/delete/batch", post_op("batchDeleteUsers", "Batch delete users",
        "Deletes multiple users. Skips unknown IDs.\n\n**Request:** `{ \"loginIds\": [\"a\", \"b\"] }`", t, "{ loginIds }"));
    merge_into(p, "/v1/mgmt/user/logout", post_op("forceLogout", "Force logout user",
        "Invalidates all sessions. Refresh tokens rejected.\n\n**Request:** `{ \"loginId\": \"...\" }`", t, "{ loginId }"));
    merge_into(
        p,
        "/v1/mgmt/jwt/update",
        post_op(
            "updateJwt",
            "Update JWT claims",
            "Updates custom claims on an existing JWT.",
            t,
            "{ jwt, customClaims }",
        ),
    );
}

fn add_mgmt_user_roles(p: &mut Map<String, Value>) {
    let t = "Mgmt: User Roles";
    merge_into(p, "/v1/mgmt/user/update/role/set", post_op("setRoles", "Set user roles (replace)",
        "Replaces all project-level roles.\n\n**Request:** `{ \"loginId\": \"...\", \"roleNames\": [\"admin\"] }`", t, "{ loginId, roleNames }"));
    merge_into(p, "/v1/mgmt/user/update/role/add", post_op("addRoles", "Add roles (append)",
        "Appends roles without replacing. Deduplicates.\n\n**Request:** `{ \"loginId\": \"...\", \"roleNames\": [\"editor\"] }`", t, "{ loginId, roleNames }"));
    merge_into(p, "/v1/mgmt/user/update/role/remove", post_op("removeRoles", "Remove roles",
        "Removes specified roles. Idempotent.\n\n**Request:** `{ \"loginId\": \"...\", \"roleNames\": [\"admin\"] }`", t, "{ loginId, roleNames }"));
}

fn add_mgmt_user_tenants(p: &mut Map<String, Value>) {
    let t = "Mgmt: User Tenants";
    merge_into(p, "/v1/mgmt/user/tenant/add", post_op("addTenant", "Add user to tenant",
        "Assigns user to a tenant, optionally with roles.\n\n**Request:** `{ \"loginId\": \"...\", \"tenantId\": \"acme\", \"roleNames\": [\"viewer\"] }`", t, "{ loginId, tenantId, roleNames? }"));
    merge_into(
        p,
        "/v1/mgmt/user/tenant/remove",
        post_op(
            "removeTenant",
            "Remove user from tenant",
            "**Request:** `{ \"loginId\": \"...\", \"tenantId\": \"acme\" }`",
            t,
            "{ loginId, tenantId }",
        ),
    );
    merge_into(p, "/v1/mgmt/user/tenant/setRole", post_op("setTenantRoles", "Set tenant roles",
        "Replaces roles within a tenant.\n\n**Request:** `{ \"loginId\": \"...\", \"tenantId\": \"acme\", \"roleNames\": [\"admin\"] }`", t, "{ loginId, tenantId, roleNames }"));
}

fn add_mgmt_tenants(p: &mut Map<String, Value>) {
    let t = "Mgmt: Tenants";
    merge_into(
        p,
        "/v1/mgmt/tenant/create",
        post_op(
            "createTenant",
            "Create tenant",
            "Creates a new tenant.\n\n**Request:** `{ \"name\": \"Acme Corp\", \"id\": \"acme\" }`",
            t,
            "{ name, id? }",
        ),
    );
    merge_into(
        p,
        "/v1/mgmt/tenant/all",
        get_op("listTenants", "List all tenants", "Returns all tenants.", t),
    );
    merge_into(
        p,
        "/v1/mgmt/tenant/update",
        post_op(
            "updateTenant",
            "Update tenant",
            "Updates tenant name.\n\n**Request:** `{ \"id\": \"acme\", \"name\": \"Acme Inc\" }`",
            t,
            "{ id, name }",
        ),
    );
    merge_into(
        p,
        "/v1/mgmt/tenant/search",
        post_op(
            "searchTenants",
            "Search tenants",
            "Searches tenants.",
            t,
            "{ text? }",
        ),
    );
}

fn add_mgmt_permissions(p: &mut Map<String, Value>) {
    let t = "Mgmt: Permissions";
    merge_into(
        p,
        "/v1/mgmt/authz/permission",
        post_op(
            "createPermission",
            "Create permission",
            "**Request:** `{ \"name\": \"user:read\", \"description\": \"Read user profiles\" }`",
            t,
            "{ name, description? }",
        ),
    );
    merge_into(
        p,
        "/v1/mgmt/authz/permission/all",
        get_op(
            "listPermissions",
            "List all permissions",
            "Returns all permissions.",
            t,
        ),
    );
    merge_into(
        p,
        "/v1/mgmt/authz/permission/delete",
        post_op(
            "deletePermission",
            "Delete permission",
            "**Request:** `{ \"name\": \"user:read\" }`",
            t,
            "{ name }",
        ),
    );
}

fn add_mgmt_roles(p: &mut Map<String, Value>) {
    let t = "Mgmt: Roles";
    merge_into(
        p,
        "/v1/mgmt/authz/role",
        post_op(
            "createRole",
            "Create role",
            "**Request:** `{ \"name\": \"admin\", \"permissionNames\": [\"user:read\"] }`",
            t,
            "{ name, description?, permissionNames? }",
        ),
    );
    merge_into(
        p,
        "/v1/mgmt/authz/role/all",
        get_op(
            "listRoles",
            "List all roles",
            "Returns all roles with permissions.",
            t,
        ),
    );
    merge_into(
        p,
        "/v1/mgmt/authz/role/delete",
        post_op(
            "deleteRole",
            "Delete role",
            "**Request:** `{ \"name\": \"admin\" }`",
            t,
            "{ name }",
        ),
    );
}

fn add_mgmt_passwords(p: &mut Map<String, Value>) {
    let t = "Mgmt: Passwords";
    merge_into(p, "/v1/mgmt/user/password/set/temporary", post_op("setTemporaryPassword", "Set temporary password",
        "Sets a password so the user can sign in.\n\n**Request:** `{ \"loginId\": \"...\", \"password\": \"TempPass1!\" }`", t, "{ loginId, password }"));
    merge_into(p, "/v1/mgmt/user/password/set/active", post_op("setActivePassword", "Set active password",
        "Sets an active password.\n\n**Request:** `{ \"loginId\": \"...\", \"password\": \"Pass1!\" }`", t, "{ loginId, password }"));
    merge_into(
        p,
        "/v1/mgmt/user/password/expire",
        post_op(
            "expirePassword",
            "Expire password",
            "Marks password as expired.\n\n**Request:** `{ \"loginId\": \"...\" }`",
            t,
            "{ loginId }",
        ),
    );
}

fn add_mgmt_access_keys(p: &mut Map<String, Value>) {
    let t = "Mgmt: Access Keys";
    merge_into(
        p,
        "/v1/mgmt/accesskey",
        post_op(
            "createAccessKey",
            "Create access key",
            "**Request:** `{ \"name\": \"ci-pipeline\" }`",
            t,
            "{ name, roleNames?, tenantIds? }",
        ),
    );
    merge_into(
        p,
        "/v1/mgmt/accesskey/all",
        get_op(
            "listAccessKeys",
            "List all access keys",
            "Returns all access keys.",
            t,
        ),
    );
}

fn add_mgmt_custom_attrs(p: &mut Map<String, Value>) {
    let t = "Mgmt: Custom Attributes";
    merge_into(p, "/v1/mgmt/user/attribute", post_op("createCustomAttribute", "Create custom attribute definition",
        "**Request:** `{ \"name\": \"Plan\", \"machineName\": \"plan\", \"attributeType\": \"string\" }`\n\nTypes: `string`, `number`, `boolean`", t, "{ name, machineName, attributeType }"));
    merge_into(
        p,
        "/v1/mgmt/user/attribute/all",
        get_op(
            "listCustomAttributes",
            "List custom attribute definitions",
            "Returns all attribute schemas.",
            t,
        ),
    );
}

fn add_mgmt_test_helpers(p: &mut Map<String, Value>) {
    let t = "Mgmt: Test Helpers";
    merge_into(p, "/v1/mgmt/tests/generate/otp", post_op("generateTestOtp", "Generate OTP for test user",
        "Returns OTP code directly. Only for test users.\n\n**Response:** `{ \"code\": \"123456\" }`", t, "{ loginId, deliveryMethod }"));
    merge_into(p, "/v1/mgmt/tests/generate/magiclink", post_op("generateTestMagicLink", "Generate magic link for test user",
        "Returns magic link token. Only for test users.\n\n**Response:** `{ \"token\": \"...\", \"link\": \"...\" }`", t, "{ loginId, URI }"));
    merge_into(
        p,
        "/v1/mgmt/tests/generate/enchantedlink",
        post_op(
            "generateTestEnchantedLink",
            "Generate enchanted link for test user",
            "Returns enchanted link. Only for test users.",
            t,
            "{ loginId, URI }",
        ),
    );
    merge_into(p, "/v1/mgmt/user/embeddedlink", post_op("generateEmbeddedLink", "Generate embedded link",
        "Generates token exchangeable for session without user interaction.\n\n**Response:** `{ \"token\": \"...\" }`", "Mgmt: Embedded Links", "{ loginId, customClaims? }"));
}

fn add_emulator_helpers(p: &mut Map<String, Value>) {
    let t = "Emulator Helpers";
    merge_into(p, "/emulator/otp/{login_id}", get_op("getOtp", "Get OTP code",
        "Retrieves most recent OTP for a login ID. Emulator-only.\n\n**Example:** `GET /emulator/otp/user@example.com`", t));
    merge_into(
        p,
        "/emulator/snapshot",
        get_op(
            "exportSnapshot",
            "Export emulator state",
            "Exports full state as JSON. Useful for debugging or test fixtures.",
            t,
        ),
    );
    merge_into(
        p,
        "/emulator/snapshot",
        post_op(
            "importSnapshot",
            "Import emulator state",
            "Imports a previously exported snapshot.",
            t,
            "Full snapshot JSON",
        ),
    );
    merge_into(
        p,
        "/emulator/otps",
        get_op(
            "listOtps",
            "List all OTP codes",
            "Returns all pending OTP codes. Emulator-only.",
            t,
        ),
    );
}

fn add_jwks(p: &mut Map<String, Value>) {
    merge_into(p, "/.well-known/jwks.json", get_op("jwks", "JWKS endpoint",
        "Returns JSON Web Key Set for JWT validation.\n\nPoint your app to `http://localhost:4600/.well-known/jwks.json`.", "JWKS"));
}
