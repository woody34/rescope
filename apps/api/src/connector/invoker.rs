/// Connector invocation — fire-and-forget HTTP calls to configured connectors.
///
/// When `DESCOPE_EMULATOR_CONNECTOR_MODE=invoke`, OTP/magic-link/password-reset
/// route handlers call `ConnectorInvoker::notify_*` so that external webhooks,
/// SMTP gateways, or SMS SaaS APIs actually receive payloads.
///
/// In the default `log` mode (or if no matching connector is configured), the
/// invoker simply logs the call without making an outbound request.
use crate::{
    config::EmulatorConfig,
    store::connector_store::{Connector, ConnectorType},
};
use serde_json::{json, Value};
use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectorMode {
    /// Log the payload without making an outbound request (default / safe).
    Log,
    /// Actually POST to the configured endpoint.
    Invoke,
}

impl ConnectorMode {
    pub fn from_env(config: &EmulatorConfig) -> Self {
        match config.connector_mode.as_deref() {
            Some("invoke") => ConnectorMode::Invoke,
            _ => ConnectorMode::Log,
        }
    }
}

pub struct ConnectorInvoker {
    mode: ConnectorMode,
    client: reqwest::Client,
}

impl ConnectorInvoker {
    pub fn new(mode: ConnectorMode) -> Self {
        Self {
            mode,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("failed to build reqwest client"),
        }
    }

    /// Attempt to invoke a connector. Returns immediately (fire-and-forget).
    /// Errors are logged but not propagated so auth flows always succeed.
    pub async fn invoke(&self, connector: &Connector, payload: Value) {
        match &self.mode {
            ConnectorMode::Log => {
                info!(
                    connector_id = %connector.id,
                    connector_name = %connector.name,
                    ?payload,
                    "connector invoke (log-only mode)"
                );
            }
            ConnectorMode::Invoke => {
                if let Err(e) = self.do_invoke(connector, payload).await {
                    warn!(
                        connector_id = %connector.id,
                        error = %e,
                        "connector invoke failed (continuing)"
                    );
                }
            }
        }
    }

    async fn do_invoke(
        &self,
        connector: &Connector,
        payload: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match connector.connector_type {
            ConnectorType::GenericHttp => {
                let base_url = connector
                    .config
                    .get("baseUrl")
                    .and_then(|v| v.as_str())
                    .ok_or("genericHttp connector missing baseUrl")?;

                let mut req = self.client.post(base_url).json(&payload);

                // Auth header
                if let Some(auth_type) = connector.config.get("authType").and_then(|v| v.as_str()) {
                    match auth_type {
                        "bearer" => {
                            if let Some(token) =
                                connector.config.get("bearerToken").and_then(|v| v.as_str())
                            {
                                req = req.bearer_auth(token);
                            }
                        }
                        "basic" => {
                            let user = connector
                                .config
                                .get("basicUsername")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            let pass = connector
                                .config
                                .get("basicPassword")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            req = req.basic_auth(user, Some(pass));
                        }
                        _ => {}
                    }
                }

                let resp = req.send().await?;
                info!(
                    connector_id = %connector.id,
                    status = %resp.status(),
                    "connector HTTP response"
                );
                Ok(())
            }
            _ => {
                info!(
                    connector_id = %connector.id,
                    connector_type = ?connector.connector_type,
                    ?payload,
                    "non-HTTP connector invoke (log-only)"
                );
                Ok(())
            }
        }
    }
}

// ─── Payload builders ─────────────────────────────────────────────────────────

/// Build an OTP delivery payload for email/SMS connectors.
pub fn otp_payload(login_id: &str, code: &str, method: &str) -> Value {
    json!({
        "event": "otp_send",
        "method": method,
        "loginId": login_id,
        "code": code,
    })
}

/// Build a magic link delivery payload.
pub fn magic_link_payload(login_id: &str, token: &str, redirect_url: Option<&str>) -> Value {
    json!({
        "event": "magic_link_send",
        "loginId": login_id,
        "token": token,
        "redirectUrl": redirect_url.unwrap_or(""),
    })
}

/// Build a password reset payload.
pub fn password_reset_payload(login_id: &str, token: &str) -> Value {
    json!({
        "event": "password_reset",
        "loginId": login_id,
        "token": token,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_from_env_defaults_to_log() {
        let config = EmulatorConfig::default();
        assert_eq!(ConnectorMode::from_env(&config), ConnectorMode::Log);
    }

    #[test]
    fn otp_payload_has_required_fields() {
        let p = otp_payload("user@test.com", "123456", "email");
        assert_eq!(p["event"], "otp_send");
        assert_eq!(p["loginId"], "user@test.com");
        assert_eq!(p["code"], "123456");
    }

    #[test]
    fn magic_link_payload_has_required_fields() {
        let p = magic_link_payload("user@test.com", "tok123", Some("https://example.com"));
        assert_eq!(p["event"], "magic_link_send");
        assert_eq!(p["token"], "tok123");
    }
}
