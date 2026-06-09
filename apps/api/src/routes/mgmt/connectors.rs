use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{error::EmulatorError, state::EmulatorState, store::connector_store::ConnectorType};

// ── POST /v1/mgmt/connector ───────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectorRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub connector_type: ConnectorType,
    pub config: Value,
}

pub async fn create_connector(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateConnectorRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let connector =
        state
            .connectors
            .write()
            .await
            .create(req.name, req.connector_type, req.config)?;
    Ok(Json(json!({ "connector": connector })))
}

// ── GET /v1/mgmt/connector/all ────────────────────────────────────────────────

pub async fn load_all_connectors(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let connectors: Vec<_> = state
        .connectors
        .read()
        .await
        .load_all()
        .into_iter()
        .cloned()
        .collect();
    Ok(Json(json!({ "connectors": connectors })))
}

// ── POST /v1/mgmt/connector/update ───────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConnectorRequest {
    pub id: String,
    pub name: Option<String>,
    pub config: Option<Value>,
}

pub async fn update_connector(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateConnectorRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state
        .connectors
        .write()
        .await
        .update(&req.id, req.name, req.config)?;
    Ok(Json(json!({ "ok": true })))
}

// ── DELETE /v1/mgmt/connector/delete ─────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteConnectorRequest {
    pub id: String,
}

pub async fn delete_connector(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeleteConnectorRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.connectors.write().await.delete(&req.id)?;
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
    use serde_json::json;

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
    async fn create_and_list_connectors() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create_connector(
            State(state.clone()),
            headers.clone(),
            Json(CreateConnectorRequest {
                name: "my-webhook".into(),
                connector_type: ConnectorType::GenericHttp,
                config: json!({ "baseUrl": "https://example.com" }),
            }),
        )
        .await
        .unwrap();
        let result = load_all_connectors(State(state), headers).await.unwrap();
        assert_eq!(result["connectors"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn unauthorized_request_rejected() {
        let state = make_state().await;
        let err = load_all_connectors(State(state), {
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
