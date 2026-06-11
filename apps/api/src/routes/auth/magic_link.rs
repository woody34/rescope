use crate::extractor::PermissiveJson;
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    auth_policy::AuthPolicyGuard,
    connector::invoker::magic_link_payload,
    cookies::build_auth_cookies,
    error::EmulatorError,
    jwt::token_generator::{generate_refresh_jwt, generate_session_jwt},
    jwt::token_validator::validate_refresh_jwt,
    state::EmulatorState,
    store::token_store::generate_token,
};

// ─── Sign-in initiation ───────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MagicLinkSigninRequest {
    pub login_id: String,
    #[serde(rename = "URI")]
    pub uri: Option<String>,
}

pub async fn signin_email(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<MagicLinkSigninRequest>,
) -> Result<Json<Value>, EmulatorError> {
    use crate::{store::user_store::new_user_id, types::User};
    AuthPolicyGuard::check_method_enabled(&state, "magic_link").await?;

    // Real Descope magic-link sign-in is sign-up-in: auto-provision the user
    // if they don't exist. The identity is confirmed when they click the link.
    let (uid, email) = {
        let users_r = state.users.read().await;
        match users_r.load(&req.login_id) {
            Ok(u) => (u.user_id.clone(), u.email.clone().unwrap_or_default()),
            Err(EmulatorError::UserNotFound) => {
                drop(users_r);
                let new_uid = new_user_id();
                let email = if req.login_id.contains('@') {
                    Some(req.login_id.clone())
                } else {
                    None
                };
                state.users.write().await.insert(User {
                    user_id: new_uid.clone(),
                    login_ids: vec![req.login_id.clone()],
                    email: email.clone(),
                    status: "enabled".into(),
                    ..Default::default()
                })?;
                (new_uid, email.unwrap_or_else(|| req.login_id.clone()))
            }
            Err(e) => return Err(e),
        }
    };

    let masked = masked_email(&email);
    let token = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(token.clone(), uid, crate::types::TokenType::Magic);
    tracing::info!(login_id = %req.login_id, token = %token, "📧 Magic link generated");

    // Fire-and-forget: email connector delivery
    let email_connector_id = state
        .auth_method_config
        .read()
        .await
        .get()
        .magic_link
        .email_connector_id
        .clone();
    if let Some(cid) = email_connector_id {
        if let Ok(c) = state.connectors.read().await.load(&cid).cloned() {
            let inv = state.invoker.clone();
            let payload = magic_link_payload(&req.login_id, &token, req.uri.as_deref());
            tokio::spawn(async move { inv.invoke(&c, payload).await });
        }
    }

    Ok(Json(json!({
        "ok": true,
        "maskedEmail": masked,
        "token": token // Emulator returns token for test convenience
    })))
}

/// POST /v1/auth/magiclink/signup/email — creates user if not found, then initiates magic link.
pub async fn signup_email(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<MagicLinkSigninRequest>,
) -> Result<Json<Value>, EmulatorError> {
    use crate::{store::user_store::new_user_id, types::User};
    AuthPolicyGuard::check_method_enabled(&state, "magic_link").await?;

    // Auto-create the user if they don't exist (signup semantics).
    let uid = {
        let users_r = state.users.read().await;
        match users_r.load(&req.login_id) {
            Ok(u) => u.user_id.clone(),
            Err(_) => {
                drop(users_r);
                let email = if req.login_id.contains('@') {
                    Some(req.login_id.clone())
                } else {
                    None
                };
                let uid = new_user_id();
                let user = User {
                    user_id: uid.clone(),
                    login_ids: vec![req.login_id.clone()],
                    email,
                    status: "enabled".into(),
                    ..Default::default()
                };
                state.users.write().await.insert(user)?;
                uid
            }
        }
    };

    let email_str = state
        .users
        .read()
        .await
        .load(&req.login_id)
        .map(|u| u.email.clone().unwrap_or_default())
        .unwrap_or_default();
    let masked = masked_email(&email_str);

    let token = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(token.clone(), uid, crate::types::TokenType::Magic);
    tracing::info!(login_id = %req.login_id, token = %token, "📧 Magic link generated");

    Ok(Json(json!({
        "ok": true,
        "maskedEmail": masked,
        "token": token
    })))
}

fn masked_email(email: &str) -> String {
    if let Some((user, domain)) = email.split_once('@') {
        let masked = if user.len() <= 2 {
            "*".repeat(user.len())
        } else {
            format!("{}***", &user[..2])
        };
        format!("{masked}@{domain}")
    } else {
        "***".into()
    }
}

// ─── Verify ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub token: String,
}

pub async fn verify(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<VerifyRequest>,
) -> Result<(axum::http::HeaderMap, Json<Value>), EmulatorError> {
    let entry = state.tokens.write().await.consume(&req.token)?;
    let user_id = entry.user_id;

    let users = state.users.read().await;
    let user = users.load_by_user_id(&user_id)?;
    if user.status == "disabled" {
        return Err(EmulatorError::UserDisabled);
    }

    let session_jwt = {
        let tmpl_store = state.jwt_templates.read().await;
        let active_tmpl = tmpl_store.active();
        generate_session_jwt(
            &*state.km().await,
            user,
            &state.config.project_id,
            state.config.session_ttl,
            active_tmpl,
            &*state.roles.read().await,
            "magiclink",
        )
        .map_err(|e: anyhow::Error| EmulatorError::Internal(e.to_string()))?
    };
    let refresh_jwt = generate_refresh_jwt(
        &*state.km().await,
        &user.user_id,
        &state.config.project_id,
        state.config.refresh_ttl,
    )
    .map_err(|e: anyhow::Error| EmulatorError::Internal(e.to_string()))?;

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

// ─── Update email ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEmailRequest {
    pub login_id: String,
    pub email: String,
    pub token: Option<String>,
}

pub async fn update_email(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<UpdateEmailRequest>,
) -> Result<Json<Value>, EmulatorError> {
    if let Some(token) = &req.token {
        let km = &*state.km().await;
        validate_refresh_jwt(km, token)?;
    }

    // Look up the user to update
    let user_id = {
        let users = state.users.read().await;
        let user = users.load(&req.login_id)?;
        user.user_id.clone()
    };

    // Persist the email update and reset verification (new email is unverified)
    state.users.write().await.patch(
        &req.login_id,
        crate::store::user_store::UserPatch {
            email: Some(req.email.clone()),
            verified_email: Some(false),
            ..Default::default()
        },
    )?;

    // Generate a magic link token for the new email (mirrors real Descope behaviour)
    let token = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(token.clone(), user_id, crate::types::TokenType::Magic);
    tracing::info!(login_id = %req.login_id, token = %token, "📧 Magic link generated");

    let masked = masked_email(&req.email);
    Ok(Json(json!({
        "ok": true,
        "maskedEmail": masked,
        "token": token
    })))
}

fn masked_phone(phone: &str) -> String {
    if phone.len() > 4 {
        format!("***{}", &phone[phone.len() - 4..])
    } else {
        "***".into()
    }
}

// ─── Signup SMS ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MagicLinkSmsRequest {
    pub login_id: String,
    #[serde(rename = "URI")]
    pub uri: Option<String>,
}

pub async fn signup_sms(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<MagicLinkSmsRequest>,
) -> Result<Json<Value>, EmulatorError> {
    use crate::{
        store::user_store::new_user_id,
        types::{TokenType, User},
    };
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Fail if user already exists (signup semantics)
    if state.users.read().await.load(&req.login_id).is_ok() {
        return Err(EmulatorError::UserAlreadyExists);
    }

    let uid = new_user_id();
    state.users.write().await.insert(User {
        user_id: uid.clone(),
        login_ids: vec![req.login_id.clone()],
        phone: Some(req.login_id.clone()),
        status: "enabled".into(),
        created_time: now,
        ..Default::default()
    })?;

    let token = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(token.clone(), uid, TokenType::Magic);
    tracing::info!(login_id = %req.login_id, token = %token, "📱 Magic link generated");
    let masked = masked_phone(&req.login_id);
    Ok(Json(
        json!({ "ok": true, "maskedPhone": masked, "token": token }),
    ))
}

// ─── Signin SMS ───────────────────────────────────────────────────────────────

pub async fn signin_sms(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<MagicLinkSmsRequest>,
) -> Result<Json<Value>, EmulatorError> {
    use crate::types::TokenType;
    let users = state.users.read().await;
    let user = users.load(&req.login_id)?;
    let phone = user.phone.clone().unwrap_or_else(|| req.login_id.clone());
    let uid = user.user_id.clone();
    drop(users);

    let token = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(token.clone(), uid, TokenType::Magic);
    tracing::info!(login_id = %req.login_id, token = %token, "📱 Magic link generated");
    let masked = masked_phone(&phone);
    Ok(Json(
        json!({ "ok": true, "maskedPhone": masked, "token": token }),
    ))
}

// ─── Signup-In Email (composite) ─────────────────────────────────────────────

pub async fn signup_in_email(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<MagicLinkSigninRequest>,
) -> Result<Json<Value>, EmulatorError> {
    use crate::{store::user_store::new_user_id, types::User};
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let (uid, email) = {
        let users = state.users.read().await;
        match users.load(&req.login_id) {
            Ok(u) => (
                u.user_id.clone(),
                u.email.clone().unwrap_or_else(|| req.login_id.clone()),
            ),
            Err(EmulatorError::UserNotFound) => {
                drop(users);
                let new_uid = new_user_id();
                state.users.write().await.insert(User {
                    user_id: new_uid.clone(),
                    login_ids: vec![req.login_id.clone()],
                    email: Some(req.login_id.clone()),
                    status: "enabled".into(),
                    created_time: now,
                    ..Default::default()
                })?;
                (new_uid, req.login_id.clone())
            }
            Err(e) => return Err(e),
        }
    };

    let token = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(token.clone(), uid, crate::types::TokenType::Magic);
    tracing::info!(login_id = %req.login_id, token = %token, "📧 Magic link generated");
    let masked = masked_email(&email);
    Ok(Json(
        json!({ "ok": true, "maskedEmail": masked, "token": token }),
    ))
}

// ─── Signup-In SMS (composite) ────────────────────────────────────────────────

pub async fn signup_in_sms(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<MagicLinkSmsRequest>,
) -> Result<Json<Value>, EmulatorError> {
    use crate::{store::user_store::new_user_id, types::User};
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let (uid, phone) = {
        let users = state.users.read().await;
        match users.load(&req.login_id) {
            Ok(u) => (
                u.user_id.clone(),
                u.phone.clone().unwrap_or_else(|| req.login_id.clone()),
            ),
            Err(EmulatorError::UserNotFound) => {
                drop(users);
                let new_uid = new_user_id();
                state.users.write().await.insert(User {
                    user_id: new_uid.clone(),
                    login_ids: vec![req.login_id.clone()],
                    phone: Some(req.login_id.clone()),
                    status: "enabled".into(),
                    created_time: now,
                    ..Default::default()
                })?;
                (new_uid, req.login_id.clone())
            }
            Err(e) => return Err(e),
        }
    };

    let token = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(token.clone(), uid, crate::types::TokenType::Magic);
    tracing::info!(login_id = %req.login_id, token = %token, "📱 Magic link generated");
    let masked = masked_phone(&phone);
    Ok(Json(
        json!({ "ok": true, "maskedPhone": masked, "token": token }),
    ))
}

// ─── Update phone via SMS ─────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePhoneRequest {
    pub login_id: String,
    pub phone: String,
    pub token: Option<String>,
    /// Real SDKs send `addToLoginIDs` FLAT in the POST body — same wire shape
    /// as the OTP update/phone endpoint.
    pub add_to_login_i_ds: Option<bool>,
}

pub async fn update_phone_sms(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<UpdatePhoneRequest>,
) -> Result<Json<Value>, EmulatorError> {
    // Validate JWT if provided
    if let Some(token) = &req.token {
        validate_refresh_jwt(&*state.km().await, token)?;
    }
    // Persist the phone update; when asked, register the phone as a loginId —
    // real Descope does so the user can sign in by the new number afterwards.
    let uid = {
        let mut users = state.users.write().await;
        let user = users.load_mut(&req.login_id)?;
        user.phone = Some(req.phone.clone());
        if req.add_to_login_i_ds.unwrap_or(false) && !user.login_ids.contains(&req.phone) {
            user.login_ids.push(req.phone.clone());
        }
        user.user_id.clone()
    };
    let token = generate_token();
    state
        .tokens
        .write()
        .await
        .insert(token.clone(), uid, crate::types::TokenType::Magic);
    tracing::info!(login_id = %req.login_id, token = %token, "📱 Magic link generated");
    let masked = masked_phone(&req.phone);
    Ok(Json(
        json!({ "ok": true, "maskedPhone": masked, "token": token }),
    ))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::EmulatorConfig, extractor::PermissiveJson, state::EmulatorState,
        store::user_store::new_user_id, types::User,
    };

    async fn make_state() -> EmulatorState {
        EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap()
    }

    async fn insert_user(state: &EmulatorState, login_id: &str, phone: Option<&str>) -> String {
        let uid = new_user_id();
        let mut u = User::default();
        u.user_id = uid.clone();
        u.login_ids = vec![login_id.to_string()];
        u.email = if login_id.contains('@') {
            Some(login_id.to_string())
        } else {
            None
        };
        u.phone = phone.map(|p| p.to_string());
        u.status = "enabled".into();
        state.users.write().await.insert(u).unwrap();
        uid
    }

    // ─── signup_sms ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn signup_sms_new_user_returns_token() {
        let state = make_state().await;
        let result = signup_sms(
            State(state.clone()),
            PermissiveJson(MagicLinkSmsRequest {
                login_id: "+15550010001".into(),
                uri: None,
            }),
        )
        .await
        .unwrap();
        assert!(result["token"].as_str().is_some());
        assert!(result["maskedPhone"].as_str().is_some());
        assert!(state.users.read().await.load("+15550010001").is_ok());
    }

    #[tokio::test]
    async fn signup_sms_duplicate_returns_conflict() {
        let state = make_state().await;
        insert_user(&state, "+15550010002", Some("+15550010002")).await;
        let err = signup_sms(
            State(state),
            PermissiveJson(MagicLinkSmsRequest {
                login_id: "+15550010002".into(),
                uri: None,
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::UserAlreadyExists));
    }

    // ─── signin_sms ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn signin_sms_existing_user_returns_token() {
        let state = make_state().await;
        insert_user(&state, "+15550010003", Some("+15550010003")).await;
        let result = signin_sms(
            State(state),
            PermissiveJson(MagicLinkSmsRequest {
                login_id: "+15550010003".into(),
                uri: None,
            }),
        )
        .await
        .unwrap();
        assert!(result["token"].as_str().is_some());
    }

    #[tokio::test]
    async fn signin_sms_unknown_user_returns_not_found() {
        let state = make_state().await;
        let err = signin_sms(
            State(state),
            PermissiveJson(MagicLinkSmsRequest {
                login_id: "+10000000000".into(),
                uri: None,
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::UserNotFound));
    }

    // ─── signup_in_email ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn signup_in_email_new_user_created_returns_token() {
        let state = make_state().await;
        let result = signup_in_email(
            State(state.clone()),
            PermissiveJson(MagicLinkSigninRequest {
                login_id: "newemail@test.com".into(),
                uri: None,
            }),
        )
        .await
        .unwrap();
        assert!(result["token"].as_str().is_some());
        assert!(state.users.read().await.load("newemail@test.com").is_ok());
    }

    #[tokio::test]
    async fn signup_in_email_existing_user_no_duplicate() {
        let state = make_state().await;
        insert_user(&state, "existing@test.com", None).await;
        let before = state.users.read().await.all_users().len();
        let result = signup_in_email(
            State(state.clone()),
            PermissiveJson(MagicLinkSigninRequest {
                login_id: "existing@test.com".into(),
                uri: None,
            }),
        )
        .await
        .unwrap();
        let after = state.users.read().await.all_users().len();
        assert_eq!(before, after, "no new user should be created");
        assert!(result["token"].as_str().is_some());
    }

    // ─── signup_in_sms ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn signup_in_sms_new_user_created_returns_token() {
        let state = make_state().await;
        let result = signup_in_sms(
            State(state.clone()),
            PermissiveJson(MagicLinkSmsRequest {
                login_id: "+15550010004".into(),
                uri: None,
            }),
        )
        .await
        .unwrap();
        assert!(result["token"].as_str().is_some());
        assert!(state.users.read().await.load("+15550010004").is_ok());
    }

    #[tokio::test]
    async fn signup_in_sms_existing_user_no_duplicate() {
        let state = make_state().await;
        insert_user(&state, "+15550010005", Some("+15550010005")).await;
        let before = state.users.read().await.all_users().len();
        let _ = signup_in_sms(
            State(state.clone()),
            PermissiveJson(MagicLinkSmsRequest {
                login_id: "+15550010005".into(),
                uri: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(before, state.users.read().await.all_users().len());
    }

    // ─── update_phone_sms ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn update_phone_sms_persists_phone_and_returns_token() {
        let state = make_state().await;
        insert_user(&state, "user@test.com", None).await;
        let result = update_phone_sms(
            State(state.clone()),
            PermissiveJson(UpdatePhoneRequest {
                login_id: "user@test.com".into(),
                phone: "+15550020001".into(),
                token: None,
                add_to_login_i_ds: None,
            }),
        )
        .await
        .unwrap();
        assert!(result["token"].as_str().is_some());
        let u = state.users.read().await;
        assert_eq!(
            u.load("user@test.com").unwrap().phone.as_deref(),
            Some("+15550020001")
        );
    }

    // Why: real SDKs send `addToLoginIDs` flat in the POST body, and real
    //      Descope then registers the phone as a loginId.
    #[tokio::test]
    async fn update_phone_sms_flat_add_to_login_ids_registers_login_id() {
        let state = make_state().await;
        insert_user(&state, "lid@test.com", None).await;

        let req: UpdatePhoneRequest = serde_json::from_value(json!({
            "loginId": "lid@test.com",
            "phone": "+15550020002",
            "addToLoginIDs": true,
        }))
        .unwrap();
        let _ = update_phone_sms(State(state.clone()), PermissiveJson(req))
            .await
            .unwrap();

        let users = state.users.read().await;
        let u = users.load("lid@test.com").unwrap();
        assert_eq!(u.phone.as_deref(), Some("+15550020002"));
        assert!(u.login_ids.contains(&"+15550020002".to_string()));
    }

    #[tokio::test]
    async fn update_phone_sms_unknown_user_returns_not_found() {
        let state = make_state().await;
        let err = update_phone_sms(
            State(state),
            PermissiveJson(UpdatePhoneRequest {
                login_id: "ghost@test.com".into(),
                phone: "+15550099999".into(),
                token: None,
                add_to_login_i_ds: None,
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::UserNotFound));
    }
}
