/// AuthPolicyGuard — checks per-method enabled status and per-loginId lockout state.
///
/// Used by OTP, magic link, and password route handlers to enforce:
/// 1. Auth method enabled (from AuthMethodConfigStore)
/// 2. Per-loginId lockout after repeated failures
use crate::{error::EmulatorError, state::EmulatorState};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_FAILURES: u32 = 5;
const LOCKOUT_WINDOW_SECS: u64 = 300; // 5 minutes

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub struct AuthPolicyGuard;

impl AuthPolicyGuard {
    /// Check that the given auth method is enabled. Returns Unauthorized if disabled.
    pub async fn check_method_enabled(
        state: &EmulatorState,
        method: &str,
    ) -> Result<(), EmulatorError> {
        let config = state.auth_method_config.read().await;
        if config.is_enabled(method) {
            Ok(())
        } else {
            Err(EmulatorError::AuthMethodDisabled)
        }
    }

    /// Check the lockout state for a loginId.
    /// Returns `Err(EmulatorError::TooManyRequests)` if locked out.
    pub async fn check_not_locked_out(
        state: &EmulatorState,
        login_id: &str,
    ) -> Result<(), EmulatorError> {
        let lockouts = state.lockouts.read().await;
        if let Some(&(failures, first_at)) = lockouts.get(login_id) {
            let elapsed = now_secs().saturating_sub(first_at);
            if failures >= MAX_FAILURES && elapsed < LOCKOUT_WINDOW_SECS {
                return Err(EmulatorError::TooManyRequests);
            }
        }
        Ok(())
    }

    /// Record a failed auth attempt for a loginId. Increments failure counter;
    /// resets window if LOCKOUT_WINDOW_SECS has elapsed.
    pub async fn record_failure(state: &EmulatorState, login_id: &str) {
        let mut lockouts = state.lockouts.write().await;
        let now = now_secs();
        let entry = lockouts.entry(login_id.to_string()).or_insert((0, now));
        let elapsed = now.saturating_sub(entry.1);
        if elapsed >= LOCKOUT_WINDOW_SECS {
            *entry = (1, now);
        } else {
            entry.0 += 1;
        }
    }

    /// Clear the lockout record for a loginId on successful auth.
    pub async fn clear_failures(state: &EmulatorState, login_id: &str) {
        state.lockouts.write().await.remove(login_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::EmulatorConfig, state::EmulatorState};

    async fn make_state() -> EmulatorState {
        EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn not_locked_out_initially() {
        let state = make_state().await;
        assert!(
            AuthPolicyGuard::check_not_locked_out(&state, "user@test.com")
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn locked_out_after_max_failures() {
        let state = make_state().await;
        for _ in 0..MAX_FAILURES {
            AuthPolicyGuard::record_failure(&state, "user@test.com").await;
        }
        let result = AuthPolicyGuard::check_not_locked_out(&state, "user@test.com").await;
        assert!(
            matches!(result, Err(EmulatorError::TooManyRequests)),
            "should be locked out after {MAX_FAILURES} failures"
        );
    }

    #[tokio::test]
    async fn failure_cleared_on_success() {
        let state = make_state().await;
        for _ in 0..MAX_FAILURES {
            AuthPolicyGuard::record_failure(&state, "user@test.com").await;
        }
        AuthPolicyGuard::clear_failures(&state, "user@test.com").await;
        assert!(
            AuthPolicyGuard::check_not_locked_out(&state, "user@test.com")
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn method_enabled_by_default() {
        let state = make_state().await;
        // All methods should be enabled by default
        for method in &["otp", "magic_link", "password"] {
            assert!(
                AuthPolicyGuard::check_method_enabled(&state, method)
                    .await
                    .is_ok(),
                "method {method} should be enabled by default"
            );
        }
    }
}
