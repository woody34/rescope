/// Shared management API authentication helper.
///
/// Checks the `Authorization` header in this order:
/// 1. No header → OK (emulator is local-only; UI uses same-origin requests)
/// 2. Bootstrap key: `Bearer <project_id>:<management_key>` → OK
/// 3. Otherwise → 401 Unauthorized
use crate::{config::EmulatorConfig, error::EmulatorError, state::EmulatorState};

pub fn check_mgmt_auth(
    headers: &axum::http::HeaderMap,
    config: &EmulatorConfig,
) -> Result<(), EmulatorError> {
    match headers.get("Authorization").and_then(|v| v.to_str().ok()) {
        // No header → allow (same-origin UI requests don't carry auth)
        None => Ok(()),
        Some(auth) => {
            let expected = format!("Bearer {}:{}", config.project_id, config.management_key);
            if auth == expected {
                Ok(())
            } else {
                Err(EmulatorError::Unauthorized)
            }
        }
    }
}

/// Extended check that also accepts valid access keys from the AccessKeyStore.
/// Falls back to bootstrap key check if no access key matches.
/// Pass `caller_ip` as an optional string (e.g. `"1.2.3.4"`) for IP allowlist checking.
/// If no Authorization header is present, the request is allowed through.
pub async fn check_mgmt_auth_with_keys(
    headers: &axum::http::HeaderMap,
    state: &EmulatorState,
    caller_ip: Option<&str>,
) -> Result<(), EmulatorError> {
    let auth = match headers.get("Authorization").and_then(|v| v.to_str().ok()) {
        // No header → allow (same-origin UI requests don't carry auth)
        None => return Ok(()),
        Some(a) => a,
    };

    // Try bootstrap key first (cheap, no lock needed)
    let expected = format!(
        "Bearer {}:{}",
        state.config.project_id, state.config.management_key
    );
    if auth == expected {
        return Ok(());
    }

    // Try access key: expect `Bearer <raw_key>`
    let raw_key = auth
        .strip_prefix("Bearer ")
        .ok_or(EmulatorError::Unauthorized)?;
    let keys = state.access_keys.read().await;
    keys.validate(raw_key, caller_ip)
        .map(|_| ())
        .map_err(|_| EmulatorError::Unauthorized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::EmulatorConfig, state::EmulatorState};
    use axum::http::{HeaderMap, HeaderValue};

    async fn make_state() -> EmulatorState {
        EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap()
    }

    fn bootstrap_headers(state: &EmulatorState) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(
            "Authorization",
            HeaderValue::from_str(&format!(
                "Bearer {}:{}",
                state.config.project_id, state.config.management_key
            ))
            .unwrap(),
        );
        h
    }

    #[test]
    fn bootstrap_key_accepted() {
        let config = EmulatorConfig::default();
        let mut h = HeaderMap::new();
        h.insert(
            "Authorization",
            HeaderValue::from_str(&format!(
                "Bearer {}:{}",
                config.project_id, config.management_key
            ))
            .unwrap(),
        );
        assert!(check_mgmt_auth(&h, &config).is_ok());
    }

    #[test]
    fn wrong_key_rejected() {
        let config = EmulatorConfig::default();
        let mut h = HeaderMap::new();
        h.insert(
            "Authorization",
            HeaderValue::from_str("Bearer wrong:key").unwrap(),
        );
        assert!(matches!(
            check_mgmt_auth(&h, &config),
            Err(EmulatorError::Unauthorized)
        ));
    }

    #[test]
    fn missing_header_allowed() {
        // No Authorization header → allowed (local emulator UI access)
        let config = EmulatorConfig::default();
        assert!(check_mgmt_auth(&HeaderMap::new(), &config).is_ok());
    }

    #[tokio::test]
    async fn access_key_accepted_by_extended_check() {
        let state = make_state().await;
        let headers = bootstrap_headers(&state);

        // Create a key via the store
        let (_, raw_key) = state
            .access_keys
            .write()
            .await
            .create(
                "test-key".into(),
                None,
                vec![],
                vec![],
                vec![],
                "test".into(),
            )
            .unwrap();

        // Use cleartext key in auth header
        let mut key_headers = HeaderMap::new();
        key_headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {raw_key}")).unwrap(),
        );

        let result = check_mgmt_auth_with_keys(&key_headers, &state, None).await;
        assert!(result.is_ok(), "access key should be accepted");

        // Bootstrap key still works
        let result2 = check_mgmt_auth_with_keys(&headers, &state, None).await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn invalid_bearer_rejected_by_extended_check() {
        let state = make_state().await;
        let mut h = HeaderMap::new();
        h.insert(
            "Authorization",
            HeaderValue::from_str("Bearer invalid_key_xyz").unwrap(),
        );
        let result = check_mgmt_auth_with_keys(&h, &state, None).await;
        assert!(matches!(result, Err(EmulatorError::Unauthorized)));
    }
}
