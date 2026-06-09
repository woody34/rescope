use crate::store::idp_store::IdpEmulator;
use crate::{
    error::EmulatorError,
    state::EmulatorState,
    store::{
        auth_method_config::AuthMethodConfig, connector_store::Connector,
        jwt_template_store::JwtTemplate, permission_store::Permission, role_store::Role,
        user_store::new_user_id,
    },
    types::{AuthType, OidcConfig, SamlConfig, Tenant, User, UserTenant},
};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedFile {
    #[serde(default)]
    tenants: Vec<SeedTenant>,
    #[serde(default)]
    users: Vec<SeedUser>,
    /// Pre-defined permissions (loaded before roles).
    #[serde(default)]
    permissions: Vec<Permission>,
    /// Pre-defined roles.
    #[serde(default)]
    roles: Vec<Role>,
    /// Full auth method config to apply on startup.
    #[serde(default)]
    auth_method_config: Option<AuthMethodConfig>,
    /// JWT template definitions.
    #[serde(default)]
    jwt_templates: Vec<JwtTemplate>,
    /// Connector definitions.
    #[serde(default)]
    connectors: Vec<Connector>,
    /// IdP emulator configurations.
    #[serde(default)]
    idp_emulators: Vec<IdpEmulator>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SeedTenant {
    id: String,
    name: Option<String>,
    domains: Option<Vec<String>>,
    auth_type: Option<String>,
    saml_config: Option<SamlConfig>,
    oidc_config: Option<OidcConfig>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedUser {
    login_id: String,
    email: Option<String>,
    phone: Option<String>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    password: Option<String>,
    is_test_user: Option<bool>,
    verified_email: Option<bool>,
    verified_phone: Option<bool>,
    custom_attributes: Option<HashMap<String, serde_json::Value>>,
    tenant_ids: Option<Vec<String>>,
    role_names: Option<Vec<String>>,
    saml: Option<bool>,
}

pub async fn load(path: &str, state: &EmulatorState) -> Result<()> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("Cannot read seed file: {path}"))?;
    let seed: SeedFile = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON in seed file: {path}"))?;

    // ── 1. Permissions (must be before roles) ────────────────────────────────
    if !seed.permissions.is_empty() {
        let mut perm_store = state.permissions.write().await;
        perm_store.restore(seed.permissions);
    }

    // ── 2. Roles ─────────────────────────────────────────────────────────────
    if !seed.roles.is_empty() {
        let mut role_store = state.roles.write().await;
        role_store.restore(seed.roles);
    }

    // ── 3. Auth method config ─────────────────────────────────────────────────
    if let Some(cfg) = seed.auth_method_config {
        state.auth_method_config.write().await.restore(cfg);
    }

    // ── 4. JWT templates ──────────────────────────────────────────────────────
    if !seed.jwt_templates.is_empty() {
        state
            .jwt_templates
            .write()
            .await
            .restore(seed.jwt_templates);
    }

    // ── 5. Connectors ─────────────────────────────────────────────────────────
    if !seed.connectors.is_empty() {
        state.connectors.write().await.restore(seed.connectors);
    }

    // ── 6. Tenants (must be before users) ────────────────────────────────────
    {
        let mut tenant_store = state.tenants.write().await;
        for st in &seed.tenants {
            let auth_type = match st.auth_type.as_deref() {
                Some("saml") => AuthType::Saml,
                Some("oidc") => AuthType::Oidc,
                _ => AuthType::None,
            };
            tenant_store.insert(Tenant {
                id: st.id.clone(),
                name: st.name.clone().unwrap_or_else(|| st.id.clone()),
                domains: st.domains.clone().unwrap_or_default(),
                auth_type,
                saml_config: st.saml_config.clone(),
                oidc_config: st.oidc_config.clone(),
                ..Default::default()
            });
        }
    }

    // ── 6b. IdP Emulators ────────────────────────────────────────────────────
    if !seed.idp_emulators.is_empty() {
        let mut idp_store = state.idp_emulators.write().await;
        for idp in seed.idp_emulators {
            idp_store.insert(idp)?;
        }
    }

    // ── 7. Users ──────────────────────────────────────────────────────────────
    let tenant_store_snap: Vec<SeedTenant> = seed.tenants.clone();

    for su in seed.users {
        let email = su.email.clone().or_else(|| {
            if su.login_id.contains('@') {
                Some(su.login_id.clone())
            } else {
                None
            }
        });

        let user_tenants: Vec<UserTenant> = su
            .tenant_ids
            .unwrap_or_default()
            .into_iter()
            .map(|tid| {
                let name = tenant_store_snap
                    .iter()
                    .find(|t| t.id == tid)
                    .and_then(|t| t.name.clone())
                    .unwrap_or_else(|| tid.clone());
                UserTenant {
                    tenant_id: tid,
                    tenant_name: name,
                    role_names: vec![],
                }
            })
            .collect();

        let password_hash = if let Some(pwd) = su.password {
            let hash = tokio::task::spawn_blocking(move || {
                bcrypt::hash(&pwd, 10).map_err(|e| EmulatorError::Internal(e.to_string()))
            })
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))??;
            Some(hash)
        } else {
            None
        };

        // Honor a caller-provided `uid` as the userId. The backend stores that uid
        // as the user's descopeId and resolves admins via
        // findById(descopeId)/loadByUserId, so the emulator's userId must equal the
        // uid or those lookups miss. When no uid is supplied, fall back to a
        // generated id.
        let user_id = su
            .custom_attributes
            .as_ref()
            .and_then(|a| a.get("uid"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(new_user_id);
        let user = User {
            user_id,
            login_ids: vec![su.login_id.clone()],
            email,
            phone: su.phone,
            name: su.name,
            given_name: su.given_name,
            family_name: su.family_name,
            verified_email: su.verified_email.unwrap_or(false),
            verified_phone: su.verified_phone.unwrap_or(false),
            custom_attributes: su.custom_attributes.unwrap_or_default(),
            user_tenants,
            role_names: su.role_names.unwrap_or_default(),
            saml: su.saml.unwrap_or(false),
            password: password_hash.is_some(),
            status: "enabled".into(),
            created_time: now(),
            _is_test_user: su.is_test_user.unwrap_or(false),
            _password_hash: password_hash,
            ..Default::default()
        };

        state
            .users
            .write()
            .await
            .insert(user)
            .with_context(|| format!("Failed to insert seed user: {}", su.login_id))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::EmulatorConfig, state::EmulatorState};
    use std::io::Write;
    use tempfile::NamedTempFile;

    async fn test_state() -> EmulatorState {
        EmulatorState::new(&EmulatorConfig::from_env())
            .await
            .unwrap()
    }

    fn write_seed(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[tokio::test]
    async fn loads_tenants_before_users() {
        let state = test_state().await;
        let seed_content = r#"{
            "tenants": [{ "id": "t1", "name": "Acme", "domains": ["acme.com"], "authType": "saml" }],
            "users": [{ "loginId": "alice@acme.com", "tenantIds": ["t1"] }]
        }"#;
        let f = write_seed(seed_content);
        load(f.path().to_str().unwrap(), &state).await.unwrap();

        let users = state.users.read().await;
        let user = users.load("alice@acme.com").unwrap();
        assert_eq!(user.user_tenants.len(), 1);
        assert_eq!(user.user_tenants[0].tenant_id, "t1");
    }

    #[tokio::test]
    async fn missing_file_errors() {
        let state = test_state().await;
        let result = load("/nonexistent/seed.json", &state).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn invalid_json_errors() {
        let state = test_state().await;
        let f = write_seed("not json at all");
        let result = load(f.path().to_str().unwrap(), &state).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn seeded_user_with_password_can_sign_in() {
        let state = test_state().await;
        let seed_content = r#"{"users": [{ "loginId": "bob@test.com", "password": "secret123" }]}"#;
        let f = write_seed(seed_content);
        load(f.path().to_str().unwrap(), &state).await.unwrap();

        let users = state.users.read().await;
        let result = users.check_password("bob@test.com", "secret123");
        assert!(result.is_ok());
    }
}
