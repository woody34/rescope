pub mod idp_oidc;
pub mod idp_saml;
pub mod snapshot;

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{error::EmulatorError, extractor::PermissiveJson, seed, state::EmulatorState};

pub async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}

pub async fn reset(State(state): State<EmulatorState>) -> Json<Value> {
    state.reset_stores().await;

    // Re-apply seed if configured
    if let Some(seed_path) = &state.config.seed_file.clone() {
        // Best-effort — log errors but don't fail the reset
        if let Err(e) = seed::load(seed_path, &state).await {
            tracing::warn!(error = %e, "Seed re-apply failed during reset");
        }
    }

    Json(json!({ "ok": true }))
}

/// GET /emulator/otp/:loginId
/// Returns the last-issued OTP code for the given login ID without consuming it.
/// This is an emulator-specific escape hatch for SDK-driven test flows.
pub async fn get_otp(
    State(state): State<EmulatorState>,
    Path(login_id): Path<String>,
) -> Result<Json<Value>, EmulatorError> {
    let user_id = {
        let users = state.users.read().await;
        let user = users.load(&login_id)?;
        user.user_id.clone()
    };

    let otps = state.otps.read().await;
    let code = otps.peek(&user_id).ok_or(EmulatorError::InvalidToken)?;
    Ok(Json(json!({ "code": code })))
}

/// POST /emulator/seed/users
/// Batch-creates users without requiring management auth.
/// Body: { "users": [{ "loginId": "...", "email": "...", "name": "...", "password": "..." }, ...] }
/// The optional `password` field sets a bcrypt-hashed password on the user after creation.

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedUsersRequest {
    pub users: Vec<SeedUserEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedUserEntry {
    #[serde(flatten)]
    pub user: crate::routes::mgmt::user::CreateUserRequest,
    pub password: Option<String>,
}

pub async fn seed_users(
    State(state): State<EmulatorState>,
    PermissiveJson(req): PermissiveJson<SeedUsersRequest>,
) -> Result<Json<Value>, EmulatorError> {
    let mut created = vec![];
    let mut failed = vec![];
    for entry in req.users {
        let login_id = entry
            .user
            .resolved_login_id()
            .ok_or_else(|| EmulatorError::Internal("loginId is required".into()))?;
        // Seed path: honor each user's customAttributes.uid as the userId so it
        // matches the backend seed's descopeId (findById/loadByUserId resolve on it).
        match crate::routes::mgmt::user::create_user_impl(&state, entry.user, false, true).await {
            Ok(_r) => {
                // Seed users are pre-existing — set status to "enabled"
                let _ = state.users.write().await.set_status(&login_id, "enabled");
                if let Some(plain) = entry.password {
                    let hash = bcrypt::hash(&plain, 4)
                        .map_err(|e| EmulatorError::Internal(format!("bcrypt error: {e}")))?;
                    let _ = state.users.write().await.set_password(&login_id, hash);
                }
                let users = state.users.read().await;
                if let Ok(user) = users.load(&login_id) {
                    created.push(serde_json::to_value(user.to_response()).unwrap_or_default());
                }
            }
            Err(EmulatorError::UserAlreadyExists) => {
                // Skip existing users — idempotent seed behavior
                let users = state.users.read().await;
                if let Ok(user) = users.load(&login_id) {
                    created.push(serde_json::to_value(user.to_response()).unwrap_or_default());
                }
            }
            Err(e) => {
                failed.push(json!({ "loginId": login_id, "error": e.to_string() }));
            }
        }
    }
    Ok(Json(
        json!({ "createdUsers": created, "failedUsers": failed }),
    ))
}

/// POST /emulator/tenant
/// Creates a SAML tenant in the emulator's tenant store for testing.
/// Body: { "id": "...", "name": "...", "domains": ["example.com"], "authType": "saml" }
pub async fn create_tenant(
    State(state): State<EmulatorState>,
    Json(body): Json<serde_json::Value>,
) -> Json<Value> {
    use crate::types::{AuthType, Tenant};
    let id = body["id"].as_str().unwrap_or("test-tenant").to_string();
    let name = body["name"].as_str().unwrap_or(&id).to_string();
    let domains: Vec<String> = body["domains"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let auth_type = match body["authType"].as_str().unwrap_or("saml") {
        "oidc" => AuthType::Oidc,
        _ => AuthType::Saml,
    };
    let tenant = Tenant {
        id: id.clone(),
        name,
        domains,
        auth_type,
        ..Default::default()
    };
    state.tenants.write().await.insert(tenant);
    Json(json!({ "ok": true, "id": id }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::EmulatorConfig,
        state::EmulatorState,
        store::user_store::new_user_id,
        types::{TokenType, User},
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
        u.status = "enabled".into();
        state.users.write().await.insert(u).unwrap();
        uid
    }

    #[tokio::test]
    async fn get_otp_returns_pending_code_without_consuming() {
        let state = make_state().await;
        let uid = insert_user(&state, "test@example.com").await;
        state.otps.write().await.store(&uid, "987654".into());

        let result = get_otp(State(state.clone()), Path("test@example.com".into()))
            .await
            .unwrap();
        assert_eq!(result["code"].as_str().unwrap(), "987654");
        // Code should still be present after peek
        assert!(state.otps.read().await.peek(&uid).is_some());
    }

    #[tokio::test]
    async fn get_otp_unknown_login_id_returns_user_not_found() {
        let state = make_state().await;
        let err = get_otp(State(state), Path("ghost@test.com".into()))
            .await
            .unwrap_err();
        assert!(matches!(err, EmulatorError::UserNotFound));
    }

    #[tokio::test]
    async fn get_otp_no_pending_code_returns_invalid_token() {
        let state = make_state().await;
        insert_user(&state, "no-code@test.com").await;
        let err = get_otp(State(state), Path("no-code@test.com".into()))
            .await
            .unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }

    #[tokio::test]
    async fn reset_preserves_in_flight_magic_link_tokens() {
        // Why: parallel test shards share one emulator process. A /emulator/reset
        // issued by one shard must not destroy a magic-link token another shard
        // just minted and is about to verify — otherwise verify fails with a
        // flaky "Failed to load magic link token" (token not found).
        let state = make_state().await;
        state.tokens.write().await.insert(
            "in-flight-token".into(),
            "user-1".into(),
            TokenType::Magic,
        );

        // A concurrent test-isolation reset runs.
        reset(State(state.clone())).await;

        // The in-flight token must still be verifiable after the reset.
        let entry = state
            .tokens
            .write()
            .await
            .consume("in-flight-token")
            .expect("magic-link token must survive /emulator/reset");
        assert_eq!(entry.user_id, "user-1");
    }
}
