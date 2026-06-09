use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::{
    error::EmulatorError, state::EmulatorState, store::auth_method_config::AuthMethodConfig,
};

// ── GET /v1/mgmt/config/auth-methods ─────────────────────────────────────────

pub async fn get_auth_methods(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    let cfg = state.auth_method_config.read().await;
    Ok(Json(json!({ "authMethods": cfg.get() })))
}

// ── PUT /v1/mgmt/config/auth-methods ─────────────────────────────────────────

pub async fn put_auth_methods(
    State(state): State<EmulatorState>,
    headers: axum::http::HeaderMap,
    Json(new_config): Json<AuthMethodConfig>,
) -> Result<Json<Value>, EmulatorError> {
    crate::mgmt_auth::check_mgmt_auth_with_keys(&headers, &state, None).await?;
    state.auth_method_config.write().await.replace(new_config);
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
    async fn get_returns_defaults() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let result = get_auth_methods(State(state), headers).await.unwrap();
        let expiry = result["authMethods"]["otp"]["expirationSeconds"]
            .as_u64()
            .unwrap();
        assert_eq!(expiry, 180);
    }

    #[tokio::test]
    async fn put_updates_config() {
        let state = make_state().await;
        let headers = mgmt_headers(&state);
        let mut new_cfg = AuthMethodConfig::new();
        new_cfg.otp.expiration_seconds = 60;
        let _ = put_auth_methods(State(state.clone()), headers.clone(), Json(new_cfg))
            .await
            .unwrap();
        let result = get_auth_methods(State(state), headers).await.unwrap();
        assert_eq!(
            result["authMethods"]["otp"]["expirationSeconds"]
                .as_u64()
                .unwrap(),
            60
        );
    }

    #[tokio::test]
    async fn unauthorized_request_rejected() {
        let state = make_state().await;
        let err = get_auth_methods(State(state), {
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
