use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    error::EmulatorError, state::EmulatorState, store::access_key_store::TenantRoleBinding,
};

// ── POST /v1/mgmt/accesskey ──────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccessKeyRequest {
    pub name: String,
    pub expire_time: Option<u64>,
    #[serde(default)]
    pub permitted_ips: Vec<String>,
    #[serde(default)]
    pub role_names: Vec<String>,
    #[serde(default)]
    pub key_tenants: Vec<TenantRoleBinding>,
}

pub async fn create_access_key(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateAccessKeyRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let (key, raw_key) = state.access_keys.write().await.create(
        req.name,
        req.expire_time,
        req.permitted_ips,
        req.role_names,
        req.key_tenants,
        "mgmt-api".into(),
    )?;
    // clearkey is shown ONCE; strip it afterward
    Ok(Json(json!({
        "key": key,
        "cleartext": raw_key,
    })))
}

// ── GET /v1/mgmt/accesskey/all ────────────────────────────────────────────────

pub async fn load_all_access_keys(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let keys: Vec<_> = state
        .access_keys
        .read()
        .await
        .load_all()
        .into_iter()
        .cloned()
        .collect();
    Ok(Json(json!({ "keys": keys })))
}

// ── POST /v1/mgmt/accesskey/update ───────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccessKeyRequest {
    pub id: String,
    pub name: Option<String>,
    pub role_names: Option<Vec<String>>,
    pub key_tenants: Option<Vec<TenantRoleBinding>>,
}

pub async fn update_access_key(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateAccessKeyRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state
        .access_keys
        .write()
        .await
        .update(&req.id, req.name, req.role_names, req.key_tenants)?;
    Ok(Json(json!({ "ok": true })))
}

// ── DELETE /v1/mgmt/accesskey/delete ─────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAccessKeyRequest {
    pub id: String,
}

pub async fn delete_access_key(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeleteAccessKeyRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.access_keys.write().await.delete(&req.id)?;
    Ok(Json(json!({ "ok": true })))
}

// ── POST /v1/mgmt/accesskey/disable ──────────────────────────────────────────

pub async fn disable_access_key(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeleteAccessKeyRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.access_keys.write().await.disable(&req.id)?;
    Ok(Json(json!({ "ok": true })))
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::EmulatorConfig, state::EmulatorState};
    use axum::{
        extract::State,
        http::{HeaderMap, HeaderValue},
    };

    async fn make_state() -> EmulatorState {
        EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap()
    }

    fn mgmt_headers(state: &EmulatorState) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!(
                "Bearer {}:{}",
                state.config.project_id, state.config.management_key
            ))
            .unwrap(),
        );
        headers
    }

    #[tokio::test]
    async fn create_access_key_returns_cleartext() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let result = create_access_key(
            State(state),
            headers,
            Json(CreateAccessKeyRequest {
                name: "test-key".into(),
                expire_time: None,
                permitted_ips: vec![],
                role_names: vec![],
                key_tenants: vec![],
            }),
        )
        .await
        .unwrap();
        assert!(!result["cleartext"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_keys_after_create() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create_access_key(
            State(state.clone()),
            headers.clone(),
            Json(CreateAccessKeyRequest {
                name: "k1".into(),
                expire_time: None,
                permitted_ips: vec![],
                role_names: vec![],
                key_tenants: vec![],
            }),
        )
        .await
        .unwrap();
        let result = load_all_access_keys(State(state), headers).await.unwrap();
        assert_eq!(result["keys"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn unauthorized_request_rejected() {
        let state = make_state().await;
        let err = load_all_access_keys(State(state), {
            let mut h = HeaderMap::new();
            h.insert(
                axum::http::header::AUTHORIZATION,
                axum::http::HeaderValue::from_static("Bearer wrong:key"),
            );
            h
        })
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::Unauthorized));
    }
}
