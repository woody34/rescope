/// Auth method configurations — all 13 method policy structs.
/// This store is NOT reset on `emulator/reset`; it persists across test runs just like
/// the real Descope console config would.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Per-method configs ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpConfig {
    pub enabled: bool,
    pub expiration_seconds: u64,
    pub max_retries: u32,
    pub retry_timeframe_seconds: u64,
    pub allow_unverified_recipients: bool,
    /// Connector ID for email delivery, if any.
    pub email_connector_id: Option<String>,
    pub sms_connector_id: Option<String>,
    pub voice_connector_id: Option<String>,
}

impl Default for OtpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            expiration_seconds: 180,
            max_retries: 5,
            retry_timeframe_seconds: 60,
            allow_unverified_recipients: false,
            email_connector_id: None,
            sms_connector_id: None,
            voice_connector_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MagicLinkConfig {
    pub enabled: bool,
    pub redirect_url: String,
    pub expiration_seconds: u64,
    pub max_retries: u32,
    pub retry_timeframe_seconds: u64,
    pub allow_unverified_recipients: bool,
    pub email_connector_id: Option<String>,
    pub sms_connector_id: Option<String>,
}

impl Default for MagicLinkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            redirect_url: String::new(),
            expiration_seconds: 600,
            max_retries: 5,
            retry_timeframe_seconds: 60,
            allow_unverified_recipients: false,
            email_connector_id: None,
            sms_connector_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnchantedLinkConfig {
    pub enabled: bool,
    pub redirect_url: String,
    pub expiration_seconds: u64,
    pub max_retries: u32,
    pub retry_timeframe_seconds: u64,
    pub allow_unverified_recipients: bool,
    pub email_connector_id: Option<String>,
}

impl Default for EnchantedLinkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            redirect_url: String::new(),
            expiration_seconds: 600,
            max_retries: 5,
            retry_timeframe_seconds: 60,
            allow_unverified_recipients: false,
            email_connector_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedLinkConfig {
    pub enabled: bool,
    pub expiration_seconds: u64,
}

impl Default for EmbeddedLinkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            expiration_seconds: 600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotpConfig {
    pub enabled: bool,
    pub authenticator_label: String,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            authenticator_label: "Descope".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasskeysConfig {
    pub enabled: bool,
    pub top_level_domain: String,
}

impl Default for PasskeysConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            top_level_domain: "localhost".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum OAuthTriggerMethod {
    EnableAll,
    BlockApiAndSdk,
    DisableAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthProviderConfig {
    pub trigger_method: OAuthTriggerMethod,
    pub use_descope_account: bool,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub scopes: Vec<String>,
}

impl Default for OAuthProviderConfig {
    fn default() -> Self {
        Self {
            trigger_method: OAuthTriggerMethod::DisableAll,
            use_descope_account: true,
            client_id: None,
            client_secret: None,
            scopes: vec!["openid".into(), "email".into(), "profile".into()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthConfig {
    pub providers: HashMap<String, OAuthProviderConfig>,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        let provider_names = [
            "apple",
            "discord",
            "facebook",
            "github",
            "gitlab",
            "google",
            "linkedin",
            "microsoft",
            "slack",
        ];
        let mut providers = HashMap::new();
        for name in &provider_names {
            providers.insert(name.to_string(), OAuthProviderConfig::default());
        }
        Self { providers }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SsoRoleHandling {
    AddToExisting,
    Override,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SsoConfig {
    pub enabled: bool,
    pub convert_existing_to_sso_only: bool,
    pub allow_duplicate_domains: bool,
    pub group_priority: bool,
    pub redirect_url: String,
    pub role_handling: SsoRoleHandling,
}

impl Default for SsoConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            convert_existing_to_sso_only: false,
            allow_duplicate_domains: false,
            group_priority: false,
            redirect_url: String::new(),
            role_handling: SsoRoleHandling::AddToExisting,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PasswordStrength {
    None,
    Weak,
    Medium,
    Strong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordConfig {
    pub enabled: bool,
    pub min_length: u32,
    pub require_lowercase: bool,
    pub require_uppercase: bool,
    pub require_number: bool,
    pub require_non_alphanumeric: bool,
    pub expiration_enabled: bool,
    pub expiration_days: u32,
    pub max_reuse: u32,
    pub account_lockout_enabled: bool,
    pub lockout_attempts: u32,
    pub lockout_duration_seconds: u64,
    pub strength_enforcement: PasswordStrength,
    pub reset_connector_id: Option<String>,
    pub allow_unverified_reset: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_length: 6,
            require_lowercase: false,
            require_uppercase: false,
            require_number: false,
            require_non_alphanumeric: false,
            expiration_enabled: false,
            expiration_days: 90,
            max_reuse: 0,
            account_lockout_enabled: false,
            lockout_attempts: 5,
            lockout_duration_seconds: 300,
            strength_enforcement: PasswordStrength::None,
            reset_connector_id: None,
            allow_unverified_reset: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityQuestionsConfig {
    pub questions: Vec<String>,
    pub required_for_setup: u32,
    pub required_for_verify: u32,
    pub lockout_attempts: u32,
    pub lockout_duration_seconds: u64,
}

impl Default for SecurityQuestionsConfig {
    fn default() -> Self {
        Self {
            questions: vec![
                "What was the name of your first pet?".into(),
                "What city were you born in?".into(),
                "What is your mother's maiden name?".into(),
            ],
            required_for_setup: 2,
            required_for_verify: 1,
            lockout_attempts: 5,
            lockout_duration_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryCodesConfig {
    pub code_count: u32,
    pub lockout_attempts: u32,
    pub lockout_duration_seconds: u64,
}

impl Default for RecoveryCodesConfig {
    fn default() -> Self {
        Self {
            code_count: 8,
            lockout_attempts: 5,
            lockout_duration_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceAuthConfig {
    pub enabled: bool,
    pub expiration_seconds: u64,
    pub verification_uri: String,
}

impl Default for DeviceAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            expiration_seconds: 300,
            verification_uri: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotpConfig {
    pub enabled: bool,
    pub expiration_seconds: u64,
    pub allow_unverified_recipients: bool,
    pub connector_id: Option<String>,
}

impl Default for NotpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            expiration_seconds: 180,
            allow_unverified_recipients: false,
            connector_id: None,
        }
    }
}

// ─── Root config ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethodConfig {
    pub otp: OtpConfig,
    pub magic_link: MagicLinkConfig,
    pub enchanted_link: EnchantedLinkConfig,
    pub embedded_link: EmbeddedLinkConfig,
    pub totp: TotpConfig,
    pub passkeys: PasskeysConfig,
    pub oauth: OAuthConfig,
    pub sso: SsoConfig,
    pub password: PasswordConfig,
    pub security_questions: SecurityQuestionsConfig,
    pub recovery_codes: RecoveryCodesConfig,
    pub device_auth: DeviceAuthConfig,
    pub notp: NotpConfig,
}

impl AuthMethodConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

// ─── Store wrapper ────────────────────────────────────────────────────────────

/// Thin wrapper — allows the config to live behind `Arc<RwLock<AuthMethodConfigStore>>`
/// alongside the other stores.
pub struct AuthMethodConfigStore {
    pub config: AuthMethodConfig,
}

impl Default for AuthMethodConfigStore {
    fn default() -> Self {
        Self {
            config: AuthMethodConfig::new(),
        }
    }
}

impl AuthMethodConfigStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self) -> &AuthMethodConfig {
        &self.config
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn replace(&mut self, new_config: AuthMethodConfig) {
        self.config = new_config;
    }

    /// Auth method config is NOT reset on emulator reset — only snapshot import can change it.
    /// This method exists for explicit snapshot restore.
    pub fn restore(&mut self, config: AuthMethodConfig) {
        self.config = config;
    }

    /// Check whether a given auth method is enabled.
    /// Method names: "otp", "magic_link", "enchanted_link", "embedded_link",
    /// "totp", "passkeys", "sso", "password", "security_questions",
    /// "recovery_codes", "device_auth", "notp". Returns `true` for unknown methods.
    pub fn is_enabled(&self, method: &str) -> bool {
        match method {
            "otp" => self.config.otp.enabled,
            "magic_link" => self.config.magic_link.enabled,
            "enchanted_link" => self.config.enchanted_link.enabled,
            "embedded_link" => self.config.embedded_link.enabled,
            "totp" => self.config.totp.enabled,
            "passkeys" => self.config.passkeys.enabled,
            "sso" => self.config.sso.enabled,
            "password" => self.config.password.enabled,
            "device_auth" => self.config.device_auth.enabled,
            "notp" => self.config.notp.enabled,
            _ => true, // unknown method names are allowed by default
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_descope_production_values() {
        let cfg = AuthMethodConfig::new();
        assert!(cfg.otp.enabled);
        assert_eq!(cfg.otp.expiration_seconds, 180);
        assert_eq!(cfg.otp.max_retries, 5);
        assert!(cfg.magic_link.enabled);
        assert_eq!(cfg.magic_link.expiration_seconds, 600);
        assert!(cfg.password.enabled);
        assert_eq!(cfg.password.min_length, 6);
        assert!(!cfg.device_auth.enabled, "device auth off by default");
        assert!(!cfg.notp.enabled, "nOTP off by default");
    }

    #[test]
    fn oauth_has_all_nine_providers_by_default() {
        let cfg = AuthMethodConfig::new();
        for provider in &[
            "apple",
            "discord",
            "facebook",
            "github",
            "gitlab",
            "google",
            "linkedin",
            "microsoft",
            "slack",
        ] {
            assert!(
                cfg.oauth.providers.contains_key(*provider),
                "missing provider: {provider}"
            );
        }
    }

    #[test]
    fn replace_updates_config() {
        let mut store = AuthMethodConfigStore::new();
        let mut new_cfg = AuthMethodConfig::new();
        new_cfg.otp.expiration_seconds = 60;
        store.replace(new_cfg);
        assert_eq!(store.get().otp.expiration_seconds, 60);
    }

    #[test]
    fn serialization_roundtrip() {
        let cfg = AuthMethodConfig::new();
        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: AuthMethodConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.otp.expiration_seconds, cfg.otp.expiration_seconds);
    }
}
