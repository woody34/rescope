use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    error::EmulatorError,
    extractor::PermissiveJson,
    state::EmulatorState,
    types::{AuthType, OidcConfig, SamlConfig},
};

pub async fn load_all(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let tenants: Vec<_> = state
        .tenants
        .read()
        .await
        .load_all()
        .into_iter()
        .cloned()
        .collect();
    Ok(Json(json!({ "tenants": tenants })))
}

// ── Tenant Create ────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateTenantRequest {
    pub id: Option<String>,
    pub name: String,
    pub self_provisioning_domains: Option<Vec<String>>,
    pub auth_type: Option<AuthType>,
    pub saml_config: Option<SamlConfig>,
    pub oidc_config: Option<OidcConfig>,
    pub enforce_sso: Option<bool>,
    pub parent_tenant_id: Option<String>,
    pub session_token_ttl_override: Option<u64>,
    pub refresh_token_ttl_override: Option<u64>,
}

pub async fn create(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<CreateTenantRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let id = state.tenants.write().await.create(
        req.id,
        req.name,
        req.self_provisioning_domains.unwrap_or_default(),
    )?;
    // Apply optional SSO fields if provided
    if req.auth_type.is_some()
        || req.saml_config.is_some()
        || req.oidc_config.is_some()
        || req.enforce_sso.is_some()
        || req.parent_tenant_id.is_some()
        || req.session_token_ttl_override.is_some()
        || req.refresh_token_ttl_override.is_some()
    {
        state.tenants.write().await.update(
            &id,
            None,
            None,
            req.auth_type,
            req.saml_config,
            req.oidc_config,
            req.enforce_sso,
            req.parent_tenant_id,
            req.session_token_ttl_override,
            req.refresh_token_ttl_override,
        )?;
    }
    let t = state.tenants.read().await.load(&id)?.clone();
    Ok(Json(json!({ "tenant": t })))
}

// ── Tenant Update ────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTenantRequest {
    pub id: String,
    pub name: Option<String>,
    pub self_provisioning_domains: Option<Vec<String>>,
    pub auth_type: Option<AuthType>,
    pub saml_config: Option<SamlConfig>,
    pub oidc_config: Option<OidcConfig>,
    pub enforce_sso: Option<bool>,
    pub parent_tenant_id: Option<String>,
    pub session_token_ttl_override: Option<u64>,
    pub refresh_token_ttl_override: Option<u64>,
}

pub async fn update(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<UpdateTenantRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.tenants.write().await.update(
        &req.id,
        req.name,
        req.self_provisioning_domains,
        req.auth_type,
        req.saml_config,
        req.oidc_config,
        req.enforce_sso,
        req.parent_tenant_id,
        req.session_token_ttl_override,
        req.refresh_token_ttl_override,
    )?;
    let t = state.tenants.read().await.load(&req.id)?.clone();
    Ok(Json(json!({ "tenant": t })))
}

// ── Tenant Load ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TenantIdQuery {
    #[serde(rename = "id")]
    pub id: Option<String>,
}

pub async fn load(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Query(q): Query<TenantIdQuery>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let id = q.id.ok_or(EmulatorError::TenantNotFound)?;
    let t = state.tenants.read().await.load(&id)?.clone();
    Ok(Json(json!({ "tenant": t })))
}

pub async fn delete_tenant(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Query(q): Query<TenantIdQuery>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    if let Some(id) = q.id {
        state.tenants.write().await.delete_tenant(&id);
    }
    Ok(Json(json!({ "ok": true })))
}

// POST /v1/mgmt/tenant/delete — Node SDK variant (body: {id, cascade})
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTenantPostRequest {
    pub id: Option<String>,
    pub cascade: Option<bool>,
}

pub async fn delete_tenant_post(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<DeleteTenantPostRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    if let Some(ref id) = req.id {
        state.tenants.write().await.delete_tenant(id);
    }
    Ok(Json(json!({ "ok": true })))
}

// ── Tenant Search ────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TenantSearchRequest {
    pub tenant_ids: Option<Vec<String>>,
    pub tenant_names: Option<Vec<String>>,
}

pub async fn search(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<TenantSearchRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let store = state.tenants.read().await;
    let results: Vec<_> = store
        .search(req.tenant_ids.as_deref(), req.tenant_names.as_deref())
        .into_iter()
        .cloned()
        .collect();
    Ok(Json(json!({ "tenants": results })))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::EmulatorConfig, state::EmulatorState};
    use axum::{
        extract::{Query, State},
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

    // ─── create ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_tenant_returns_tenant_with_id() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let result = create(
            State(state.clone()),
            headers,
            axum::extract::Json(CreateTenantRequest {
                id: Some("t-001".into()),
                name: "Test Corp".into(),
                self_provisioning_domains: None,
                auth_type: None,
                saml_config: None,
                oidc_config: None,
                enforce_sso: None,
                parent_tenant_id: None,
                session_token_ttl_override: None,
                refresh_token_ttl_override: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(result["tenant"]["id"], "t-001");
        assert_eq!(result["tenant"]["name"], "Test Corp");
    }

    #[tokio::test]
    async fn create_tenant_duplicate_returns_conflict() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create(
            State(state.clone()),
            headers.clone(),
            axum::extract::Json(CreateTenantRequest {
                id: Some("t-dup".into()),
                name: "Corp".into(),
                self_provisioning_domains: None,
                auth_type: None,
                saml_config: None,
                oidc_config: None,
                enforce_sso: None,
                parent_tenant_id: None,
                session_token_ttl_override: None,
                refresh_token_ttl_override: None,
            }),
        )
        .await
        .unwrap();
        let err = create(
            State(state),
            headers,
            axum::extract::Json(CreateTenantRequest {
                id: Some("t-dup".into()),
                name: "Corp2".into(),
                self_provisioning_domains: None,
                auth_type: None,
                saml_config: None,
                oidc_config: None,
                enforce_sso: None,
                parent_tenant_id: None,
                session_token_ttl_override: None,
                refresh_token_ttl_override: None,
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::TenantAlreadyExists));
    }

    // ─── update ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn update_tenant_name() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create(
            State(state.clone()),
            headers.clone(),
            axum::extract::Json(CreateTenantRequest {
                id: Some("upd-t".into()),
                name: "Old Name".into(),
                self_provisioning_domains: None,
                auth_type: None,
                saml_config: None,
                oidc_config: None,
                enforce_sso: None,
                parent_tenant_id: None,
                session_token_ttl_override: None,
                refresh_token_ttl_override: None,
            }),
        )
        .await
        .unwrap();
        let result = update(
            State(state),
            headers,
            axum::extract::Json(UpdateTenantRequest {
                id: "upd-t".into(),
                name: Some("New Name".into()),
                self_provisioning_domains: None,
                auth_type: None,
                saml_config: None,
                oidc_config: None,
                enforce_sso: None,
                parent_tenant_id: None,
                session_token_ttl_override: None,
                refresh_token_ttl_override: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(result["tenant"]["name"], "New Name");
    }

    #[tokio::test]
    async fn update_unknown_tenant_returns_not_found() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let err = update(
            State(state),
            headers,
            axum::extract::Json(UpdateTenantRequest {
                id: "ghost".into(),
                name: Some("X".into()),
                self_provisioning_domains: None,
                auth_type: None,
                saml_config: None,
                oidc_config: None,
                enforce_sso: None,
                parent_tenant_id: None,
                session_token_ttl_override: None,
                refresh_token_ttl_override: None,
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::TenantNotFound));
    }

    // ─── load ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn load_tenant_returns_correct_tenant() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create(
            State(state.clone()),
            headers.clone(),
            axum::extract::Json(CreateTenantRequest {
                id: Some("load-t".into()),
                name: "Load Me".into(),
                self_provisioning_domains: None,
                auth_type: None,
                saml_config: None,
                oidc_config: None,
                enforce_sso: None,
                parent_tenant_id: None,
                session_token_ttl_override: None,
                refresh_token_ttl_override: None,
            }),
        )
        .await
        .unwrap();
        let result = load(
            State(state),
            headers,
            Query(TenantIdQuery {
                id: Some("load-t".into()),
            }),
        )
        .await
        .unwrap();
        assert_eq!(result["tenant"]["name"], "Load Me");
    }

    #[tokio::test]
    async fn load_unknown_tenant_returns_not_found() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let err = load(
            State(state),
            headers,
            Query(TenantIdQuery {
                id: Some("no-such".into()),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::TenantNotFound));
    }

    // ─── delete ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn delete_tenant_then_load_returns_not_found() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create(
            State(state.clone()),
            headers.clone(),
            axum::extract::Json(CreateTenantRequest {
                id: Some("del-t".into()),
                name: "Delete Me".into(),
                self_provisioning_domains: None,
                auth_type: None,
                saml_config: None,
                oidc_config: None,
                enforce_sso: None,
                parent_tenant_id: None,
                session_token_ttl_override: None,
                refresh_token_ttl_override: None,
            }),
        )
        .await
        .unwrap();
        let _ = delete_tenant(
            State(state.clone()),
            headers.clone(),
            Query(TenantIdQuery {
                id: Some("del-t".into()),
            }),
        )
        .await
        .unwrap();
        let err = load(
            State(state),
            headers,
            Query(TenantIdQuery {
                id: Some("del-t".into()),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::TenantNotFound));
    }

    // ─── search ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn search_by_name_returns_matching_tenants() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create(
            State(state.clone()),
            headers.clone(),
            axum::extract::Json(CreateTenantRequest {
                id: Some("srch-1".into()),
                name: "Acme Corp".into(),
                self_provisioning_domains: None,
                auth_type: None,
                saml_config: None,
                oidc_config: None,
                enforce_sso: None,
                parent_tenant_id: None,
                session_token_ttl_override: None,
                refresh_token_ttl_override: None,
            }),
        )
        .await
        .unwrap();
        let _ = create(
            State(state.clone()),
            headers.clone(),
            axum::extract::Json(CreateTenantRequest {
                id: Some("srch-2".into()),
                name: "Other Co".into(),
                self_provisioning_domains: None,
                auth_type: None,
                saml_config: None,
                oidc_config: None,
                enforce_sso: None,
                parent_tenant_id: None,
                session_token_ttl_override: None,
                refresh_token_ttl_override: None,
            }),
        )
        .await
        .unwrap();
        let result = search(
            State(state),
            headers,
            axum::extract::Json(TenantSearchRequest {
                tenant_ids: None,
                tenant_names: Some(vec!["Acme Corp".into()]),
            }),
        )
        .await
        .unwrap();
        let tenants = result["tenants"].as_array().unwrap();
        assert_eq!(tenants.len(), 1);
        assert_eq!(tenants[0]["name"], "Acme Corp");
    }

    #[tokio::test]
    async fn search_empty_filter_returns_all() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let _ = create(
            State(state.clone()),
            headers.clone(),
            axum::extract::Json(CreateTenantRequest {
                id: Some("all-1".into()),
                name: "Alpha".into(),
                self_provisioning_domains: None,
                auth_type: None,
                saml_config: None,
                oidc_config: None,
                enforce_sso: None,
                parent_tenant_id: None,
                session_token_ttl_override: None,
                refresh_token_ttl_override: None,
            }),
        )
        .await
        .unwrap();
        let _ = create(
            State(state.clone()),
            headers.clone(),
            axum::extract::Json(CreateTenantRequest {
                id: Some("all-2".into()),
                name: "Beta".into(),
                self_provisioning_domains: None,
                auth_type: None,
                saml_config: None,
                oidc_config: None,
                enforce_sso: None,
                parent_tenant_id: None,
                session_token_ttl_override: None,
                refresh_token_ttl_override: None,
            }),
        )
        .await
        .unwrap();
        let result = search(
            State(state),
            headers,
            axum::extract::Json(TenantSearchRequest {
                tenant_ids: None,
                tenant_names: None,
            }),
        )
        .await
        .unwrap();
        let count = result["tenants"].as_array().unwrap().len();
        assert!(count >= 2);
    }
}
