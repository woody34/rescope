//! Descope management: project snapshot export.
//!
//! `POST /v1/mgmt/project/snapshot/export` — returns the project configuration
//! snapshot in Descope's `{ "files": { ... } }` shape. Matching cloud Descope,
//! **users and tenants are not part of a project snapshot** (they are exported
//! separately via the user/tenant management APIs).
use axum::{extract::State, Json};
use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::{error::EmulatorError, state::EmulatorState};

fn to_value<T: Serialize>(value: T) -> Result<Value, EmulatorError> {
    serde_json::to_value(value)
        .map_err(|e| EmulatorError::Internal(format!("snapshot serialize failed: {e}")))
}

// ── POST /v1/mgmt/project/snapshot/export ──────────────────────────────────────

pub async fn export_snapshot(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;

    let mut files = Map::new();
    files.insert(
        "roles.json".into(),
        to_value(state.roles.read().await.snapshot())?,
    );
    files.insert(
        "permissions.json".into(),
        to_value(state.permissions.read().await.snapshot())?,
    );
    files.insert(
        "connectors.json".into(),
        to_value(state.connectors.read().await.snapshot())?,
    );
    files.insert(
        "jwtTemplates.json".into(),
        to_value(state.jwt_templates.read().await.snapshot())?,
    );
    files.insert(
        "customAttributes.json".into(),
        to_value(state.custom_attributes.read().await.snapshot())?,
    );
    files.insert(
        "authMethodConfig.json".into(),
        to_value(state.auth_method_config.read().await.get().clone())?,
    );

    Ok(Json(json!({ "files": Value::Object(files) })))
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::EmulatorConfig, state::EmulatorState};
    use axum::{extract::State, http::HeaderMap};

    #[tokio::test]
    async fn export_snapshot_returns_files_map() {
        let state = EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap();
        let res = export_snapshot(State(state), HeaderMap::new())
            .await
            .unwrap();
        let files = res.0.get("files").unwrap().as_object().unwrap().clone();
        assert!(files.contains_key("roles.json"));
        assert!(files.contains_key("permissions.json"));
        assert!(files.contains_key("authMethodConfig.json"));
    }

    #[tokio::test]
    async fn export_snapshot_reflects_created_role() {
        let state = EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap();
        state
            .roles
            .write()
            .await
            .create("Auditor".into(), "Read-only".into(), vec![])
            .unwrap();

        let res = export_snapshot(State(state), HeaderMap::new())
            .await
            .unwrap();
        let roles = res.0["files"]["roles.json"].as_array().unwrap().clone();
        assert!(roles.iter().any(|r| r["name"] == "Auditor"));
    }
}
