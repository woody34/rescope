use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{error::EmulatorError, state::EmulatorState, store::jwt_template_store::JwtTemplate};

// ── POST /v1/mgmt/jwt/template ───────────────────────────────────────────────

pub async fn create_template(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(template): Json<JwtTemplate>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let created = state.jwt_templates.write().await.create(template)?;
    Ok(Json(json!({ "template": created })))
}

// ── GET /v1/mgmt/jwt/template/all ────────────────────────────────────────────

pub async fn load_all_templates(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let templates: Vec<_> = state
        .jwt_templates
        .read()
        .await
        .load_all()
        .into_iter()
        .cloned()
        .collect();
    Ok(Json(json!({ "templates": templates })))
}

// ── POST /v1/mgmt/jwt/template/update ────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTemplateRequest {
    pub id: String,
    #[serde(flatten)]
    pub template: JwtTemplate,
}

pub async fn update_template(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateTemplateRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state
        .jwt_templates
        .write()
        .await
        .update(&req.id, req.template)?;
    Ok(Json(json!({ "ok": true })))
}

// ── DELETE /v1/mgmt/jwt/template/delete ──────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTemplateRequest {
    pub id: String,
}

pub async fn delete_template(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeleteTemplateRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.jwt_templates.write().await.delete(&req.id)?;
    Ok(Json(json!({ "ok": true })))
}

// ── POST /v1/mgmt/jwt/template/set-active ────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetActiveTemplateRequest {
    pub id: String,
}

pub async fn set_active_template(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<SetActiveTemplateRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.jwt_templates.write().await.set_active(&req.id)?;
    Ok(Json(json!({ "ok": true })))
}

// ── GET /v1/mgmt/jwt/template/active ─────────────────────────────────────────

pub async fn get_active_template(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let active = state.jwt_templates.read().await.active().cloned();
    Ok(Json(json!({ "template": active })))
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::EmulatorConfig,
        state::EmulatorState,
        store::jwt_template_store::{AuthorizationClaimsFormat, JwtTemplate},
    };
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

    fn basic_template(name: &str) -> JwtTemplate {
        JwtTemplate {
            id: String::new(),
            name: name.to_string(),
            authorization_claims_format: AuthorizationClaimsFormat::Flat,
            custom_claims: vec![],
            subject_override: None,
            include_jti: false,
            is_active: false,
        }
    }

    #[tokio::test]
    async fn create_and_list_templates() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create_template(
            State(state.clone()),
            headers.clone(),
            Json(basic_template("t1")),
        )
        .await
        .unwrap();
        let result = load_all_templates(State(state), headers).await.unwrap();
        assert_eq!(result["templates"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn set_active_reflects_in_get_active() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let t = create_template(
            State(state.clone()),
            headers.clone(),
            Json(basic_template("my-tmpl")),
        )
        .await
        .unwrap();
        let id = t["template"]["id"].as_str().unwrap().to_string();
        let _ = set_active_template(
            State(state.clone()),
            headers.clone(),
            Json(SetActiveTemplateRequest { id }),
        )
        .await
        .unwrap();
        let result = get_active_template(State(state), headers).await.unwrap();
        assert_eq!(result["template"]["name"], "my-tmpl");
    }

    #[tokio::test]
    async fn unauthorized_request_rejected() {
        let state = make_state().await;
        let err = load_all_templates(State(state), {
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
