use crate::extractor::PermissiveJson;
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{
    error::EmulatorError,
    jwt::token_validator::{validate_refresh_jwt, validate_session_jwt},
    state::EmulatorState,
};

// ─── JWT Update ───────────────────────────────────────────────────────────────

/// Descope limits for custom claims inside JWTs.
const MAX_CLAIM_KEY_LENGTH: usize = 60;
const MAX_CLAIM_VALUE_LENGTH: usize = 500;
const MAX_CLAIM_KEYS: usize = 100;

/// POST /v1/mgmt/jwt/update
/// Accepts an existing session JWT + custom claims, returns a new session JWT
/// with the custom claims merged in. Auth: management key.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateJwtRequest {
    pub jwt: String,
    pub custom_claims: Option<HashMap<String, Value>>,
}

pub async fn update(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<UpdateJwtRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;

    // Validate the existing session JWT
    let claims = validate_session_jwt(&*state.km().await, &req.jwt)?;
    let user_id = &claims.sub;

    // Load user to rebuild session JWT
    let users = state.users.read().await;
    let user = users.load_by_user_id(user_id)?;

    // Validate + merge custom claims
    let extra = req.custom_claims.unwrap_or_default();
    validate_custom_claims(&extra)?;

    let new_jwt = crate::jwt::token_generator::generate_session_jwt_with_extra(
        &*state.km().await,
        user,
        &state.config.project_id,
        state.config.session_ttl,
        &extra,
        &*state.roles.read().await,
        "pwd",
    )
    .map_err(|e| EmulatorError::Internal(e.to_string()))?;

    Ok(Json(json!({ "jwt": new_jwt })))
}

/// Validates custom claims against Descope's limits:
/// - Each key ≤ 60 chars
/// - Each serialised value ≤ 500 chars
/// - ≤ 100 keys total
pub fn validate_custom_claims(claims: &HashMap<String, Value>) -> Result<(), EmulatorError> {
    if claims.len() > MAX_CLAIM_KEYS {
        return Err(EmulatorError::ValidationError(format!(
            "JWT can have at most {} custom claim keys (got {})",
            MAX_CLAIM_KEYS,
            claims.len()
        )));
    }
    for (key, value) in claims {
        if key.len() > MAX_CLAIM_KEY_LENGTH {
            return Err(EmulatorError::ValidationError(format!(
                "Custom claim key must be {} characters or fewer (got {} for key '{}')",
                MAX_CLAIM_KEY_LENGTH,
                key.len(),
                key
            )));
        }
        let value_str = value.to_string();
        if value_str.len() > MAX_CLAIM_VALUE_LENGTH {
            return Err(EmulatorError::ValidationError(format!(
                "Custom claim value must be {} characters or fewer when serialised (got {} for key '{}')",
                MAX_CLAIM_VALUE_LENGTH,
                value_str.len(),
                key
            )));
        }
    }
    Ok(())
}

// ─── Impersonation ────────────────────────────────────────────────────────────

/// POST /v1/mgmt/impersonate
///
/// Issues a session token for `login_id` that records `impersonator_id` in the
/// Descope-documented `act` claim. Auth: management key.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpersonateRequest {
    pub impersonator_id: String,
    pub login_id: String,
    /// Accepted for wire compatibility. Descope refuses without the subject's
    /// consent when this is set; the emulator models no consent store, so the
    /// field is read and not enforced.
    #[serde(default)]
    pub validate_consent: Option<bool>,
    #[serde(default)]
    pub custom_claims: Option<HashMap<String, Value>>,
    /// Accepted for wire compatibility. Descope narrows the issued token's
    /// tenant claims to this tenant; the emulator issues the subject's full
    /// tenant set regardless. A test that depends on tenant narrowing will
    /// not reproduce Descope here.
    #[serde(default)]
    pub selected_tenant: Option<String>,
    /// Applied as the issued token's lifetime in seconds. Descope names this
    /// the refresh-token duration and returns a refresh JWT; this emulator
    /// deals in session tokens throughout, like the `jwt/update` route above,
    /// so the value lands on the session token instead. That is what makes an
    /// expiry test possible locally, and it is a deliberate divergence.
    #[serde(default)]
    pub refresh_duration: Option<u64>,
}

pub async fn impersonate(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<ImpersonateRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;

    // The subject is resolved through the store's normal login-id resolution,
    // which also matches on email and on a bare username prefix. An
    // SSO-provisioned login id carries a random suffix after the email, so
    // exact-match-only lookup would miss exactly those users.
    let users = state.users.read().await;
    let subject = users.load(&req.login_id)?;

    // Caller-supplied claims are held to Descope's published limits. They ride on
    // the refresh token alongside the actor so the refresh exchange can put them
    // back on each session it mints, for the same reason the actor is carried.
    let custom = req.custom_claims.unwrap_or_default();
    validate_custom_claims(&custom)?;

    let ttl = req.refresh_duration.unwrap_or(state.config.refresh_ttl);

    // A REFRESH token, not a session token. This is the one a browser can adopt:
    // the Descope SDK takes it, posts it to `/v1/auth/refresh` and stores what
    // comes back. Returning a session token here made the management round trip
    // pass while the browser path silently dropped the impersonation on its
    // first refresh (ENG-2366 spike). Real Descope documents this route as
    // returning a refresh JWT, so this is also the faithful shape.
    let jwt = crate::jwt::token_generator::generate_impersonated_refresh_jwt(
        &*state.km().await,
        &subject.user_id,
        &state.config.project_id,
        ttl,
        Some(json!({ "sub": req.impersonator_id })),
        if custom.is_empty() {
            None
        } else {
            Some(custom)
        },
    )
    .map_err(|e| EmulatorError::Internal(e.to_string()))?;

    Ok(Json(json!({ "jwt": jwt })))
}

/// POST /v1/mgmt/stop/impersonation
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopImpersonationRequest {
    pub jwt: String,
    #[serde(default)]
    pub custom_claims: Option<HashMap<String, Value>>,
    /// Accepted for wire compatibility, not enforced. See
    /// [`ImpersonateRequest::selected_tenant`].
    #[serde(default)]
    pub selected_tenant: Option<String>,
    /// Applied as the issued token's lifetime in seconds. See
    /// [`ImpersonateRequest::refresh_duration`].
    #[serde(default)]
    pub refresh_duration: Option<u64>,
}

pub async fn stop_impersonation(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    PermissiveJson(req): PermissiveJson<StopImpersonationRequest>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;

    let km = state.km().await;

    // The caller hands back what impersonate gave them, which is a refresh
    // token. Validated as one so an expired or forged token is refused before
    // anything is read off it.
    let claims = validate_refresh_jwt(&km, &req.jwt)?;

    // A token with no `act.sub` is not an impersonated session, so there is no
    // actor to return to.
    let actor_id = claims
        .act
        .as_ref()
        .and_then(|act| act.get("sub"))
        .and_then(|sub| sub.as_str())
        .ok_or(EmulatorError::InvalidToken)?;

    let users = state.users.read().await;
    let actor = users.load_by_user_id(actor_id)?;

    let custom = req.custom_claims.unwrap_or_default();
    validate_custom_claims(&custom)?;

    let ttl = req.refresh_duration.unwrap_or(state.config.refresh_ttl);

    // No actor is carried, which is what makes the returned token an ordinary
    // refresh token again. Refreshing it mints a plain session for the operator.
    let jwt = crate::jwt::token_generator::generate_impersonated_refresh_jwt(
        &km,
        &actor.user_id,
        &state.config.project_id,
        ttl,
        None,
        if custom.is_empty() {
            None
        } else {
            Some(custom)
        },
    )
    .map_err(|e| EmulatorError::Internal(e.to_string()))?;

    Ok(Json(json!({ "jwt": jwt })))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::EmulatorConfig, jwt::token_generator::generate_session_jwt, state::EmulatorState,
        store::user_store::new_user_id, types::User,
    };

    async fn make_state() -> EmulatorState {
        let config = EmulatorConfig::default();
        EmulatorState::new(&config).await.unwrap()
    }

    fn make_mgmt_headers(state: &EmulatorState) -> axum::http::HeaderMap {
        let mut h = axum::http::HeaderMap::new();
        let val = format!(
            "Bearer {}:{}",
            state.config.project_id, state.config.management_key
        );
        h.insert("Authorization", val.parse().unwrap());
        h
    }

    async fn insert_user_and_get_session_jwt(
        state: &EmulatorState,
        login_id: &str,
    ) -> (String, String) {
        let uid = new_user_id();
        let mut u = User::default();
        u.user_id = uid.clone();
        u.login_ids = vec![login_id.to_string()];
        u.email = Some(login_id.to_string());
        u.status = "enabled".into();
        state.users.write().await.insert(u).unwrap();

        let user_ref = state.users.read().await;
        let user = user_ref.load(login_id).unwrap();
        let jwt = generate_session_jwt(
            &*state.km().await,
            user,
            &state.config.project_id,
            state.config.session_ttl,
            None,
            &crate::store::role_store::RoleStore::new(),
            "pwd",
        )
        .unwrap();
        (uid, jwt)
    }

    #[tokio::test]
    async fn jwt_update_merges_custom_claims() {
        let state = make_state().await;
        let (_, session_jwt) =
            insert_user_and_get_session_jwt(&state, "jwt-test@example.com").await;

        let mut custom = HashMap::new();
        custom.insert("appRole".to_string(), json!("admin"));

        let result = update(
            State(state.clone()),
            make_mgmt_headers(&state),
            PermissiveJson(UpdateJwtRequest {
                jwt: session_jwt,
                custom_claims: Some(custom),
            }),
        )
        .await
        .unwrap();

        let new_jwt = result["jwt"].as_str().unwrap();
        // Decode without validation to check claims
        let mut v = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        v.validate_exp = false;
        let decoded =
            jsonwebtoken::decode::<serde_json::Value>(new_jwt, &state.km().await.decoding_key, &v)
                .unwrap();
        assert_eq!(decoded.claims["appRole"].as_str().unwrap(), "admin");
    }

    #[tokio::test]
    async fn jwt_update_invalid_session_jwt_returns_unauthorized() {
        let state = make_state().await;
        let err = update(
            State(state.clone()),
            make_mgmt_headers(&state),
            PermissiveJson(UpdateJwtRequest {
                jwt: "not.a.jwt".to_string(),
                custom_claims: None,
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }

    /// Decode a token's claims without validating expiry, for assertions.
    async fn decode_claims(state: &EmulatorState, token: &str) -> serde_json::Value {
        let mut v = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        v.validate_exp = false;
        jsonwebtoken::decode::<serde_json::Value>(token, &state.km().await.decoding_key, &v)
            .unwrap()
            .claims
    }

    fn impersonate_request(impersonator_id: &str, login_id: &str) -> ImpersonateRequest {
        ImpersonateRequest {
            impersonator_id: impersonator_id.to_string(),
            login_id: login_id.to_string(),
            validate_consent: Some(false),
            custom_claims: None,
            selected_tenant: None,
            refresh_duration: None,
        }
    }

    #[tokio::test]
    async fn impersonate_issues_subject_token_carrying_the_actor_in_act_sub() {
        let state = make_state().await;
        let (actor_uid, _) = insert_user_and_get_session_jwt(&state, "actor@example.com").await;
        let (subject_uid, _) = insert_user_and_get_session_jwt(&state, "subject@example.com").await;

        let result = impersonate(
            State(state.clone()),
            make_mgmt_headers(&state),
            PermissiveJson(impersonate_request(&actor_uid, "subject@example.com")),
        )
        .await
        .unwrap();

        let claims = decode_claims(&state, result["jwt"].as_str().unwrap()).await;
        assert_eq!(claims["sub"].as_str().unwrap(), subject_uid);
        assert_eq!(claims["act"]["sub"].as_str().unwrap(), actor_uid);
    }

    #[tokio::test]
    async fn stop_impersonation_returns_the_actor_token_without_an_act_claim() {
        let state = make_state().await;
        let (actor_uid, _) = insert_user_and_get_session_jwt(&state, "actor2@example.com").await;
        insert_user_and_get_session_jwt(&state, "subject2@example.com").await;

        let impersonated = impersonate(
            State(state.clone()),
            make_mgmt_headers(&state),
            PermissiveJson(impersonate_request(&actor_uid, "subject2@example.com")),
        )
        .await
        .unwrap();

        let result = stop_impersonation(
            State(state.clone()),
            make_mgmt_headers(&state),
            PermissiveJson(StopImpersonationRequest {
                jwt: impersonated["jwt"].as_str().unwrap().to_string(),
                custom_claims: None,
                selected_tenant: None,
                refresh_duration: None,
            }),
        )
        .await
        .unwrap();

        let claims = decode_claims(&state, result["jwt"].as_str().unwrap()).await;
        assert_eq!(claims["sub"].as_str().unwrap(), actor_uid);
        assert!(
            claims.get("act").is_none(),
            "act must be absent after stop impersonation, got {:?}",
            claims.get("act")
        );
    }

    /// A *wrong* key is refused. An ABSENT header is deliberately allowed on
    /// every management route (`mgmt_auth`: "No header -> allow"), because the
    /// emulator's own same-origin UI calls carry none. Making impersonation the
    /// single route that rejects an absent header would diverge from the rest
    /// of the management API for no security gain, since anyone who can reach
    /// the emulator can also omit a header.
    #[tokio::test]
    async fn impersonate_requires_a_valid_management_key() {
        let state = make_state().await;
        let (actor_uid, _) = insert_user_and_get_session_jwt(&state, "actor3@example.com").await;
        insert_user_and_get_session_jwt(&state, "subject3@example.com").await;

        let mut wrong = axum::http::HeaderMap::new();
        wrong.insert("Authorization", "Bearer wrong:key".parse().unwrap());

        let err = impersonate(
            State(state.clone()),
            wrong,
            PermissiveJson(impersonate_request(&actor_uid, "subject3@example.com")),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, EmulatorError::Unauthorized));
    }

    #[tokio::test]
    async fn stop_impersonation_requires_a_valid_management_key() {
        let state = make_state().await;

        let mut wrong = axum::http::HeaderMap::new();
        wrong.insert("Authorization", "Bearer wrong:key".parse().unwrap());

        let err = stop_impersonation(
            State(state.clone()),
            wrong,
            PermissiveJson(StopImpersonationRequest {
                jwt: "not.a.jwt".to_string(),
                custom_claims: None,
                selected_tenant: None,
                refresh_duration: None,
            }),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, EmulatorError::Unauthorized));
    }

    /// An ordinary session token names no actor, so there is nothing to return
    /// to. Guards the `act.sub` read against silently minting a token for
    /// whoever happened to be the subject.
    #[tokio::test]
    async fn stop_impersonation_on_a_non_impersonated_token_is_refused() {
        let state = make_state().await;
        let (_, session_jwt) = insert_user_and_get_session_jwt(&state, "plain@example.com").await;

        let err = stop_impersonation(
            State(state.clone()),
            make_mgmt_headers(&state),
            PermissiveJson(StopImpersonationRequest {
                jwt: session_jwt,
                custom_claims: None,
                selected_tenant: None,
                refresh_duration: None,
            }),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, EmulatorError::InvalidToken));
    }

    #[tokio::test]
    async fn impersonate_unknown_login_id_is_refused() {
        let state = make_state().await;
        let (actor_uid, _) = insert_user_and_get_session_jwt(&state, "actor4@example.com").await;

        let err = impersonate(
            State(state.clone()),
            make_mgmt_headers(&state),
            PermissiveJson(impersonate_request(&actor_uid, "nobody@example.com")),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, EmulatorError::UserNotFound));
    }
}
