use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{error::EmulatorError, state::EmulatorState};

// ── POST /v1/mgmt/authz/role ──────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub permission_names: Vec<String>,
}

pub async fn create_role(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateRoleRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let role = state
        .roles
        .write()
        .await
        .create(req.name, req.description, req.permission_names)?;
    Ok(Json(json!({ "role": role })))
}

// ── GET /v1/mgmt/authz/role/all ──────────────────────────────────────────────

pub async fn load_all_roles(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let roles: Vec<_> = state
        .roles
        .read()
        .await
        .load_all()
        .into_iter()
        .cloned()
        .collect();
    Ok(Json(json!({ "roles": roles })))
}

// ── POST /v1/mgmt/authz/role/update ──────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleRequest {
    pub name: String,
    pub new_name: Option<String>,
    pub description: Option<String>,
    pub permission_names: Option<Vec<String>>,
}

pub async fn update_role(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.roles.write().await.update(
        &req.name,
        req.new_name,
        req.description,
        req.permission_names,
        None,
        None,
    )?;
    Ok(Json(json!({ "ok": true })))
}

// ── DELETE /v1/mgmt/authz/role/delete ────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRoleRequest {
    pub name: String,
}

pub async fn delete_role(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeleteRoleRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.roles.write().await.delete(&req.name)?;
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
    async fn create_and_list_roles() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create_role(
            State(state.clone()),
            headers.clone(),
            Json(CreateRoleRequest {
                name: "viewer".into(),
                description: "Read only".into(),
                permission_names: vec!["read".into()],
            }),
        )
        .await
        .unwrap();
        let result = load_all_roles(State(state), headers).await.unwrap();
        let roles = result["roles"].as_array().unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0]["name"], "viewer");
    }

    #[tokio::test]
    async fn delete_role_works() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create_role(
            State(state.clone()),
            headers.clone(),
            Json(CreateRoleRequest {
                name: "to-delete".into(),
                description: "".into(),
                permission_names: vec![],
            }),
        )
        .await
        .unwrap();
        let _ = delete_role(
            State(state.clone()),
            headers.clone(),
            Json(DeleteRoleRequest {
                name: "to-delete".into(),
            }),
        )
        .await
        .unwrap();
        let result = load_all_roles(State(state), headers).await.unwrap();
        assert_eq!(result["roles"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn unauthorized_request_rejected() {
        let state = make_state().await;
        let err = load_all_roles(State(state), {
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
