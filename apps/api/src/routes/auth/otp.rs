use crate::extractor::PermissiveJson;
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    auth_policy::AuthPolicyGuard,
    connector::invoker::otp_payload,
    cookies::build_auth_cookies,
    error::EmulatorError,
    jwt::token_generator::{generate_refresh_jwt, generate_session_jwt},
    state::EmulatorState,
    store::{otp_store::generate_otp_code, user_store::new_user_id},
    types::User,
};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
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

fn masked_phone(phone: &str) -> String {
    if phone.len() > 4 {
        format!("***{}", &phone[phone.len() - 4..])
    } else {
        "***".into()
    }
}

fn build_auth_response(
    state: &EmulatorState,
    session_jwt: &str,
    refresh_jwt: &str,
    user_resp: Value,
) -> Value {
    let exp = now() + state.config.session_ttl;
    json!({
        "sessionJwt": session_jwt,
        "refreshJwt": refresh_jwt,
        "cookieDomain": "",
        "cookiePath": "/",
        "cookieMaxAge": state.config.session_ttl,
        "cookieExpiration": exp,
        "firstSeen": false,
        "user": user_resp
    })
}

// ─── OTP Signup (email) ───────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpSignupEmailRequest {
    pub login_id: String,
}

pub async fn signup_email(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<OtpSignupEmailRequest>,
) -> Result<Json<Value>, EmulatorError> {
    AuthPolicyGuard::check_method_enabled(&state, "otp").await?;
    let user_id = new_user_id();
    let user = User {
        user_id: user_id.clone(),
        login_ids: vec![req.login_id.clone()],
        email: Some(req.login_id.clone()),
        verified_email: false,
        status: "enabled".into(),
        created_time: now(),
        ..Default::default()
    };

    state
        .users
        .write()
        .await
        .insert(user)
        .map_err(|_| EmulatorError::UserAlreadyExists)?;

    let code = generate_otp_code();
    state.otps.write().await.store(&user_id, code.clone());
    tracing::info!(login_id = %req.login_id, code = %code, "📧 OTP generated");

    let masked = masked_email(&req.login_id);

    // Fire-and-forget: notify the configured email connector (if any)
    let email_connector_id = state
        .auth_method_config
        .read()
        .await
        .get()
        .otp
        .email_connector_id
        .clone();
    if let Some(cid) = email_connector_id {
        if let Ok(c) = state.connectors.read().await.load(&cid).cloned() {
            let inv = state.invoker.clone();
            let payload = otp_payload(&req.login_id, &code, "email");
            tokio::spawn(async move { inv.invoke(&c, payload).await });
        }
    }

    Ok(Json(json!({ "maskedEmail": masked, "code": code })))
}

// ─── OTP Signup (phone/SMS) ───────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpSignupPhoneRequest {
    pub login_id: String,
}

pub async fn signup_phone_sms(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<OtpSignupPhoneRequest>,
) -> Result<Json<Value>, EmulatorError> {
    AuthPolicyGuard::check_method_enabled(&state, "otp").await?;
    let user_id = new_user_id();
    let user = User {
        user_id: user_id.clone(),
        login_ids: vec![req.login_id.clone()],
        phone: Some(req.login_id.clone()),
        verified_phone: false,
        status: "enabled".into(),
        created_time: now(),
        ..Default::default()
    };

    state
        .users
        .write()
        .await
        .insert(user)
        .map_err(|_| EmulatorError::UserAlreadyExists)?;

    let code = generate_otp_code();
    state.otps.write().await.store(&user_id, code.clone());
    tracing::info!(login_id = %req.login_id, code = %code, "\u{1f4f1} OTP generated");

    let masked = masked_phone(&req.login_id);

    // Fire-and-forget: notify the configured SMS connector (if any)
    let sms_connector_id = state
        .auth_method_config
        .read()
        .await
        .get()
        .otp
        .sms_connector_id
        .clone();
    if let Some(cid) = sms_connector_id {
        if let Ok(c) = state.connectors.read().await.load(&cid).cloned() {
            let inv = state.invoker.clone();
            let payload = otp_payload(&req.login_id, &code, "sms");
            tokio::spawn(async move { inv.invoke(&c, payload).await });
        }
    }

    Ok(Json(json!({ "maskedPhone": masked, "code": code })))
}

// ─── OTP Signin (email) ───────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpSigninEmailRequest {
    pub login_id: String,
}

pub async fn signin_email(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<OtpSigninEmailRequest>,
) -> Result<Json<Value>, EmulatorError> {
    AuthPolicyGuard::check_method_enabled(&state, "otp").await?;

    // Real Descope OTP sign-in is sign-up-in: auto-provision the user if they
    // don't exist yet. This matches the "magic" of Descope — users don't need
    // to separately sign up before signing in with OTP.
    let (user_id, email) = {
        let users_r = state.users.read().await;
        match users_r.load(&req.login_id) {
            Ok(u) => (
                u.user_id.clone(),
                u.email.clone().unwrap_or_else(|| req.login_id.clone()),
            ),
            Err(EmulatorError::UserNotFound) => {
                drop(users_r);
                let uid = new_user_id();
                let user = User {
                    user_id: uid.clone(),
                    login_ids: vec![req.login_id.clone()],
                    email: Some(req.login_id.clone()),
                    verified_email: false,
                    status: "enabled".into(),
                    created_time: now(),
                    ..Default::default()
                };
                state.users.write().await.insert(user)?;
                (uid, req.login_id.clone())
            }
            Err(e) => return Err(e),
        }
    };

    let code = generate_otp_code();
    state.otps.write().await.store(&user_id, code.clone());
    tracing::info!(login_id = %req.login_id, code = %code, "📧 OTP generated");

    let masked = masked_email(&email);
    Ok(Json(json!({ "maskedEmail": masked, "code": code })))
}

// ─── OTP Signin (phone/SMS) ───────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpSigninPhoneRequest {
    pub login_id: String,
}

pub async fn signin_phone_sms(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<OtpSigninPhoneRequest>,
) -> Result<Json<Value>, EmulatorError> {
    AuthPolicyGuard::check_method_enabled(&state, "otp").await?;
    let users = state.users.read().await;
    let user = users.load(&req.login_id)?;
    let user_id = user.user_id.clone();
    let phone = user.phone.clone().unwrap_or_else(|| req.login_id.clone());
    drop(users);

    let code = generate_otp_code();
    state.otps.write().await.store(&user_id, code.clone());
    tracing::info!(login_id = %req.login_id, code = %code, "📱 OTP generated");

    let masked = masked_phone(&phone);
    Ok(Json(json!({ "maskedPhone": masked, "code": code })))
}

// ─── OTP Verify (email) ───────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpVerifyEmailRequest {
    pub login_id: String,
    pub code: String,
}

pub async fn verify_email(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<OtpVerifyEmailRequest>,
) -> Result<(axum::http::HeaderMap, Json<Value>), EmulatorError> {
    // 1. Load user to get userId
    let user_id = {
        let users = state.users.read().await;
        let user = users.load(&req.login_id)?;
        if user.status == "disabled" {
            return Err(EmulatorError::UserDisabled);
        }
        user.user_id.clone()
    };

    // 2. Consume OTP code
    state.otps.write().await.consume(&user_id, &req.code)?;

    // 3. Set verified_email = true
    state.users.write().await.patch(
        &req.login_id,
        crate::store::user_store::UserPatch {
            verified_email: Some(true),
            ..Default::default()
        },
    )?;

    // 4. Issue tokens
    let users = state.users.read().await;
    let user = users.load(&req.login_id)?;
    let tmpl_store = state.jwt_templates.read().await;
    let active_tmpl = tmpl_store.active();
    let session_jwt = generate_session_jwt(
        &*state.km().await,
        user,
        &state.config.project_id,
        state.config.session_ttl,
        active_tmpl,
        &*state.roles.read().await,
        "email",
    )
    .map_err(|e| EmulatorError::Internal(e.to_string()))?;
    let refresh_jwt = generate_refresh_jwt(
        &*state.km().await,
        &user.user_id,
        &state.config.project_id,
        state.config.refresh_ttl,
    )
    .map_err(|e| EmulatorError::Internal(e.to_string()))?;

    let user_resp = serde_json::to_value(user.to_response()).unwrap();
    let cookies = build_auth_cookies(&session_jwt, &refresh_jwt, state.config.session_ttl);
    let body = build_auth_response(&state, &session_jwt, &refresh_jwt, user_resp);
    drop(users);

    // Record login timestamp
    let _ = state.users.write().await.record_login(&req.login_id);

    Ok((cookies, Json(body)))
}

// ─── OTP Verify (phone/SMS) ───────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpVerifyPhoneRequest {
    pub login_id: String,
    pub code: String,
}

pub async fn verify_phone_sms(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<OtpVerifyPhoneRequest>,
) -> Result<(axum::http::HeaderMap, Json<Value>), EmulatorError> {
    let user_id = {
        let users = state.users.read().await;
        let user = users.load(&req.login_id)?;
        if user.status == "disabled" {
            return Err(EmulatorError::UserDisabled);
        }
        user.user_id.clone()
    };

    state.otps.write().await.consume(&user_id, &req.code)?;

    state.users.write().await.patch(
        &req.login_id,
        crate::store::user_store::UserPatch {
            verified_phone: Some(true),
            ..Default::default()
        },
    )?;

    let users = state.users.read().await;
    let user = users.load(&req.login_id)?;
    let tmpl_store = state.jwt_templates.read().await;
    let active_tmpl = tmpl_store.active();
    let session_jwt = generate_session_jwt(
        &*state.km().await,
        user,
        &state.config.project_id,
        state.config.session_ttl,
        active_tmpl,
        &*state.roles.read().await,
        "email",
    )
    .map_err(|e| EmulatorError::Internal(e.to_string()))?;
    let refresh_jwt = generate_refresh_jwt(
        &*state.km().await,
        &user.user_id,
        &state.config.project_id,
        state.config.refresh_ttl,
    )
    .map_err(|e| EmulatorError::Internal(e.to_string()))?;

    let user_resp = serde_json::to_value(user.to_response()).unwrap();
    let cookies = build_auth_cookies(&session_jwt, &refresh_jwt, state.config.session_ttl);
    let body = build_auth_response(&state, &session_jwt, &refresh_jwt, user_resp);
    drop(users);

    // Record login timestamp
    let _ = state.users.write().await.record_login(&req.login_id);

    Ok((cookies, Json(body)))
}

// ─── Update phone (existing) ──────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePhoneRequest {
    pub login_id: String,
    pub phone: String,
    pub options: Option<OtpOptions>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpOptions {
    pub add_to_login_i_ds: Option<bool>,
}

pub async fn update_phone(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<UpdatePhoneRequest>,
) -> Result<Json<Value>, EmulatorError> {
    let add_to_ids = req
        .options
        .as_ref()
        .and_then(|o| o.add_to_login_i_ds)
        .unwrap_or(false);

    // Stage the new phone on the user. Real Descope sends an OTP to the new
    // number and leaves it unverified until that code is confirmed via
    // /v1/auth/otp/verify/sms — so we issue a code here instead of marking the
    // phone verified outright. Without this the code never lands in the OTP
    // store and never surfaces in the admin UI / `/emulator/otps`.
    let user_id = {
        let mut users = state.users.write().await;
        let user = users.load_mut(&req.login_id)?;
        user.phone = Some(req.phone.clone());
        user.verified_phone = false;
        if add_to_ids && !user.login_ids.contains(&req.phone) {
            user.login_ids.push(req.phone.clone());
        }
        user.user_id.clone()
    };

    let code = generate_otp_code();
    state.otps.write().await.store(&user_id, code.clone());
    tracing::info!(login_id = %req.login_id, code = %code, "\u{1f4f1} OTP generated (update phone)");

    // Fire-and-forget: notify the configured SMS connector (if any).
    let sms_connector_id = state
        .auth_method_config
        .read()
        .await
        .get()
        .otp
        .sms_connector_id
        .clone();
    if let Some(cid) = sms_connector_id {
        if let Ok(c) = state.connectors.read().await.load(&cid).cloned() {
            let inv = state.invoker.clone();
            let payload = otp_payload(&req.phone, &code, "sms");
            tokio::spawn(async move { inv.invoke(&c, payload).await });
        }
    }

    let masked = masked_phone(&req.phone);
    Ok(Json(json!({ "maskedPhone": masked, "code": code })))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::EmulatorConfig, state::EmulatorState, store::user_store::new_user_id, types::User,
    };

    async fn make_state() -> EmulatorState {
        let config = EmulatorConfig::default();
        EmulatorState::new(&config).await.unwrap()
    }

    async fn insert_user(state: &EmulatorState, login_id: &str) -> String {
        let uid = new_user_id();
        let mut u = User::default();
        u.user_id = uid.clone();
        u.login_ids = vec![login_id.to_string()];
        u.email = Some(login_id.to_string());
        u.phone = Some("+1555000000".to_string());
        u.status = "enabled".into();
        u.created_time = 0;
        state.users.write().await.insert(u).unwrap();
        uid
    }

    // Why: phone-first flows (e.g. an SMS-OTP forgot-password reset) sign in
    //      and verify by the user's phone number while the primary loginId
    //      stays the email — real Descope resolves the user by phone for SMS OTP.
    //      Without phone resolution both calls 404 the user (E112102) and the
    //      reset flow dead-ends on the verify screen.
    // Decision: signin/verify resolve via the store's phone fallback; the OTP
    //           issued at sign-in verifies under the same user.
    #[tokio::test]
    async fn signin_and_verify_sms_resolve_user_by_phone_field() {
        let state = make_state().await;
        let uid = new_user_id();
        let mut u = User::default();
        u.user_id = uid.clone();
        u.login_ids = vec!["phone-first@test.com".into()];
        u.email = Some("phone-first@test.com".into());
        u.phone = Some("+15550100400".into());
        u.status = "enabled".into();
        state.users.write().await.insert(u).unwrap();

        let send = signin_phone_sms(
            State(state.clone()),
            PermissiveJson(OtpSigninPhoneRequest {
                login_id: "+15550100400".into(),
            }),
        )
        .await
        .unwrap();
        let code = send.0["code"].as_str().unwrap().to_string();

        let (_, body) = verify_phone_sms(
            State(state.clone()),
            PermissiveJson(OtpVerifyPhoneRequest {
                login_id: "+15550100400".into(),
                code,
            }),
        )
        .await
        .unwrap();
        assert_eq!(body["user"]["userId"].as_str().unwrap(), uid);
        assert!(body["sessionJwt"].as_str().is_some());
    }

    // ─── signup email ─────────────

    #[tokio::test]
    async fn signup_email_creates_user_and_returns_code() {
        let state = make_state().await;
        let result = signup_email(
            State(state.clone()),
            PermissiveJson(OtpSignupEmailRequest {
                login_id: "new@test.com".into(),
            }),
        )
        .await
        .unwrap();
        let body = result.0;
        assert!(body["maskedEmail"].as_str().is_some());
        let code = body["code"].as_str().unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        // User should exist
        assert!(state.users.read().await.load("new@test.com").is_ok());
    }

    #[tokio::test]
    async fn signup_email_duplicate_login_id_returns_conflict() {
        let state = make_state().await;
        insert_user(&state, "dup@test.com").await;
        let err = signup_email(
            State(state),
            PermissiveJson(OtpSignupEmailRequest {
                login_id: "dup@test.com".into(),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::UserAlreadyExists));
    }

    // ─── signup phone/sms ─────────

    #[tokio::test]
    async fn signup_phone_sms_creates_user_and_returns_code() {
        let state = make_state().await;
        let result = signup_phone_sms(
            State(state.clone()),
            PermissiveJson(OtpSignupPhoneRequest {
                login_id: "+15551230000".into(),
            }),
        )
        .await
        .unwrap();
        let body = result.0;
        assert!(body["maskedPhone"].as_str().is_some());
        assert_eq!(body["code"].as_str().unwrap().len(), 6);
    }

    #[tokio::test]
    async fn signup_phone_sms_duplicate_returns_conflict() {
        let state = make_state().await;
        let _ = signup_phone_sms(
            State(state.clone()),
            PermissiveJson(OtpSignupPhoneRequest {
                login_id: "+15559999999".into(),
            }),
        )
        .await
        .unwrap();
        let err = signup_phone_sms(
            State(state),
            PermissiveJson(OtpSignupPhoneRequest {
                login_id: "+15559999999".into(),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::UserAlreadyExists));
    }

    // ─── signin email ─────────────

    #[tokio::test]
    async fn signin_email_returns_code_for_existing_user() {
        let state = make_state().await;
        insert_user(&state, "alice@test.com").await;
        let result = signin_email(
            State(state),
            PermissiveJson(OtpSigninEmailRequest {
                login_id: "alice@test.com".into(),
            }),
        )
        .await
        .unwrap();
        let body = result.0;
        assert!(body["maskedEmail"].as_str().is_some());
        assert_eq!(body["code"].as_str().unwrap().len(), 6);
    }

    #[tokio::test]
    async fn signin_email_unknown_user_auto_provisions() {
        let state = make_state().await;
        let result = signin_email(
            State(state.clone()),
            PermissiveJson(OtpSigninEmailRequest {
                login_id: "ghost@test.com".into(),
            }),
        )
        .await
        .unwrap();
        let body = result.0;
        assert!(body["maskedEmail"].as_str().is_some());
        assert_eq!(body["code"].as_str().unwrap().len(), 6);
        // Verify the user was auto-created
        let users = state.users.read().await;
        assert!(users.load("ghost@test.com").is_ok());
    }

    // ─── signin phone/sms ─────────

    #[tokio::test]
    async fn signin_phone_returns_code_for_existing_user() {
        let state = make_state().await;
        insert_user(&state, "+15550001111").await;
        let result = signin_phone_sms(
            State(state),
            PermissiveJson(OtpSigninPhoneRequest {
                login_id: "+15550001111".into(),
            }),
        )
        .await
        .unwrap();
        assert_eq!(result.0["code"].as_str().unwrap().len(), 6);
    }

    #[tokio::test]
    async fn signin_phone_unknown_user_returns_not_found() {
        let state = make_state().await;
        let err = signin_phone_sms(
            State(state),
            PermissiveJson(OtpSigninPhoneRequest {
                login_id: "+00000000000".into(),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::UserNotFound));
    }

    // ─── verify email ─────────────

    #[tokio::test]
    async fn verify_email_valid_code_returns_tokens() {
        let state = make_state().await;
        let uid = insert_user(&state, "bob@test.com").await;
        state.otps.write().await.store(&uid, "654321".into());

        let (_, result) = verify_email(
            State(state.clone()),
            PermissiveJson(OtpVerifyEmailRequest {
                login_id: "bob@test.com".into(),
                code: "654321".into(),
            }),
        )
        .await
        .unwrap();
        assert!(result["sessionJwt"].as_str().is_some());
        assert!(result["refreshJwt"].as_str().is_some());
        // verified_email should be set to true
        let u = state.users.read().await;
        assert!(u.load("bob@test.com").unwrap().verified_email);
    }

    #[tokio::test]
    async fn verify_email_code_is_single_use() {
        let state = make_state().await;
        let uid = insert_user(&state, "carol@test.com").await;
        state.otps.write().await.store(&uid, "111111".into());
        let _ = verify_email(
            State(state.clone()),
            PermissiveJson(OtpVerifyEmailRequest {
                login_id: "carol@test.com".into(),
                code: "111111".into(),
            }),
        )
        .await
        .unwrap();
        let err = verify_email(
            State(state),
            PermissiveJson(OtpVerifyEmailRequest {
                login_id: "carol@test.com".into(),
                code: "111111".into(),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }

    #[tokio::test]
    async fn verify_email_wrong_code_returns_invalid_token() {
        let state = make_state().await;
        let uid = insert_user(&state, "dan@test.com").await;
        state.otps.write().await.store(&uid, "222222".into());
        let err = verify_email(
            State(state),
            PermissiveJson(OtpVerifyEmailRequest {
                login_id: "dan@test.com".into(),
                code: "000000".into(),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }

    // ─── verify phone/sms ─────────

    #[tokio::test]
    async fn verify_phone_sms_valid_code_returns_tokens() {
        let state = make_state().await;
        let uid = insert_user(&state, "+15550002222").await;
        state.otps.write().await.store(&uid, "333333".into());
        let (_, result) = verify_phone_sms(
            State(state.clone()),
            PermissiveJson(OtpVerifyPhoneRequest {
                login_id: "+15550002222".into(),
                code: "333333".into(),
            }),
        )
        .await
        .unwrap();
        assert!(result["sessionJwt"].as_str().is_some());
        let u = state.users.read().await;
        assert!(u.load("+15550002222").unwrap().verified_phone);
    }

    #[tokio::test]
    async fn verify_phone_sms_code_is_single_use() {
        let state = make_state().await;
        let uid = insert_user(&state, "+15550003333").await;
        state.otps.write().await.store(&uid, "444444".into());
        let _ = verify_phone_sms(
            State(state.clone()),
            PermissiveJson(OtpVerifyPhoneRequest {
                login_id: "+15550003333".into(),
                code: "444444".into(),
            }),
        )
        .await
        .unwrap();
        let err = verify_phone_sms(
            State(state),
            PermissiveJson(OtpVerifyPhoneRequest {
                login_id: "+15550003333".into(),
                code: "444444".into(),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }

    #[tokio::test]
    async fn verify_phone_sms_wrong_code_returns_invalid_token() {
        let state = make_state().await;
        let uid = insert_user(&state, "+15550004444").await;
        state.otps.write().await.store(&uid, "555555".into());
        let err = verify_phone_sms(
            State(state),
            PermissiveJson(OtpVerifyPhoneRequest {
                login_id: "+15550004444".into(),
                code: "000000".into(),
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }

    // ─── update phone ─────────────

    // Why: an authenticated phone update (otp.update.phone.sms) must issue a
    //      code the caller can later verify. The handler used to mark the phone
    //      verified outright and store no code, so `/emulator/otps` (and the
    //      admin UI) stayed empty and there was nothing to verify.
    #[tokio::test]
    async fn update_phone_issues_otp_and_defers_verification() {
        let state = make_state().await;
        let uid = insert_user(&state, "otp-update@test.com").await;

        let result = update_phone(
            State(state.clone()),
            PermissiveJson(UpdatePhoneRequest {
                login_id: "otp-update@test.com".into(),
                phone: "+15550107000".into(),
                options: Some(OtpOptions {
                    add_to_login_i_ds: Some(true),
                }),
            }),
        )
        .await
        .unwrap();

        // A 6-digit code is returned and stored under the user, so the admin UI
        // / GET /emulator/otps can surface it.
        let code = result.0["code"].as_str().unwrap().to_string();
        assert_eq!(code.len(), 6);
        assert!(result.0["maskedPhone"].as_str().is_some());
        assert_eq!(state.otps.read().await.peek(&uid), Some(code.as_str()));

        // Phone staged + added as a login id, but NOT yet verified.
        {
            let users = state.users.read().await;
            let u = users.load("otp-update@test.com").unwrap();
            assert_eq!(u.phone.as_deref(), Some("+15550107000"));
            assert!(!u.verified_phone);
            assert!(u.login_ids.contains(&"+15550107000".to_string()));
        }

        // Verifying the issued code flips verified_phone to true and mints tokens.
        let (_, body) = verify_phone_sms(
            State(state.clone()),
            PermissiveJson(OtpVerifyPhoneRequest {
                login_id: "otp-update@test.com".into(),
                code,
            }),
        )
        .await
        .unwrap();
        assert!(body["sessionJwt"].as_str().is_some());
        assert!(
            state
                .users
                .read()
                .await
                .load("otp-update@test.com")
                .unwrap()
                .verified_phone
        );
    }

    // ─── signup_in_email ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn signup_in_email_existing_user_gets_otp_no_duplicate() {
        let state = make_state().await;
        insert_user(&state, "existing@test.com").await;
        let before_count = state.users.read().await.all_users().len();
        let result = signup_in_email(
            State(state.clone()),
            PermissiveJson(OtpSignupInEmailRequest {
                login_id: "existing@test.com".into(),
            }),
        )
        .await
        .unwrap();
        let after_count = state.users.read().await.all_users().len();
        assert_eq!(before_count, after_count, "no new user should be created");
        assert!(result["maskedEmail"].as_str().is_some());
        assert!(result["code"].as_str().is_some());
    }

    #[tokio::test]
    async fn signup_in_email_new_user_is_created_and_gets_otp() {
        let state = make_state().await;
        let result = signup_in_email(
            State(state.clone()),
            PermissiveJson(OtpSignupInEmailRequest {
                login_id: "new@test.com".into(),
            }),
        )
        .await
        .unwrap();
        assert!(state.users.read().await.load("new@test.com").is_ok());
        let code = result["code"].as_str().unwrap().to_string();
        // OTP is verifiable
        let (_, tokens) = verify_email(
            State(state),
            PermissiveJson(OtpVerifyEmailRequest {
                login_id: "new@test.com".into(),
                code,
            }),
        )
        .await
        .unwrap();
        assert!(tokens["sessionJwt"].as_str().is_some());
    }

    // ─── signup_in_phone_sms ─────────────────────────────────────────────────

    #[tokio::test]
    async fn signup_in_sms_existing_user_gets_otp_no_duplicate() {
        let state = make_state().await;
        let mut u = crate::types::User::default();
        u.user_id = crate::store::user_store::new_user_id();
        u.login_ids = vec!["+15550009999".into()];
        u.phone = Some("+15550009999".into());
        u.status = "enabled".into();
        state.users.write().await.insert(u).unwrap();
        let before_count = state.users.read().await.all_users().len();
        let result = signup_in_phone_sms(
            State(state.clone()),
            PermissiveJson(OtpSignupInPhoneRequest {
                login_id: "+15550009999".into(),
            }),
        )
        .await
        .unwrap();
        let after_count = state.users.read().await.all_users().len();
        assert_eq!(before_count, after_count);
        assert!(result["maskedPhone"].as_str().is_some());
        assert!(result["code"].as_str().is_some());
    }

    #[tokio::test]
    async fn signup_in_sms_new_user_is_created_and_gets_otp() {
        let state = make_state().await;
        let result = signup_in_phone_sms(
            State(state.clone()),
            PermissiveJson(OtpSignupInPhoneRequest {
                login_id: "+15550008888".into(),
            }),
        )
        .await
        .unwrap();
        assert!(state.users.read().await.load("+15550008888").is_ok());
        let code = result["code"].as_str().unwrap().to_string();
        let (_, tokens) = verify_phone_sms(
            State(state),
            PermissiveJson(OtpVerifyPhoneRequest {
                login_id: "+15550008888".into(),
                code,
            }),
        )
        .await
        .unwrap();
        assert!(tokens["sessionJwt"].as_str().is_some());
    }
}

// ─── OTP Signup-In (email) ────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpSignupInEmailRequest {
    pub login_id: String,
}

pub async fn signup_in_email(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<OtpSignupInEmailRequest>,
) -> Result<Json<Value>, EmulatorError> {
    // Load-or-create: try to find existing user, create if not found
    let (user_id, email) = {
        let users = state.users.read().await;
        match users.load(&req.login_id) {
            Ok(user) => (
                user.user_id.clone(),
                user.email.clone().unwrap_or_else(|| req.login_id.clone()),
            ),
            Err(EmulatorError::UserNotFound) => {
                drop(users);
                let uid = new_user_id();
                state.users.write().await.insert(User {
                    user_id: uid.clone(),
                    login_ids: vec![req.login_id.clone()],
                    email: Some(req.login_id.clone()),
                    verified_email: false,
                    status: "enabled".into(),
                    created_time: now(),
                    ..Default::default()
                })?;
                (uid, req.login_id.clone())
            }
            Err(e) => return Err(e),
        }
    };

    let code = generate_otp_code();
    state.otps.write().await.store(&user_id, code.clone());
    tracing::info!(login_id = %req.login_id, code = %code, "📧 OTP generated");

    let masked = masked_email(&email);
    Ok(Json(json!({ "maskedEmail": masked, "code": code })))
}

// ─── OTP Signup-In (phone/SMS) ────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpSignupInPhoneRequest {
    pub login_id: String,
}

pub async fn signup_in_phone_sms(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<OtpSignupInPhoneRequest>,
) -> Result<Json<Value>, EmulatorError> {
    let (user_id, phone) = {
        let users = state.users.read().await;
        match users.load(&req.login_id) {
            Ok(user) => (
                user.user_id.clone(),
                user.phone.clone().unwrap_or_else(|| req.login_id.clone()),
            ),
            Err(EmulatorError::UserNotFound) => {
                drop(users);
                let uid = new_user_id();
                state.users.write().await.insert(User {
                    user_id: uid.clone(),
                    login_ids: vec![req.login_id.clone()],
                    phone: Some(req.login_id.clone()),
                    verified_phone: false,
                    status: "enabled".into(),
                    created_time: now(),
                    ..Default::default()
                })?;
                (uid, req.login_id.clone())
            }
            Err(e) => return Err(e),
        }
    };

    let code = generate_otp_code();
    state.otps.write().await.store(&user_id, code.clone());
    tracing::info!(login_id = %req.login_id, code = %code, "📱 OTP generated");

    let masked = masked_phone(&phone);
    Ok(Json(json!({ "maskedPhone": masked, "code": code })))
}
