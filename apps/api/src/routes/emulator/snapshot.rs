/// GET  /emulator/snapshot  — export the full emulator state as JSON
/// POST /emulator/snapshot  — import (restore) emulator state from JSON
/// GET  /emulator/otps      — list all pending OTP codes (test helper)
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    error::EmulatorError,
    state::EmulatorState,
    store::{
        access_key_store::AccessKey, auth_method_config::AuthMethodConfig,
        connector_store::Connector, custom_attribute_store::CustomAttribute,
        idp_store::IdpEmulator, jwt_template_store::JwtTemplate, permission_store::Permission,
        role_store::Role,
    },
    types::{Tenant, User},
};

// ─── Snapshot Types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorSnapshot {
    // ── Runtime (reset on /emulator/reset) ────────────────────────────────────
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub tenants: Vec<Tenant>,
    // ── Config (not reset on /emulator/reset) ─────────────────────────────────
    #[serde(default)]
    pub permissions: Vec<Permission>,
    #[serde(default)]
    pub roles: Vec<Role>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_method_config: Option<AuthMethodConfig>,
    #[serde(default)]
    pub jwt_templates: Vec<JwtTemplate>,
    #[serde(default)]
    pub connectors: Vec<Connector>,
    #[serde(default)]
    pub custom_attributes: Vec<CustomAttribute>,
    #[serde(default)]
    pub access_keys: Vec<AccessKey>,
    #[serde(default)]
    pub idp_emulators: Vec<IdpEmulator>,
    /// RSA key pair (PKCS8 PEM). Present in export; if absent on import, a new key pair is generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<String>,
}

// ── GET /emulator/snapshot ────────────────────────────────────────────────────

pub async fn export(
    State(state): State<EmulatorState>,
) -> Result<Json<EmulatorSnapshot>, EmulatorError> {
    let snapshot = EmulatorSnapshot {
        users: state.users.read().await.snapshot(),
        tenants: state.tenants.read().await.snapshot(),
        permissions: state.permissions.read().await.snapshot(),
        roles: state.roles.read().await.snapshot(),
        auth_method_config: Some(state.auth_method_config.read().await.get().clone()),
        jwt_templates: state.jwt_templates.read().await.snapshot(),
        connectors: state.connectors.read().await.snapshot(),
        custom_attributes: state.custom_attributes.read().await.snapshot(),
        access_keys: state.access_keys.read().await.snapshot(),
        idp_emulators: state.idp_emulators.read().await.snapshot(),
        keys: Some(state.km().await.private_pem.clone()),
    };
    Ok(Json(snapshot))
}

// ── POST /emulator/snapshot ───────────────────────────────────────────────────

pub async fn import(
    State(state): State<EmulatorState>,
    Json(snap): Json<EmulatorSnapshot>,
) -> Result<Json<Value>, EmulatorError> {
    // Reset runtime stores first
    state.reset_stores().await;

    // Restore users & tenants
    if !snap.users.is_empty() {
        state.users.write().await.restore(snap.users);
    }
    if !snap.tenants.is_empty() {
        state.tenants.write().await.restore(snap.tenants);
    }

    // Restore config stores
    if !snap.permissions.is_empty() {
        state.permissions.write().await.restore(snap.permissions);
    }
    if !snap.roles.is_empty() {
        state.roles.write().await.restore(snap.roles);
    }
    if let Some(cfg) = snap.auth_method_config {
        state.auth_method_config.write().await.restore(cfg);
    }
    if !snap.jwt_templates.is_empty() {
        state
            .jwt_templates
            .write()
            .await
            .restore(snap.jwt_templates);
    }
    if !snap.connectors.is_empty() {
        state.connectors.write().await.restore(snap.connectors);
    }
    if !snap.custom_attributes.is_empty() {
        state
            .custom_attributes
            .write()
            .await
            .restore(snap.custom_attributes);
    }
    if !snap.access_keys.is_empty() {
        state.access_keys.write().await.restore(snap.access_keys);
    }
    if !snap.idp_emulators.is_empty() {
        state
            .idp_emulators
            .write()
            .await
            .restore(snap.idp_emulators);
    }

    // Restore RSA key pair if present in snapshot
    if let Some(pem) = snap.keys {
        let new_keys = crate::jwt::key_manager::KeyManager::from_private_pem(&pem)
            .map_err(|e| EmulatorError::Internal(format!("snapshot key restore failed: {e}")))?;
        // Atomically replace state.keys (Arc swap)
        state.swap_keys(new_keys);
    }

    Ok(Json(json!({ "ok": true })))
}

// ── GET /emulator/otps ────────────────────────────────────────────────────────

pub async fn list_otps(State(state): State<EmulatorState>) -> Json<Value> {
    let otps_by_uid = state.otps.read().await.list_all();
    let users = state.users.read().await;
    // Map userId → loginId (first loginId), falling back to userId if not found
    let otps: std::collections::HashMap<String, String> = otps_by_uid
        .into_iter()
        .map(|(uid, code)| {
            let login_id = users
                .load_by_user_id(&uid)
                .ok()
                .and_then(|u| u.login_ids.first().cloned())
                .unwrap_or(uid);
            (login_id, code)
        })
        .collect();
    Json(json!({ "otps": otps }))
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::EmulatorConfig, state::EmulatorState};
    use axum::extract::State;

    async fn make_state() -> EmulatorState {
        EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn export_returns_snapshot_with_empty_stores() {
        let state = make_state().await;
        let result = export(State(state)).await.unwrap();
        let snap = result.0;
        assert!(snap.users.is_empty());
        assert!(snap.tenants.is_empty());
        assert!(snap.permissions.is_empty());
        assert!(snap.auth_method_config.is_some()); // defaults always present
    }

    #[tokio::test]
    async fn import_then_export_roundtrip() {
        let state = make_state().await;

        // Build a minimal snapshot with a permission
        let mut snap = EmulatorSnapshot::default();
        snap.permissions = vec![crate::store::permission_store::Permission {
            id: "p1".into(),
            name: "read:all".into(),
            description: "Read everything".into(),
        }];

        let _ = import(State(state.clone()), Json(snap)).await.unwrap();

        let result = export(State(state)).await.unwrap();
        assert_eq!(result.0.permissions.len(), 1);
        assert_eq!(result.0.permissions[0].name, "read:all");
    }
}
