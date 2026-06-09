use crate::{
    config::EmulatorConfig,
    connector::invoker::{ConnectorInvoker, ConnectorMode},
    jwt::key_manager::KeyManager,
    routes::emulator::idp_oidc::OidcCodeStore,
    store::{
        access_key_store::AccessKeyStore, auth_method_config::AuthMethodConfigStore,
        connector_store::ConnectorStore, custom_attribute_store::CustomAttributeStore,
        idp_store::IdpStore, jwt_template_store::JwtTemplateStore, otp_store::OtpStore,
        permission_store::PermissionStore, revocation_store::RevocationStore,
        role_store::RoleStore, tenant_store::TenantStore, token_store::TokenStore,
        user_store::UserStore,
    },
};
use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// Per-login-id lockout tracking: maps login_id → (failure_count, first_failure_at_secs).
pub type LockoutMap = HashMap<String, (u32, u64)>;

#[derive(Clone)]
pub struct EmulatorState {
    // ── Auth stores (reset on emulator/reset) ───────────────────────────────
    pub users: Arc<RwLock<UserStore>>,
    pub tenants: Arc<RwLock<TenantStore>>,
    pub tokens: Arc<RwLock<TokenStore>>,
    pub revoked: Arc<RwLock<RevocationStore>>,
    pub otps: Arc<RwLock<OtpStore>>,
    /// Per-user revocation timestamp (userId → epoch secs).
    pub user_revocations: Arc<RwLock<HashMap<String, u64>>>,
    /// Per-login-id lockout state for OTP, password, etc.
    pub lockouts: Arc<RwLock<LockoutMap>>,

    // ── Config stores (NOT reset on emulator/reset) ─────────────────────────
    pub permissions: Arc<RwLock<PermissionStore>>,
    pub roles: Arc<RwLock<RoleStore>>,
    pub auth_method_config: Arc<RwLock<AuthMethodConfigStore>>,
    pub jwt_templates: Arc<RwLock<JwtTemplateStore>>,
    pub connectors: Arc<RwLock<ConnectorStore>>,
    pub custom_attributes: Arc<RwLock<CustomAttributeStore>>,
    pub access_keys: Arc<RwLock<AccessKeyStore>>,
    pub idp_emulators: Arc<RwLock<IdpStore>>,

    // ── Infrastructure ──────────────────────────────────────────────────────
    /// RSA key pair for JWT signing. Wrapped in RwLock so snapshot import can swap keys.
    pub keys: Arc<RwLock<Arc<KeyManager>>>,
    /// Separate RSA key pair for IdP emulator signing (OIDC id_tokens, SAML assertions).
    pub idp_keys: Arc<RwLock<Arc<KeyManager>>>,
    pub config: Arc<EmulatorConfig>,
    /// Connector invoker — shared so the underlying reqwest::Client is reused.
    pub invoker: Arc<ConnectorInvoker>,
    /// OIDC authorization codes (IdP-side).
    pub oidc_codes: Arc<RwLock<OidcCodeStore>>,
}

impl EmulatorState {
    pub async fn new(config: &EmulatorConfig) -> Result<Self> {
        let keys = if let Some(path) = &config.key_file {
            KeyManager::from_pem_file(path)?
        } else {
            KeyManager::generate()?
        };

        Ok(Self {
            // Auth stores
            users: Arc::new(RwLock::new(UserStore::new())),
            tenants: Arc::new(RwLock::new(TenantStore::new())),
            tokens: Arc::new(RwLock::new(TokenStore::new())),
            revoked: Arc::new(RwLock::new(RevocationStore::new())),
            otps: Arc::new(RwLock::new(OtpStore::new())),
            user_revocations: Arc::new(RwLock::new(HashMap::new())),
            lockouts: Arc::new(RwLock::new(HashMap::new())),
            // Config stores
            permissions: Arc::new(RwLock::new(PermissionStore::new())),
            roles: Arc::new(RwLock::new(RoleStore::new())),
            auth_method_config: Arc::new(RwLock::new(AuthMethodConfigStore::new())),
            jwt_templates: Arc::new(RwLock::new(JwtTemplateStore::new())),
            connectors: Arc::new(RwLock::new(ConnectorStore::new())),
            custom_attributes: Arc::new(RwLock::new(CustomAttributeStore::new())),
            access_keys: Arc::new(RwLock::new(AccessKeyStore::new())),
            idp_emulators: Arc::new(RwLock::new(IdpStore::new())),
            // Infrastructure
            keys: Arc::new(RwLock::new(keys)),
            idp_keys: Arc::new(RwLock::new(KeyManager::generate()?)),
            config: Arc::new(config.clone()),
            invoker: Arc::new(ConnectorInvoker::new(ConnectorMode::from_env(config))),
            oidc_codes: Arc::new(RwLock::new(OidcCodeStore::new())),
        })
    }

    /// Return a snapshot of the current active key manager (cheap Arc clone).
    pub async fn km(&self) -> Arc<KeyManager> {
        self.keys.read().await.clone()
    }

    /// Atomically replace the active key manager (used by snapshot import).
    pub fn swap_keys(&self, new_km: Arc<KeyManager>) {
        // We can't use async here. Use a blocking write.
        // This is safe: we hold a reference to self, and the RwLock is tokio's.
        // We use try_write and fall back to a blocking spawn if necessary.
        // Since this is only called from the import handler (not concurrent),
        // try_write should always succeed.
        if let Ok(mut guard) = self.keys.try_write() {
            *guard = new_km;
        } else {
            // Fallback: spin until we get the lock (shouldn't normally happen)
            let keys = self.keys.clone();
            tokio::spawn(async move {
                *keys.write().await = new_km;
            });
        }
    }

    /// Clear all runtime stores back to empty (seed data, tokens, lockouts).
    /// Also resets auth_method_config so method toggles (e.g. OTP on/off) don't
    /// bleed across tests. Other config stores (roles, permissions, access_keys,
    /// jwt_templates, connectors, custom_attributes) are intentionally preserved —
    /// they mirror Descope project config in the real console.
    pub async fn reset_stores(&self) {
        self.users.write().await.reset();
        self.tenants.write().await.reset();
        self.tokens.write().await.reset();
        self.revoked.write().await.reset();
        self.otps.write().await.reset();
        self.user_revocations.write().await.clear();
        self.lockouts.write().await.clear();
        self.auth_method_config.write().await.reset();
        self.idp_emulators.write().await.reset();
        self.oidc_codes.write().await.reset();
    }
}
