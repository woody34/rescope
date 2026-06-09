use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{
    error::EmulatorError,
    state::EmulatorState,
    store::idp_store::{IdpEmulator, IdpProtocol},
};

// ── POST /v1/mgmt/idp ────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIdpRequest {
    pub protocol: IdpProtocol,
    pub display_name: String,
    pub tenant_id: String,
    #[serde(default)]
    pub attribute_mapping: HashMap<String, String>,
}

pub async fn create_idp(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateIdpRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let idp = IdpEmulator {
        id: String::new(),
        protocol: req.protocol,
        display_name: req.display_name,
        tenant_id: req.tenant_id,
        attribute_mapping: req.attribute_mapping,
    };
    let created = state.idp_emulators.write().await.insert(idp)?;
    Ok(Json(json!({ "idp": created })))
}

// ── GET /v1/mgmt/idp/all ─────────────────────────────────────────────────────

pub async fn load_all_idps(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let idps: Vec<_> = state
        .idp_emulators
        .read()
        .await
        .list()
        .into_iter()
        .cloned()
        .collect();
    Ok(Json(json!({ "idps": idps })))
}

// ── POST /v1/mgmt/idp/update ─────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIdpRequest {
    pub id: String,
    pub display_name: Option<String>,
    pub attribute_mapping: Option<HashMap<String, String>>,
}

pub async fn update_idp(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateIdpRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state
        .idp_emulators
        .write()
        .await
        .update(&req.id, req.display_name, req.attribute_mapping)?;
    Ok(Json(json!({ "ok": true })))
}

// ── POST /v1/mgmt/idp/delete ─────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteIdpRequest {
    pub id: String,
}

pub async fn delete_idp(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeleteIdpRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.idp_emulators.write().await.delete(&req.id)?;
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
    async fn create_and_list_idps() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create_idp(
            State(state.clone()),
            headers.clone(),
            Json(CreateIdpRequest {
                protocol: IdpProtocol::Oidc,
                display_name: "Mock Okta".into(),
                tenant_id: "acme".into(),
                attribute_mapping: HashMap::new(),
            }),
        )
        .await
        .unwrap();
        let result = load_all_idps(State(state), headers).await.unwrap();
        assert_eq!(result["idps"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn update_idp_display_name() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let created = create_idp(
            State(state.clone()),
            headers.clone(),
            Json(CreateIdpRequest {
                protocol: IdpProtocol::Saml,
                display_name: "Old Name".into(),
                tenant_id: "acme".into(),
                attribute_mapping: HashMap::new(),
            }),
        )
        .await
        .unwrap();
        let id = created["idp"]["id"].as_str().unwrap().to_string();
        update_idp(
            State(state.clone()),
            headers.clone(),
            Json(UpdateIdpRequest {
                id: id.clone(),
                display_name: Some("New Name".into()),
                attribute_mapping: None,
            }),
        )
        .await
        .unwrap();
        let list = load_all_idps(State(state), headers).await.unwrap();
        let idp = &list["idps"].as_array().unwrap()[0];
        assert_eq!(idp["displayName"], "New Name");
    }

    #[tokio::test]
    async fn delete_idp_works() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let created = create_idp(
            State(state.clone()),
            headers.clone(),
            Json(CreateIdpRequest {
                protocol: IdpProtocol::Oidc,
                display_name: "To Delete".into(),
                tenant_id: "acme".into(),
                attribute_mapping: HashMap::new(),
            }),
        )
        .await
        .unwrap();
        let id = created["idp"]["id"].as_str().unwrap().to_string();
        delete_idp(
            State(state.clone()),
            headers.clone(),
            Json(DeleteIdpRequest { id }),
        )
        .await
        .unwrap();
        let list = load_all_idps(State(state), headers).await.unwrap();
        assert_eq!(list["idps"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn unauthorized_request_rejected() {
        let state = make_state().await;
        let err = load_all_idps(State(state), {
            let mut h = HeaderMap::new();
            h.insert(
                axum::http::header::AUTHORIZATION,
                HeaderValue::from_static("Bearer wrong:key"),
            );
            h
        })
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::Unauthorized));
    }
}
