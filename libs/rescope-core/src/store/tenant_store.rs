use crate::{
    error::EmulatorError,
    types::{AuthType, OidcConfig, SamlConfig, Tenant},
};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Default)]
pub struct TenantStore {
    tenants: HashMap<String, Tenant>,
}

impl TenantStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, tenant: Tenant) {
        self.tenants.insert(tenant.id.clone(), tenant);
    }

    pub fn load_all(&self) -> Vec<&Tenant> {
        self.tenants.values().collect()
    }

    pub fn count(&self) -> usize {
        self.tenants.len()
    }

    pub fn load(&self, tenant_id: &str) -> Result<&Tenant, EmulatorError> {
        self.tenants
            .get(tenant_id)
            .ok_or(EmulatorError::TenantNotFound)
    }

    /// Find the first SAML/OIDC-configured tenant whose domains contain the given email's domain.
    pub fn find_by_email(&self, email: &str) -> Result<&Tenant, EmulatorError> {
        let domain = email.split('@').nth(1).unwrap_or("");
        self.tenants
            .values()
            .find(|t| {
                (t.auth_type == AuthType::Saml || t.auth_type == AuthType::Oidc)
                    && t.domains.iter().any(|d| d == domain)
            })
            .ok_or(EmulatorError::TenantNotFound)
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Serialize all tenants for export.
    pub fn snapshot(&self) -> Vec<Tenant> {
        self.tenants.values().cloned().collect()
    }

    /// Restore tenants from a snapshot (replaces all current tenants).
    pub fn restore(&mut self, tenants: Vec<Tenant>) {
        self.reset();
        for t in tenants {
            self.insert(t);
        }
    }

    /// Create a new tenant. Fails if the id already exists.
    pub fn create(
        &mut self,
        id: Option<String>,
        name: String,
        domains: Vec<String>,
    ) -> Result<String, EmulatorError> {
        let tenant_id = id.unwrap_or_else(|| format!("T{}", Uuid::new_v4().as_simple()));
        if self.tenants.contains_key(&tenant_id) {
            return Err(EmulatorError::TenantAlreadyExists);
        }
        self.tenants.insert(
            tenant_id.clone(),
            Tenant {
                id: tenant_id.clone(),
                name,
                self_provisioning_domains: domains.clone(),
                domains,
                ..Default::default()
            },
        );
        Ok(tenant_id)
    }

    /// Update name, domains, and/or SSO config. Fails if tenant not found.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        id: &str,
        name: Option<String>,
        domains: Option<Vec<String>>,
        auth_type: Option<AuthType>,
        saml_config: Option<SamlConfig>,
        oidc_config: Option<OidcConfig>,
        enforce_sso: Option<bool>,
        parent_tenant_id: Option<String>,
        session_token_ttl_override: Option<u64>,
        refresh_token_ttl_override: Option<u64>,
    ) -> Result<(), EmulatorError> {
        let tenant = self
            .tenants
            .get_mut(id)
            .ok_or(EmulatorError::TenantNotFound)?;
        if let Some(n) = name {
            tenant.name = n;
        }
        if let Some(d) = domains {
            tenant.self_provisioning_domains = d.clone();
            tenant.domains = d;
        }
        if let Some(at) = auth_type {
            tenant.auth_type = at;
        }
        if let Some(sc) = saml_config {
            tenant.saml_config = Some(sc);
        }
        if let Some(oc) = oidc_config {
            tenant.oidc_config = Some(oc);
        }
        if let Some(e) = enforce_sso {
            tenant.enforce_sso = e;
        }
        if let Some(p) = parent_tenant_id {
            tenant.parent_tenant_id = Some(p);
        }
        if let Some(s) = session_token_ttl_override {
            tenant.session_token_ttl_override = Some(s);
        }
        if let Some(r) = refresh_token_ttl_override {
            tenant.refresh_token_ttl_override = Some(r);
        }
        Ok(())
    }

    /// Delete a tenant by id. Idempotent — no error if not found.
    pub fn delete_tenant(&mut self, id: &str) {
        self.tenants.remove(id);
    }

    /// Search tenants by ids or names. Empty filters return all.
    pub fn search(&self, ids: Option<&[String]>, names: Option<&[String]>) -> Vec<&Tenant> {
        self.tenants
            .values()
            .filter(|t| {
                let id_match = ids.is_none_or(|ids| ids.contains(&t.id));
                let name_match =
                    names.is_none_or(|names| names.iter().any(|n| t.name.contains(n.as_str())));
                id_match && name_match
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Tenant;

    fn saml_tenant(id: &str, domains: &[&str]) -> Tenant {
        Tenant {
            id: id.to_string(),
            name: format!("{id} Corp"),
            domains: domains.iter().map(|s| s.to_string()).collect(),
            auth_type: AuthType::Saml,
            ..Default::default()
        }
    }

    #[test]
    fn insert_and_load_all() {
        let mut store = TenantStore::new();
        store.insert(saml_tenant("t1", &["acme.com"]));
        store.insert(saml_tenant("t2", &["widgets.io"]));
        assert_eq!(store.load_all().len(), 2);
    }

    #[test]
    fn find_by_email_returns_matching_tenant() {
        let mut store = TenantStore::new();
        store.insert(saml_tenant("acme", &["acme.com"]));
        let t = store.find_by_email("user@acme.com").unwrap();
        assert_eq!(t.id, "acme");
    }

    #[test]
    fn find_by_email_unknown_domain_fails() {
        let store = TenantStore::new();
        let err = store.find_by_email("user@unknown.com").unwrap_err();
        assert!(matches!(err, EmulatorError::TenantNotFound));
    }

    #[test]
    fn load_by_id_works() {
        let mut store = TenantStore::new();
        store.insert(saml_tenant("corp", &["corp.com"]));
        let t = store.load("corp").unwrap();
        assert_eq!(t.name, "corp Corp");
    }

    #[test]
    fn none_auth_type_excluded_from_email_lookup() {
        let mut store = TenantStore::new();
        let mut t = saml_tenant("plain", &["plain.com"]);
        t.auth_type = AuthType::None;
        store.insert(t);
        let err = store.find_by_email("user@plain.com").unwrap_err();
        assert!(matches!(err, EmulatorError::TenantNotFound));
    }

    #[test]
    fn reset_clears_tenants() {
        let mut store = TenantStore::new();
        store.insert(saml_tenant("x", &["x.com"]));
        store.reset();
        assert_eq!(store.load_all().len(), 0);
    }

    // ── Tests for TenantStore CRUD (tasks 1.7-1.8) ──────────────────────────

    #[test]
    fn create_tenant_with_explicit_id() {
        let mut store = TenantStore::new();
        let id = store
            .create(Some("t1".into()), "Acme".into(), vec![])
            .unwrap();
        assert_eq!(id, "t1");
        let t = store.load("t1").unwrap();
        assert_eq!(t.name, "Acme");
    }

    #[test]
    fn create_tenant_auto_id() {
        let mut store = TenantStore::new();
        let id = store.create(None, "AutoCorp".into(), vec![]).unwrap();
        assert!(!id.is_empty());
        store.load(&id).unwrap();
    }

    #[test]
    fn create_duplicate_tenant_returns_conflict() {
        let mut store = TenantStore::new();
        store
            .create(Some("dup".into()), "Dup".into(), vec![])
            .unwrap();
        let err = store
            .create(Some("dup".into()), "Dup2".into(), vec![])
            .unwrap_err();
        assert!(matches!(err, EmulatorError::TenantAlreadyExists));
    }

    #[test]
    fn update_tenant_name() {
        let mut store = TenantStore::new();
        store
            .create(Some("t2".into()), "Original".into(), vec![])
            .unwrap();
        store
            .update(
                "t2",
                Some("Updated".into()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        assert_eq!(store.load("t2").unwrap().name, "Updated");
    }

    #[test]
    fn update_unknown_tenant_returns_not_found() {
        let mut store = TenantStore::new();
        let err = store
            .update(
                "missing",
                Some("X".into()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap_err();
        assert!(matches!(err, EmulatorError::TenantNotFound));
    }

    #[test]
    fn delete_tenant_removes_it() {
        let mut store = TenantStore::new();
        store
            .create(Some("del".into()), "ToDelete".into(), vec![])
            .unwrap();
        store.delete_tenant("del");
        assert!(matches!(
            store.load("del"),
            Err(EmulatorError::TenantNotFound)
        ));
    }

    #[test]
    fn delete_nonexistent_tenant_is_idempotent() {
        let mut store = TenantStore::new();
        store.delete_tenant("ghost"); // should not panic
    }

    #[test]
    fn search_by_id_returns_matching() {
        let mut store = TenantStore::new();
        store
            .create(Some("a".into()), "Alpha".into(), vec![])
            .unwrap();
        store
            .create(Some("b".into()), "Beta".into(), vec![])
            .unwrap();
        let results = store.search(Some(&["a".to_string()]), None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "a");
    }

    #[test]
    fn search_by_name_substring() {
        let mut store = TenantStore::new();
        store
            .create(Some("c".into()), "Corp Alpha".into(), vec![])
            .unwrap();
        store
            .create(Some("d".into()), "Corp Beta".into(), vec![])
            .unwrap();
        let results = store.search(None, Some(&["Alpha".to_string()]));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "c");
    }

    #[test]
    fn search_empty_filters_returns_all() {
        let mut store = TenantStore::new();
        store
            .create(Some("e".into()), "E Corp".into(), vec![])
            .unwrap();
        store
            .create(Some("f".into()), "F Corp".into(), vec![])
            .unwrap();
        let results = store.search(None, None);
        assert_eq!(results.len(), 2);
    }
}
