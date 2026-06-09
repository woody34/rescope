use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── User ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub user_id: String,
    pub login_ids: Vec<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
    pub family_name: Option<String>,
    pub verified_email: bool,
    pub verified_phone: bool,
    pub picture: Option<String>,
    pub role_names: Vec<String>,
    pub user_tenants: Vec<UserTenant>,
    pub status: String,
    pub created_time: u64,
    pub custom_attributes: HashMap<String, serde_json::Value>,
    pub totp: bool,
    pub saml: bool,
    pub password: bool,
    pub oauth: HashMap<String, bool>,
    pub last_login: Option<u64>,

    // Internal fields — preserved in snapshots but excluded from UserResponse
    #[serde(default, skip_serializing_if = "bool::clone", rename = "__isTestUser")]
    pub _is_test_user: bool,
    /// Bcrypt hash included in snapshot for round-trip fidelity.
    /// Named with double-underscore prefix to clearly mark as internal.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "__passwordHash"
    )]
    pub _password_hash: Option<String>,
}

impl User {
    pub fn to_response(&self) -> UserResponse {
        UserResponse {
            user_id: self.user_id.clone(),
            login_ids: self.login_ids.clone(),
            email: self.email.clone(),
            phone: self.phone.clone(),
            name: self.name.clone(),
            given_name: self.given_name.clone(),
            middle_name: self.middle_name.clone(),
            family_name: self.family_name.clone(),
            verified_email: self.verified_email,
            verified_phone: self.verified_phone,
            picture: self.picture.clone(),
            role_names: self.role_names.clone(),
            user_tenants: self.user_tenants.clone(),
            status: self.status.clone(),
            created_time: self.created_time,
            custom_attributes: self.custom_attributes.clone(),
            totp: self.totp,
            saml: self.saml,
            password: self.password,
            oidc: false,
            scim: false,
            oauth: self.oauth.clone(),
            push: false,
            webauthn: false,
            test: self._is_test_user,
            external_ids: self.login_ids.clone(),
            sso_app_ids: vec![],
            permissions: vec![],
            recovery_email: String::new(),
            recovery_phone: String::new(),
            verified_recovery_email: false,
            verified_recovery_phone: false,
            last_login: self.last_login,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserTenant {
    pub tenant_id: String,
    #[serde(default)]
    pub tenant_name: String,
    pub role_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub user_id: String,
    pub login_ids: Vec<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
    pub family_name: Option<String>,
    pub verified_email: bool,
    pub verified_phone: bool,
    pub picture: Option<String>,
    pub role_names: Vec<String>,
    pub user_tenants: Vec<UserTenant>,
    pub status: String,
    pub created_time: u64,
    pub custom_attributes: HashMap<String, serde_json::Value>,
    // Auth method booleans — real Descope uses uppercase keys
    #[serde(rename = "TOTP")]
    pub totp: bool,
    #[serde(rename = "SAML")]
    pub saml: bool,
    pub password: bool,
    #[serde(rename = "OIDC")]
    pub oidc: bool,
    #[serde(rename = "SCIM")]
    pub scim: bool,
    #[serde(rename = "OAuth")]
    pub oauth: HashMap<String, bool>,
    pub push: bool,
    pub webauthn: bool,
    pub test: bool,
    #[serde(rename = "externalIds")]
    pub external_ids: Vec<String>,
    #[serde(rename = "ssoAppIds")]
    pub sso_app_ids: Vec<String>,
    pub permissions: Vec<String>,
    // Recovery-contact fields — real Descope always returns these (empty string /
    // false by default). Required for UserResponse SDK parity; consumers assert on
    // them across create/find/query, so they must always be present.
    pub recovery_email: String,
    pub recovery_phone: String,
    pub verified_recovery_email: bool,
    pub verified_recovery_phone: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Why: every auth surface consumes the SDK `UserResponse`, and real Descope
    ///      always returns the recovery-contact fields. They were missing from the
    ///      emulator, breaking `UserResponse` parity — downstream `create`/`find`/
    ///      `query` consumers assert `recoveryEmail`/`recoveryPhone`/
    ///      `verifiedRecovery*`, so a dropped field fails those contracts.
    /// Decision: lock the serialized shape so the fields cannot silently regress.
    #[test]
    fn to_response_serializes_recovery_fields_with_defaults() {
        let json = serde_json::to_value(User::default().to_response()).expect("serialize");
        assert_eq!(json["recoveryEmail"], "");
        assert_eq!(json["recoveryPhone"], "");
        assert_eq!(json["verifiedRecoveryEmail"], false);
        assert_eq!(json["verifiedRecoveryPhone"], false);
    }
}

// ─── SAML / OIDC config embedded in Tenant ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SamlConfig {
    pub metadata_url: Option<String>,
    pub entity_id: Option<String>,
    pub acs_url: Option<String>,
    pub certificate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OidcConfig {
    pub discovery_url: Option<String>,
    pub issuer: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    /// Attribute mapping: IdP claim → Descope user field.
    #[serde(default)]
    pub attribute_mapping: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub self_provisioning_domains: Vec<String>,
    pub custom_attributes: HashMap<String, serde_json::Value>,
    pub domains: Vec<String>,
    pub auth_type: AuthType,
    // ── SAML / OIDC ──────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saml_config: Option<SamlConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oidc_config: Option<OidcConfig>,
    // ── Per-tenant session overrides ─────────────────────────────────────────
    /// Override project-wide session TTL (seconds). None = use project default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token_ttl_override: Option<u64>,
    /// Override project-wide refresh TTL (seconds). None = use project default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token_ttl_override: Option<u64>,
    pub enforce_sso: bool,
    /// Sub-tenant support: optional parent tenant ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tenant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    #[default]
    None,
    Saml,
    Oidc,
}

// ─── Token Store Entries ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TokenEntry {
    pub user_id: String,
    pub token_type: TokenType,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Magic,
    Saml,
    Embedded,
    Reset,
}

// ─── Auth Responses ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtResponse {
    pub session_jwt: String,
    pub refresh_jwt: String,
    pub cookie_domain: String,
    pub cookie_path: String,
    pub cookie_max_age: u64,
    pub cookie_expiration: u64,
    pub first_seen: bool,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationInfo {
    pub jwt: String,
    pub token: serde_json::Value,
    pub cookies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetResponse {
    pub reset_method: String,
    pub masked_email: String,
}

// ─── SDK Response wrapper ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct SdkResponse<T: Serialize> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> SdkResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
        }
    }

    pub fn ok_empty() -> SdkResponse<()> {
        SdkResponse {
            ok: true,
            data: None,
        }
    }
}

// ─── JWT Claims ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TenantClaim {
    #[serde(rename = "roleNames", default)]
    pub role_names: Vec<String>,
    /// Permissions derived from the tenant roles. Descope embeds these so
    /// backends can do permission checks without extra DB lookups.
    #[serde(rename = "permissionNames", default)]
    pub permission_names: Vec<String>,
    /// Human-readable tenant name embedded for convenience.
    #[serde(rename = "tenantName", default)]
    pub tenant_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionClaims {
    pub sub: String,
    pub iss: String,
    pub exp: u64,
    pub iat: u64,
    /// Descope Resource Name — always "DS" for session tokens.
    #[serde(default)]
    pub drn: String,
    /// Authentication Method Reference — e.g. ["email"], ["pwd"].
    #[serde(default)]
    pub amr: Vec<String>,
    /// Global (project-level) roles assigned to this user.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,
    pub email: String,
    pub email_verified: bool,
    pub phone: String,
    pub phone_verified: bool,
    pub name: String,
    pub username: String,
    pub company: String,
    pub uid: String,
    pub photo_url: String,
    #[serde(rename = "super")]
    pub is_super: bool,
    pub support: bool,
    #[serde(rename = "descopeId")]
    pub descope_id: String,
    /// Map of tenantId -> tenant claims (roles, permissions, name). Omitted when empty.
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty", default)]
    pub tenants: std::collections::HashMap<String, TenantClaim>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshClaims {
    pub sub: String,
    pub iss: String,
    pub exp: u64,
    pub iat: u64,
    /// Descope Resource Name — always "DSR" for refresh tokens.
    #[serde(default)]
    pub drn: String,
}
