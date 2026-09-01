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
    routes::emulator::sso_provision::ResumeParams,
    sso_jit::{normalize_email, provision_sso_user, SsoProvisionInput},
    state::EmulatorState,
    store::token_store::generate_token,
    types::{AuthType, Tenant, TokenType},
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
    /// Override the tenant's `ssoJitProvisioning.prompt` for this sign-in.
    /// Set to `false` for an automated sign-in that wants the provisioned
    /// defaults and a code straight away, with no configuration screen.
    pub sso_jit_prompt: Option<bool>,
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
    pub sso_jit_prompt: Option<bool>,
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
        sso_jit_prompt: query.sso_jit_prompt.or(body.sso_jit_prompt),
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
    let redirect_url = req.redirect_url.clone().unwrap_or_default();

    // Dual resolution: an email resolves its tenant by domain; anything else is
    // a tenant ID.
    let tenant: Tenant = if req.tenant.contains('@') {
        state
            .tenants
            .read()
            .await
            .find_by_email(&req.tenant)?
            .clone()
    } else {
        let tenant = state.tenants.read().await.load(&req.tenant)?.clone();
        if tenant.auth_type != AuthType::Saml && tenant.auth_type != AuthType::Oidc {
            return Err(EmulatorError::NotSsoUser);
        }
        tenant
    };

    // Started with a tenant ID and no email, so there is nobody to sign in yet.
    // Real Descope hands off to the IdP, which asks who is signing in; the
    // configuration screen is where the emulator asks the same question.
    if !req.tenant.contains('@') {
        if !tenant.sso_jit().enabled {
            // Pre-existing behavior: a tenant-level code that exchange rejects.
            let code = generate_token();
            state.tokens.write().await.insert(
                code.clone(),
                format!("tenant:{}", tenant.id),
                TokenType::Saml,
            );
            return Ok(Json(
                json!({ "url": format!("{redirect_url}?code={code}") }),
            ));
        }
        let params = ResumeParams {
            email: None,
            tenant: Some(tenant.id.clone()),
            mode: Some("code".into()),
            redirect_url: Some(redirect_url),
            continue_url: None,
        };
        return Ok(Json(json!({ "url": params.to_url(state.config.port) })));
    }

    let user_id = match state.users.read().await.load(&req.tenant) {
        Ok(user) => Some(user.user_id.clone()),
        Err(_) => None,
    };

    let user_id = match user_id {
        Some(user_id) => user_id,
        None => {
            let jit = tenant.sso_jit();
            if !jit.enabled {
                return Err(EmulatorError::UserNotFound);
            }
            let email = normalize_email(&req.tenant).ok_or(EmulatorError::UserNotFound)?;

            // Ask how the new user should be configured unless this sign-in
            // opted out, or the tenant did.
            if req.sso_jit_prompt.unwrap_or(jit.prompt) {
                let params = ResumeParams {
                    email: Some(email),
                    tenant: Some(tenant.id.clone()),
                    mode: Some("code".into()),
                    redirect_url: Some(redirect_url),
                    continue_url: None,
                };
                return Ok(Json(json!({ "url": params.to_url(state.config.port) })));
            }

            provision_sso_user(&state, &tenant, SsoProvisionInput::new(email))
                .await?
                .user_id
        }
    };

    let code = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(code.clone(), user_id, TokenType::Saml);

    Ok(Json(
        json!({ "url": format!("{redirect_url}?code={code}") }),
    ))
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

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::EmulatorConfig,
        extractor::PermissiveJson,
        store::user_store::new_user_id,
        types::{SsoJitProvisioning, User},
    };

    async fn make_state() -> EmulatorState {
        let state = EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap();
        state.tenants.write().await.insert(Tenant {
            id: "acme".into(),
            name: "Acme".into(),
            domains: vec!["acme.test".into()],
            auth_type: AuthType::Saml,
            ..Default::default()
        });
        state
    }

    fn start_request(tenant: &str, sso_jit_prompt: Option<bool>) -> SamlStartRequest {
        SamlStartRequest {
            tenant: tenant.into(),
            redirect_url: Some("http://localhost:4200/login/sso".into()),
            login_hint: None,
            sso_jit_prompt,
        }
    }

    async fn seed_user(state: &EmulatorState, login_id: &str) {
        let user = User {
            user_id: new_user_id(),
            login_ids: vec![login_id.into()],
            email: Some(login_id.into()),
            status: "enabled".into(),
            ..Default::default()
        };
        state.users.write().await.insert(user).unwrap();
    }

    /// Why: an address the emulator has never seen is the whole point — before
    ///      this it was rejected outright, so only pre-seeded users could use
    ///      SSO. In a browser the sign-in should continue, not fail.
    /// Decision: hand back the configuration screen's URL, which the SDK
    ///      navigates to exactly like a real IdP's.
    #[tokio::test]
    async fn unknown_email_on_an_sso_domain_starts_the_configuration_screen() {
        let state = make_state().await;

        let resp = start_impl(state, start_request("Newcomer@acme.test", None))
            .await
            .unwrap();

        let url = resp.0["url"].as_str().unwrap();
        assert!(url.contains("/emulator/sso/provision"));
        assert!(url.contains("email=newcomer%40acme.test"));
        assert!(url.contains("tenant=acme"));
        assert!(url.contains("redirectUrl=http%3A%2F%2Flocalhost%3A4200%2Flogin%2Fsso"));
    }

    /// Why: automated sign-ins cannot fill in a form, and must still get a
    ///      usable session for a brand-new address.
    /// Decision: `ssoJitPrompt: false` provisions with the defaults and returns
    ///      the same `?code=` redirect an existing user would get — and that
    ///      code exchanges into a session for the newly created user.
    #[tokio::test]
    async fn prompt_disabled_provisions_and_returns_an_exchangeable_code() {
        let state = make_state().await;

        let resp = start_impl(state.clone(), start_request("auto@acme.test", Some(false)))
            .await
            .unwrap();
        let url = resp.0["url"].as_str().unwrap().to_string();
        assert!(url.starts_with("http://localhost:4200/login/sso?code="));

        let code = url.split("code=").nth(1).unwrap().to_string();
        let (_, body) = exchange(
            axum::extract::State(state.clone()),
            PermissiveJson(SamlExchangeRequest { code }),
        )
        .await
        .unwrap();

        assert!(!body.0["sessionJwt"].as_str().unwrap().is_empty());
        assert_eq!(body.0["user"]["email"], "auto@acme.test");
        assert_eq!(body.0["user"]["status"], "enabled");
        assert_eq!(body.0["user"]["userTenants"][0]["tenantId"], "acme");
    }

    /// Why: a tenant must be able to keep the strict behavior, so suites that
    ///      assert on a rejected unknown user do not silently start passing.
    /// Decision: `ssoJitProvisioning.enabled: false` restores "user not found".
    #[tokio::test]
    async fn provisioning_disabled_still_rejects_an_unknown_user() {
        let state = make_state().await;
        let mut tenant = state.tenants.read().await.load("acme").unwrap().clone();
        tenant.sso_jit_provisioning = Some(SsoJitProvisioning {
            enabled: false,
            ..Default::default()
        });
        state.tenants.write().await.insert(tenant);

        let err = start_impl(state, start_request("nobody@acme.test", None))
            .await
            .unwrap_err();
        assert!(matches!(err, EmulatorError::UserNotFound));
    }

    /// Why: provisioning is scoped to SSO tenants — an address on a domain no
    ///      tenant claims is not an SSO login at all.
    /// Decision: keep returning "tenant not found" rather than creating a user.
    #[tokio::test]
    async fn an_email_on_an_unclaimed_domain_is_still_rejected() {
        let state = make_state().await;
        let err = start_impl(state, start_request("someone@elsewhere.test", None))
            .await
            .unwrap_err();
        assert!(matches!(err, EmulatorError::TenantNotFound));
    }

    /// Why: existing users must keep the direct redirect — an interstitial for
    ///      them would break every sign-in that already works.
    /// Decision: only an unknown login ID reaches the configuration screen.
    #[tokio::test]
    async fn a_known_user_still_gets_a_direct_code_redirect() {
        let state = make_state().await;
        seed_user(&state, "known@acme.test").await;

        let resp = start_impl(state, start_request("known@acme.test", None))
            .await
            .unwrap();

        let url = resp.0["url"].as_str().unwrap();
        assert!(url.starts_with("http://localhost:4200/login/sso?code="));
        assert!(!url.contains("/emulator/sso/provision"));
    }

    /// Why: starting from a tenant ID names no user, so the emulator used to
    ///      mint a `tenant:<id>` code that exchange always rejected — a dead end.
    /// Decision: send the browser to the configuration screen, which asks who is
    ///      signing in and then resumes with a real code.
    #[tokio::test]
    async fn a_tenant_id_start_opens_the_configuration_screen() {
        let state = make_state().await;

        let resp = start_impl(state, start_request("acme", None))
            .await
            .unwrap();

        let url = resp.0["url"].as_str().unwrap();
        assert!(url.contains("/emulator/sso/provision"));
        assert!(url.contains("tenant=acme"));
        assert!(!url.contains("email="));
    }
}
