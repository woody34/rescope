//! WebAuthn (passkey) sign-up, sign-in and update, as `/v1/auth/webauthn/*`.
//!
//! Each ceremony is a start that returns the options for
//! `navigator.credentials.create` / `.get`, and a finish that takes the
//! browser's response. Flows run the same ceremonies through
//! `flow_engine` (`webauthn-*-start` / `-finish` actions).

use crate::{
    cookies::build_auth_cookies,
    error::EmulatorError,
    extractor::PermissiveJson,
    jwt::token_generator::{generate_refresh_jwt, generate_session_jwt},
    routes::auth::password::build_jwt_response,
    state::EmulatorState,
    store::{token_store::generate_token, user_store::new_user_id, webauthn_store::Ceremony},
    types::User,
};
use axum::{extract::State, http::HeaderMap, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// The relying party id is the host of the page that asked.
fn rp_id(origin: Option<&str>) -> String {
    let origin = origin.unwrap_or("http://localhost");
    let host = origin.split("://").nth(1).unwrap_or(origin);
    host.split(['/', ':', '?', '#'])
        .next()
        .filter(|h| !h.is_empty())
        .unwrap_or("localhost")
        .to_string()
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SignUpUser {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRequest {
    pub login_id: String,
    pub origin: Option<String>,
    #[serde(default)]
    pub user: Option<SignUpUser>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinishRequest {
    pub transaction_id: String,
    pub response: String,
}

async fn begin(
    state: &EmulatorState,
    ceremony: Ceremony,
    req: &StartRequest,
    user_id: &str,
    display_name: &str,
) -> Json<Value> {
    let challenge: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
    let (transaction_id, options) = state.webauthn.write().await.begin(
        ceremony,
        &req.login_id,
        user_id,
        display_name,
        &rp_id(req.origin.as_deref()),
        now(),
        generate_token(),
        challenge,
    );
    Json(json!({
        "transactionId": transaction_id,
        "options": options,
        "create": ceremony != Ceremony::SignIn
    }))
}

/// POST /v1/auth/webauthn/signup/start
pub async fn signup_start(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<StartRequest>,
) -> Result<Json<Value>, EmulatorError> {
    if state.users.read().await.load(&req.login_id).is_ok() {
        return Err(EmulatorError::UserAlreadyExists);
    }
    let name = req
        .user
        .as_ref()
        .and_then(|u| u.name.clone())
        .unwrap_or_default();
    Ok(begin(&state, Ceremony::SignUp, &req, "", &name).await)
}

/// POST /v1/auth/webauthn/signin/start
pub async fn signin_start(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<StartRequest>,
) -> Result<Json<Value>, EmulatorError> {
    let (user_id, name) = {
        let users = state.users.read().await;
        let user = users.load(&req.login_id)?;
        (user.user_id.clone(), user.name.clone().unwrap_or_default())
    };
    if !state.webauthn.read().await.has_credentials(&user_id) {
        return Err(EmulatorError::ValidationError(
            "WebAuthn: the user has no passkey".into(),
        ));
    }
    Ok(begin(&state, Ceremony::SignIn, &req, &user_id, &name).await)
}

/// POST /v1/auth/webauthn/update/start
pub async fn update_start(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<StartRequest>,
) -> Result<Json<Value>, EmulatorError> {
    let (user_id, name) = {
        let users = state.users.read().await;
        let user = users.load(&req.login_id)?;
        (user.user_id.clone(), user.name.clone().unwrap_or_default())
    };
    Ok(begin(&state, Ceremony::Update, &req, &user_id, &name).await)
}

async fn session_for(
    state: &EmulatorState,
    user_id: &str,
    first_seen: bool,
) -> Result<(HeaderMap, Json<Value>), EmulatorError> {
    let has_passkey = state.webauthn.read().await.has_credentials(user_id);
    let users = state.users.read().await;
    let user = users.load_by_user_id(user_id)?;
    if user.status == "disabled" {
        return Err(EmulatorError::UserDisabled);
    }
    let km = state.km().await;
    let session = {
        let templates = state.jwt_templates.read().await;
        generate_session_jwt(
            &km,
            user,
            &state.config.project_id,
            state.config.session_ttl,
            templates.active(),
            &*state.roles.read().await,
            "webauthn",
        )
        .map_err(|e| EmulatorError::Internal(e.to_string()))?
    };
    let refresh = generate_refresh_jwt(
        &km,
        &user.user_id,
        &state.config.project_id,
        state.config.refresh_ttl,
    )
    .map_err(|e| EmulatorError::Internal(e.to_string()))?;
    let mut user_resp = serde_json::to_value(user.to_response()).unwrap_or_default();
    user_resp["webauthn"] = json!(has_passkey);
    let mut body = build_jwt_response(state, user_id, &session, &refresh, user_resp);
    body["firstSeen"] = json!(first_seen);
    drop(users);
    let _ = state.users.write().await.record_login_by_user_id(user_id);
    Ok((
        build_auth_cookies(&session, &refresh, state.config.session_ttl),
        Json(body),
    ))
}

/// POST /v1/auth/webauthn/signup/finish
pub async fn signup_finish(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<FinishRequest>,
) -> Result<(HeaderMap, Json<Value>), EmulatorError> {
    let transaction = state
        .webauthn
        .write()
        .await
        .take_transaction(&req.transaction_id, Ceremony::SignUp)?;
    let user_id = new_user_id();
    state
        .webauthn
        .write()
        .await
        .finish_registration(&transaction, &user_id, &req.response)?;
    let login_id = transaction.login_id.to_lowercase();
    state.users.write().await.insert(User {
        user_id: user_id.clone(),
        login_ids: vec![login_id.clone()],
        email: login_id.contains('@').then_some(login_id),
        status: "enabled".into(),
        created_time: now(),
        ..Default::default()
    })?;
    session_for(&state, &user_id, true).await
}

/// POST /v1/auth/webauthn/signin/finish
pub async fn signin_finish(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<FinishRequest>,
) -> Result<(HeaderMap, Json<Value>), EmulatorError> {
    let user_id = {
        let mut webauthn = state.webauthn.write().await;
        let transaction = webauthn.take_transaction(&req.transaction_id, Ceremony::SignIn)?;
        webauthn
            .finish_assertion(&transaction, &req.response)
            .map_err(|_| EmulatorError::InvalidCredentials)?
    };
    session_for(&state, &user_id, false).await
}

/// POST /v1/auth/webauthn/update/finish
pub async fn update_finish(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<FinishRequest>,
) -> Result<Json<Value>, EmulatorError> {
    let mut webauthn = state.webauthn.write().await;
    let transaction = webauthn.take_transaction(&req.transaction_id, Ceremony::Update)?;
    let user_id = transaction.user_id.clone();
    webauthn.finish_registration(&transaction, &user_id, &req.response)?;
    Ok(Json(json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use crate::config::EmulatorConfig;
    use crate::server::build_router;
    use crate::state::EmulatorState;
    use crate::store::webauthn_store::test_support::SoftAuthenticator;
    use axum_test::TestServer;
    use serde_json::{json, Value};

    async fn setup() -> TestServer {
        let state = EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap();
        TestServer::new(build_router(state)).unwrap()
    }

    #[tokio::test]
    async fn a_passkey_signed_up_with_signs_in_again() {
        let server = setup().await;
        let mut authenticator = SoftAuthenticator::new();
        let origin = "http://localhost:4200";

        let start = server
            .post("/v1/auth/webauthn/signup/start")
            .json(&json!({ "loginId": "pk@example.com", "origin": origin, "user": { "name": "Pat" } }))
            .await
            .json::<Value>();
        let created = authenticator.create(start["options"].as_str().unwrap());
        let signed_up = server
            .post("/v1/auth/webauthn/signup/finish")
            .json(&json!({ "transactionId": start["transactionId"], "response": created }))
            .await;
        signed_up.assert_status_ok();
        let body = signed_up.json::<Value>();
        assert_eq!(body["firstSeen"], true);
        assert_eq!(body["user"]["webauthn"], true);

        let start = server
            .post("/v1/auth/webauthn/signin/start")
            .json(&json!({ "loginId": "pk@example.com", "origin": origin }))
            .await
            .json::<Value>();
        assert_eq!(start["create"], false);
        let asserted = authenticator.get(start["options"].as_str().unwrap());
        let signed_in = server
            .post("/v1/auth/webauthn/signin/finish")
            .json(&json!({ "transactionId": start["transactionId"], "response": asserted }))
            .await;

        signed_in.assert_status_ok();
        assert!(signed_in.json::<Value>()["sessionJwt"].as_str().is_some());
    }

    #[tokio::test]
    async fn an_added_passkey_signs_an_existing_user_in() {
        let server = setup().await;
        let mut authenticator = SoftAuthenticator::new();
        server
            .post("/v1/auth/password/signup")
            .json(&json!({ "loginId": "pw@example.com", "password": "Secret123!", "user": { "email": "pw@example.com" } }))
            .await
            .assert_status_ok();

        let start = server
            .post("/v1/auth/webauthn/update/start")
            .json(&json!({ "loginId": "pw@example.com", "origin": "http://localhost:4200" }))
            .await
            .json::<Value>();
        server
            .post("/v1/auth/webauthn/update/finish")
            .json(&json!({
                "transactionId": start["transactionId"],
                "response": authenticator.create(start["options"].as_str().unwrap())
            }))
            .await
            .assert_status_ok();

        let start = server
            .post("/v1/auth/webauthn/signin/start")
            .json(&json!({ "loginId": "pw@example.com", "origin": "http://localhost:4200" }))
            .await
            .json::<Value>();
        server
            .post("/v1/auth/webauthn/signin/finish")
            .json(&json!({
                "transactionId": start["transactionId"],
                "response": authenticator.get(start["options"].as_str().unwrap())
            }))
            .await
            .assert_status_ok();
    }

    #[tokio::test]
    async fn signing_in_without_a_passkey_is_refused() {
        let server = setup().await;
        server
            .post("/v1/auth/password/signup")
            .json(&json!({ "loginId": "nopk@example.com", "password": "Secret123!", "user": {} }))
            .await
            .assert_status_ok();

        server
            .post("/v1/auth/webauthn/signin/start")
            .json(&json!({ "loginId": "nopk@example.com" }))
            .await
            .assert_status_bad_request();
    }
}
