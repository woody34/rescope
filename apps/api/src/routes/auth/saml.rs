use crate::extractor::PermissiveJson;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    cookies::build_auth_cookies,
    error::EmulatorError,
    jwt::token_generator::{generate_refresh_jwt, generate_session_jwt},
    state::EmulatorState,
    store::token_store::generate_token,
    types::{AuthType, TokenType},
};

// ─── SAML Start ───────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SamlStartRequest {
    pub tenant: String,
    #[serde(alias = "redirectURL")]
    pub redirect_url: Option<String>,
    #[serde(alias = "loginHint")]
    pub login_hint: Option<String>,
}

/// All-optional version for query params and empty-body fallback.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SamlStartQueryParams {
    #[serde(default)]
    pub tenant: String,
    #[serde(alias = "redirectURL")]
    pub redirect_url: Option<String>,
    #[serde(alias = "loginHint")]
    pub login_hint: Option<String>,
}

/// POST handler — reads query params first, then merges/falls back to JSON body.
pub async fn start(
    State(state): State<EmulatorState>,
    Query(query): Query<SamlStartQueryParams>,
    PermissiveJson(body): PermissiveJson<SamlStartQueryParams>,
) -> Result<Json<Value>, EmulatorError> {
    let req = SamlStartRequest {
        tenant: if query.tenant.is_empty() {
            body.tenant
        } else {
            query.tenant
        },
        redirect_url: query.redirect_url.or(body.redirect_url),
        login_hint: query.login_hint.or(body.login_hint),
    };
    start_impl(state, req).await
}

/// GET handler — reads from query parameters.
pub async fn start_get(
    State(state): State<EmulatorState>,
    Query(req): Query<SamlStartRequest>,
) -> Result<Json<Value>, EmulatorError> {
    start_impl(state, req).await
}

async fn start_impl(
    state: EmulatorState,
    req: SamlStartRequest,
) -> Result<Json<Value>, EmulatorError> {
    // Dual resolution: email → find user → find tenant; else tenant ID
    let tenant_id = if req.tenant.contains('@') {
        // Lookup user's SAML tenant by email domain
        let tenants = state.tenants.read().await;
        let tenant = tenants.find_by_email(&req.tenant)?;
        // Also verify user exists
        let users = state.users.read().await;
        users.load(&req.tenant)?;
        tenant.id.clone()
    } else {
        // Tenant ID direct lookup
        let tenants = state.tenants.read().await;
        let tenant = tenants.load(&req.tenant)?;
        if tenant.auth_type != AuthType::Saml && tenant.auth_type != AuthType::Oidc {
            return Err(EmulatorError::NotSsoUser);
        }
        tenant.id.clone()
    };

    // Find first user in this tenant or use the email as user lookup
    let user_id = if req.tenant.contains('@') {
        state.users.read().await.load(&req.tenant)?.user_id.clone()
    } else {
        // Without an email we cannot resolve to a specific user — return a tenant-level code
        format!("tenant:{}", tenant_id)
    };

    let code = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(code.clone(), user_id, TokenType::Saml);

    let redirect_url = req.redirect_url.unwrap_or_default();
    let url = format!("{redirect_url}?code={code}");

    Ok(Json(json!({ "url": url })))
}

// ─── SAML Exchange ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SamlExchangeRequest {
    pub code: String,
}

pub async fn exchange(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<SamlExchangeRequest>,
) -> Result<(axum::http::HeaderMap, Json<Value>), EmulatorError> {
    let entry = state.tokens.write().await.consume(&req.code)?;
    let user_id = entry.user_id;

    // user_id may be "tenant:<id>" if saml.start was called with a tenant ID (no user resolution)
    if user_id.starts_with("tenant:") {
        return Err(EmulatorError::UserNotFound);
    }

    let users = state.users.read().await;
    let user = users.load_by_user_id(&user_id)?;
    if user.status == "disabled" {
        return Err(EmulatorError::UserDisabled);
    }

    let tmpl_store = state.jwt_templates.read().await;
    let active_tmpl = tmpl_store.active();
    let session_jwt = generate_session_jwt(
        &*state.km().await,
        user,
        &state.config.project_id,
        state.config.session_ttl,
        active_tmpl,
        &*state.roles.read().await,
        "saml",
    )
    .map_err(|e| EmulatorError::Internal(e.to_string()))?;
    let refresh_jwt = generate_refresh_jwt(
        &*state.km().await,
        &user.user_id,
        &state.config.project_id,
        state.config.refresh_ttl,
    )
    .map_err(|e| EmulatorError::Internal(e.to_string()))?;

    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + state.config.session_ttl;
    let user_resp = serde_json::to_value(user.to_response()).unwrap();
    let cookies = build_auth_cookies(&session_jwt, &refresh_jwt, state.config.session_ttl);
    let body = json!({
        "sessionJwt": session_jwt,
        "refreshJwt": refresh_jwt,
        "cookieDomain": "",
        "cookiePath": "/",
        "cookieMaxAge": state.config.session_ttl,
        "cookieExpiration": exp,
        "firstSeen": false,
        "user": user_resp
    });
    drop(users);

    // Record login timestamp
    let _ = state.users.write().await.record_login_by_user_id(&user_id);

    Ok((cookies, Json(body)))
}
