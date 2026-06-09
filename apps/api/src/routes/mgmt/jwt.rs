use crate::extractor::PermissiveJson;
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{
    error::EmulatorError, jwt::token_validator::validate_session_jwt, state::EmulatorState,
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
}
