use crate::error::EmulatorError;
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AccessKeyStatus {
    Active,
    Expired,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantRoleBinding {
    pub tenant_id: String,
    pub role_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessKey {
    pub id: String,
    pub name: String,
    /// Bcrypt hash of the raw key value. The raw value is never stored.
    pub key_hash: String,
    /// Unix timestamp seconds. None = never expires.
    pub expires_at: Option<u64>,
    /// CIDR ranges that are allowed to use this key. Empty = any IP.
    pub permitted_ips: Vec<String>,
    pub role_names: Vec<String>,
    pub tenant_roles: Vec<TenantRoleBinding>,
    pub status: AccessKeyStatus,
    pub created_at: u64,
    pub created_by: String,
}

// ─── Store ───────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct AccessKeyStore {
    by_id: HashMap<String, AccessKey>,
}

impl AccessKeyStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new access key. Returns (AccessKey metadata, raw_key_value).
    /// The raw key is shown only once and is NOT stored.
    pub fn create(
        &mut self,
        name: String,
        expires_at: Option<u64>,
        permitted_ips: Vec<String>,
        role_names: Vec<String>,
        tenant_roles: Vec<TenantRoleBinding>,
        created_by: String,
    ) -> Result<(AccessKey, String), EmulatorError> {
        let id = format!("K{}", Uuid::new_v4().as_simple());
        // Raw key = "<id>:<random>" — same pattern as Descope
        let raw = format!("{}:{}", id, Uuid::new_v4());
        let key_hash = hash(&raw, DEFAULT_COST)
            .map_err(|e| EmulatorError::Internal(format!("bcrypt error: {e}")))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let key = AccessKey {
            id: id.clone(),
            name,
            key_hash,
            expires_at,
            permitted_ips,
            role_names,
            tenant_roles,
            status: AccessKeyStatus::Active,
            created_at: now,
            created_by,
        };
        self.by_id.insert(id, key.clone());
        Ok((key, raw))
    }

    pub fn load_all(&self) -> Vec<&AccessKey> {
        self.by_id.values().collect()
    }

    pub fn load(&self, id: &str) -> Result<&AccessKey, EmulatorError> {
        self.by_id.get(id).ok_or(EmulatorError::AccessKeyNotFound)
    }

    /// Validate a raw key value. Returns the key entry if valid, active, and not expired.
    /// Checks IP allowlist if `caller_ip` is provided.
    pub fn validate(
        &self,
        raw_key: &str,
        caller_ip: Option<&str>,
    ) -> Result<&AccessKey, EmulatorError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for key in self.by_id.values() {
            if key.status != AccessKeyStatus::Active {
                continue;
            }
            if let Some(expires) = key.expires_at {
                if now >= expires {
                    continue;
                }
            }
            if !key.permitted_ips.is_empty() {
                if let Some(ip_str) = caller_ip {
                    if let Ok(ip) = ip_str.parse::<IpAddr>() {
                        let allowed = key.permitted_ips.iter().any(|cidr| cidr_contains(cidr, ip));
                        if !allowed {
                            continue;
                        }
                    }
                }
            }
            if verify(raw_key, &key.key_hash).unwrap_or(false) {
                return Ok(key);
            }
        }
        Err(EmulatorError::Unauthorized)
    }

    pub fn update(
        &mut self,
        id: &str,
        name: Option<String>,
        role_names: Option<Vec<String>>,
        tenant_roles: Option<Vec<TenantRoleBinding>>,
    ) -> Result<(), EmulatorError> {
        let key = self
            .by_id
            .get_mut(id)
            .ok_or(EmulatorError::AccessKeyNotFound)?;
        if let Some(n) = name {
            key.name = n;
        }
        if let Some(r) = role_names {
            key.role_names = r;
        }
        if let Some(t) = tenant_roles {
            key.tenant_roles = t;
        }
        Ok(())
    }

    pub fn disable(&mut self, id: &str) -> Result<(), EmulatorError> {
        self.by_id
            .get_mut(id)
            .ok_or(EmulatorError::AccessKeyNotFound)?
            .status = AccessKeyStatus::Disabled;
        Ok(())
    }

    pub fn delete(&mut self, id: &str) -> Result<(), EmulatorError> {
        self.by_id
            .remove(id)
            .ok_or(EmulatorError::AccessKeyNotFound)?;
        Ok(())
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn snapshot(&self) -> Vec<AccessKey> {
        self.by_id.values().cloned().collect()
    }

    pub fn restore(&mut self, keys: Vec<AccessKey>) {
        self.by_id.clear();
        for k in keys {
            self.by_id.insert(k.id.clone(), k);
        }
    }
}

// ─── CIDR helper ─────────────────────────────────────────────────────────────

fn cidr_contains(cidr: &str, ip: IpAddr) -> bool {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return cidr == ip.to_string();
    }
    let Ok(prefix_len): Result<u32, _> = parts[1].parse() else {
        return false;
    };
    let Ok(net_ip): Result<IpAddr, _> = parts[0].parse() else {
        return false;
    };
    match (net_ip, ip) {
        (IpAddr::V4(net), IpAddr::V4(addr)) => {
            if prefix_len > 32 {
                return false;
            }
            let mask = if prefix_len == 0 {
                0u32
            } else {
                !0u32 << (32 - prefix_len)
            };
            (u32::from(net) & mask) == (u32::from(addr) & mask)
        }
        (IpAddr::V6(net), IpAddr::V6(addr)) => {
            if prefix_len > 128 {
                return false;
            }
            let net_bits: u128 = u128::from(net);
            let addr_bits: u128 = u128::from(addr);
            let mask = if prefix_len == 0 {
                0u128
            } else {
                !0u128 << (128 - prefix_len)
            };
            (net_bits & mask) == (addr_bits & mask)
        }
        _ => false,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn create_key(store: &mut AccessKeyStore) -> (AccessKey, String) {
        store
            .create(
                "test-key".into(),
                None,
                vec![],
                vec![],
                vec![],
                "admin".into(),
            )
            .unwrap()
    }

    #[test]
    fn create_returns_raw_key_and_metadata() {
        let mut store = AccessKeyStore::new();
        let (key, raw) = create_key(&mut store);
        assert!(!key.id.is_empty());
        assert!(!raw.is_empty());
        assert_eq!(key.status, AccessKeyStatus::Active);
    }

    #[test]
    fn validate_with_correct_raw_key_succeeds() {
        let mut store = AccessKeyStore::new();
        let (key, raw) = create_key(&mut store);
        let validated = store.validate(&raw, None).unwrap();
        assert_eq!(validated.id, key.id);
    }

    #[test]
    fn validate_with_wrong_key_fails() {
        let mut store = AccessKeyStore::new();
        create_key(&mut store);
        assert!(matches!(
            store.validate("wrong-key", None),
            Err(EmulatorError::Unauthorized)
        ));
    }

    #[test]
    fn validate_expired_key_fails() {
        let mut store = AccessKeyStore::new();
        let id = format!("K{}", Uuid::new_v4().as_simple());
        let raw = format!("{}:{}", id, Uuid::new_v4());
        let key_hash = hash(&raw, DEFAULT_COST).unwrap();
        store.by_id.insert(
            id.clone(),
            AccessKey {
                id: id.clone(),
                name: "expired".into(),
                key_hash,
                expires_at: Some(1), // epoch 1 = already expired
                permitted_ips: vec![],
                role_names: vec![],
                tenant_roles: vec![],
                status: AccessKeyStatus::Active,
                created_at: 0,
                created_by: "test".into(),
            },
        );
        assert!(matches!(
            store.validate(&raw, None),
            Err(EmulatorError::Unauthorized)
        ));
    }

    #[test]
    fn disabled_key_is_rejected() {
        let mut store = AccessKeyStore::new();
        let (key, raw) = create_key(&mut store);
        store.disable(&key.id).unwrap();
        assert!(matches!(
            store.validate(&raw, None),
            Err(EmulatorError::Unauthorized)
        ));
    }

    #[test]
    fn delete_key() {
        let mut store = AccessKeyStore::new();
        let (key, _) = create_key(&mut store);
        store.delete(&key.id).unwrap();
        assert!(matches!(
            store.load(&key.id),
            Err(EmulatorError::AccessKeyNotFound)
        ));
    }

    #[test]
    fn cidr_v4_match() {
        assert!(cidr_contains(
            "192.168.1.0/24",
            "192.168.1.42".parse().unwrap()
        ));
        assert!(!cidr_contains(
            "192.168.1.0/24",
            "192.168.2.1".parse().unwrap()
        ));
    }

    #[test]
    fn snapshot_and_restore_preserves_hashes() {
        let mut store = AccessKeyStore::new();
        let (key, raw) = create_key(&mut store);
        let snap = store.snapshot();
        let mut store2 = AccessKeyStore::new();
        store2.restore(snap);
        // The hash should still validate against the original raw key
        let validated = store2.validate(&raw, None).unwrap();
        assert_eq!(validated.id, key.id);
    }
}
