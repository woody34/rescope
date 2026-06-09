use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{error::EmulatorError, state::EmulatorState};

// ── POST /v1/mgmt/authz/permission ───────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePermissionRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

pub async fn create_permission(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreatePermissionRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let perm = state
        .permissions
        .write()
        .await
        .create(req.name, req.description)?;
    Ok(Json(json!({ "permission": perm })))
}

// ── GET /v1/mgmt/authz/permission/all ────────────────────────────────────────

pub async fn load_all_permissions(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let perms: Vec<_> = state
        .permissions
        .read()
        .await
        .load_all()
        .into_iter()
        .cloned()
        .collect();
    Ok(Json(json!({ "permissions": perms })))
}

// ── POST /v1/mgmt/authz/permission/update ────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePermissionRequest {
    pub name: String,
    pub new_name: Option<String>,
    pub description: Option<String>,
}

pub async fn update_permission(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdatePermissionRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state
        .permissions
        .write()
        .await
        .update(&req.name, req.new_name, req.description)?;
    Ok(Json(json!({ "ok": true })))
}

// ── DELETE /v1/mgmt/authz/permission/delete ──────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePermissionRequest {
    pub name: String,
}

pub async fn delete_permission(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeletePermissionRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.permissions.write().await.delete(&req.name)?;
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
    async fn create_and_list_permissions() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create_permission(
            State(state.clone()),
            headers.clone(),
            Json(CreatePermissionRequest {
                name: "read:reports".into(),
                description: "View reports".into(),
            }),
        )
        .await
        .unwrap();
        let result = load_all_permissions(State(state), headers).await.unwrap();
        let perms = result["permissions"].as_array().unwrap();
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0]["name"], "read:reports");
    }

    #[tokio::test]
    async fn delete_permission_works() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create_permission(
            State(state.clone()),
            headers.clone(),
            Json(CreatePermissionRequest {
                name: "del-me".into(),
                description: "".into(),
            }),
        )
        .await
        .unwrap();
        let _ = delete_permission(
            State(state.clone()),
            headers.clone(),
            Json(DeletePermissionRequest {
                name: "del-me".into(),
            }),
        )
        .await
        .unwrap();
        let result = load_all_permissions(State(state), headers).await.unwrap();
        assert_eq!(result["permissions"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn unauthorized_request_rejected() {
        let state = make_state().await;
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_static("Bearer wrong:key"),
        );
        let err = load_all_permissions(State(state), headers)
            .await
            .unwrap_err();
        assert!(matches!(err, EmulatorError::Unauthorized));
    }
}
