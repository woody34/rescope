use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    error::EmulatorError,
    state::EmulatorState,
    store::custom_attribute_store::{AttributePermissions, AttributeType},
};

// ── POST /v1/mgmt/user/attribute ─────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttributeRequest {
    pub name: String,
    pub machine_name: String,
    pub attribute_type: AttributeType,
    pub permissions: AttributePermissions,
}

pub async fn create_attribute(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateAttributeRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let attr = state.custom_attributes.write().await.create(
        req.name,
        req.machine_name,
        req.attribute_type,
        req.permissions,
    )?;
    Ok(Json(json!({ "attribute": attr })))
}

// ── GET /v1/mgmt/user/attribute/all ──────────────────────────────────────────

pub async fn load_all_attributes(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let attrs: Vec<_> = state
        .custom_attributes
        .read()
        .await
        .load_all()
        .into_iter()
        .cloned()
        .collect();
    Ok(Json(json!({ "attributes": attrs })))
}

// ── DELETE /v1/mgmt/user/attribute/delete ────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAttributeRequest {
    pub machine_name: String,
}

pub async fn delete_attribute(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeleteAttributeRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state
        .custom_attributes
        .write()
        .await
        .delete(&req.machine_name)?;
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
    async fn create_and_list_attributes() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create_attribute(
            State(state.clone()),
            headers.clone(),
            Json(CreateAttributeRequest {
                name: "Plan".into(),
                machine_name: "plan".into(),
                attribute_type: AttributeType::Text,
                permissions: AttributePermissions::All,
            }),
        )
        .await
        .unwrap();
        let result = load_all_attributes(State(state), headers).await.unwrap();
        assert_eq!(result["attributes"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn delete_attribute_works() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create_attribute(
            State(state.clone()),
            headers.clone(),
            Json(CreateAttributeRequest {
                name: "Score".into(),
                machine_name: "score".into(),
                attribute_type: AttributeType::Number,
                permissions: AttributePermissions::Admin,
            }),
        )
        .await
        .unwrap();
        let _ = delete_attribute(
            State(state.clone()),
            headers.clone(),
            Json(DeleteAttributeRequest {
                machine_name: "score".into(),
            }),
        )
        .await
        .unwrap();
        let result = load_all_attributes(State(state), headers).await.unwrap();
        assert_eq!(result["attributes"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn unauthorized_request_rejected() {
        let state = make_state().await;
        let err = load_all_attributes(State(state), {
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
