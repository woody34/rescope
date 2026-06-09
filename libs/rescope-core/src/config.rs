use std::env;

#[derive(Debug, Clone)]
pub struct EmulatorConfig {
    pub port: u16,
    pub project_id: String,
    pub management_key: String,
    pub seed_file: Option<String>,
    pub key_file: Option<String>,
    pub session_ttl: u64,
    pub refresh_ttl: u64,
    /// Controls whether connectors actually fire HTTP requests.
    /// `None` or `"log"` = log only (default). `"invoke"` = make real HTTP calls.
    pub connector_mode: Option<String>,
}

impl EmulatorConfig {
    pub fn from_env() -> Self {
        Self {
            port: env::var("DESCOPE_EMULATOR_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4600),
            project_id: env::var("DESCOPE_PROJECT_ID")
                .unwrap_or_else(|_| "emulator-project".to_string()),
            management_key: env::var("DESCOPE_MANAGEMENT_KEY")
                .unwrap_or_else(|_| "emulator-key".to_string()),
            seed_file: env::var("DESCOPE_EMULATOR_SEED_FILE").ok(),
            key_file: env::var("DESCOPE_EMULATOR_KEY_FILE").ok(),
            session_ttl: env::var("DESCOPE_EMULATOR_SESSION_TTL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
            refresh_ttl: env::var("DESCOPE_EMULATOR_REFRESH_TTL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2_592_000),
            connector_mode: env::var("DESCOPE_EMULATOR_CONNECTOR_MODE").ok(),
        }
    }
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            port: 4600,
            project_id: "emulator-project".to_string(),
            management_key: "emulator-key".to_string(),
            seed_file: None,
            key_file: None,
            session_ttl: 3600,
            refresh_ttl: 2_592_000,
            connector_mode: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_applied() {
        let config = EmulatorConfig::from_env();
        // Only assert on values not overridden by test env
        assert_eq!(config.session_ttl, 3600);
        assert_eq!(config.refresh_ttl, 2_592_000);
    }
}
