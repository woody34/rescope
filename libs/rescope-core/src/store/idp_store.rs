use crate::error::EmulatorError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IdpProtocol {
    Oidc,
    Saml,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdpEmulator {
    pub id: String,
    pub protocol: IdpProtocol,
    pub display_name: String,
    pub tenant_id: String,
    /// Maps IdP claim name → user field path (e.g. "email" → "user.email").
    #[serde(default)]
    pub attribute_mapping: HashMap<String, String>,
}

// ─── Store ───────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct IdpStore {
    by_id: HashMap<String, IdpEmulator>,
}

impl IdpStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, mut idp: IdpEmulator) -> Result<IdpEmulator, EmulatorError> {
        if idp.id.is_empty() {
            idp.id = format!("IDP{}", Uuid::new_v4().as_simple());
        }
        self.by_id.insert(idp.id.clone(), idp.clone());
        Ok(idp)
    }

    pub fn load(&self, id: &str) -> Result<&IdpEmulator, EmulatorError> {
        self.by_id.get(id).ok_or(EmulatorError::IdpNotFound)
    }

    pub fn list(&self) -> Vec<&IdpEmulator> {
        self.by_id.values().collect()
    }

    pub fn update(
        &mut self,
        id: &str,
        display_name: Option<String>,
        attribute_mapping: Option<HashMap<String, String>>,
    ) -> Result<(), EmulatorError> {
        let idp = self.by_id.get_mut(id).ok_or(EmulatorError::IdpNotFound)?;
        if let Some(name) = display_name {
            idp.display_name = name;
        }
        if let Some(mapping) = attribute_mapping {
            idp.attribute_mapping = mapping;
        }
        Ok(())
    }

    pub fn delete(&mut self, id: &str) -> Result<(), EmulatorError> {
        self.by_id.remove(id).ok_or(EmulatorError::IdpNotFound)?;
        Ok(())
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn snapshot(&self) -> Vec<IdpEmulator> {
        self.by_id.values().cloned().collect()
    }

    pub fn restore(&mut self, idps: Vec<IdpEmulator>) {
        self.by_id.clear();
        for idp in idps {
            self.by_id.insert(idp.id.clone(), idp);
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_idp(protocol: IdpProtocol) -> IdpEmulator {
        IdpEmulator {
            id: String::new(),
            protocol,
            display_name: "Mock IdP".into(),
            tenant_id: "acme".into(),
            attribute_mapping: HashMap::from([
                ("email".into(), "user.email".into()),
                ("name".into(), "user.name".into()),
            ]),
        }
    }

    #[test]
    fn insert_generates_id() {
        let mut store = IdpStore::new();
        let idp = store.insert(test_idp(IdpProtocol::Oidc)).unwrap();
        assert!(!idp.id.is_empty());
        assert!(idp.id.starts_with("IDP"));
    }

    #[test]
    fn load_by_id() {
        let mut store = IdpStore::new();
        let idp = store.insert(test_idp(IdpProtocol::Oidc)).unwrap();
        let loaded = store.load(&idp.id).unwrap();
        assert_eq!(loaded.display_name, "Mock IdP");
        assert_eq!(loaded.tenant_id, "acme");
    }

    #[test]
    fn load_unknown_returns_not_found() {
        let store = IdpStore::new();
        assert!(matches!(
            store.load("ghost"),
            Err(EmulatorError::IdpNotFound)
        ));
    }

    #[test]
    fn list_returns_all() {
        let mut store = IdpStore::new();
        store.insert(test_idp(IdpProtocol::Oidc)).unwrap();
        let mut saml_idp = test_idp(IdpProtocol::Saml);
        saml_idp.display_name = "SAML IdP".into();
        store.insert(saml_idp).unwrap();
        assert_eq!(store.list().len(), 2);
    }

    #[test]
    fn update_display_name() {
        let mut store = IdpStore::new();
        let idp = store.insert(test_idp(IdpProtocol::Oidc)).unwrap();
        store
            .update(&idp.id, Some("Updated Name".into()), None)
            .unwrap();
        assert_eq!(store.load(&idp.id).unwrap().display_name, "Updated Name");
    }

    #[test]
    fn update_attribute_mapping() {
        let mut store = IdpStore::new();
        let idp = store.insert(test_idp(IdpProtocol::Oidc)).unwrap();
        let new_mapping = HashMap::from([(
            "department".into(),
            "user.customAttributes.department".into(),
        )]);
        store.update(&idp.id, None, Some(new_mapping)).unwrap();
        let loaded = store.load(&idp.id).unwrap();
        assert!(loaded.attribute_mapping.contains_key("department"));
        assert!(!loaded.attribute_mapping.contains_key("email")); // replaced, not merged
    }

    #[test]
    fn delete_idp() {
        let mut store = IdpStore::new();
        let idp = store.insert(test_idp(IdpProtocol::Oidc)).unwrap();
        store.delete(&idp.id).unwrap();
        assert!(matches!(
            store.load(&idp.id),
            Err(EmulatorError::IdpNotFound)
        ));
    }

    #[test]
    fn delete_unknown_returns_not_found() {
        let mut store = IdpStore::new();
        assert!(matches!(
            store.delete("ghost"),
            Err(EmulatorError::IdpNotFound)
        ));
    }

    #[test]
    fn snapshot_and_restore() {
        let mut store = IdpStore::new();
        store.insert(test_idp(IdpProtocol::Oidc)).unwrap();
        store.insert(test_idp(IdpProtocol::Saml)).unwrap();
        let snap = store.snapshot();
        assert_eq!(snap.len(), 2);

        let mut store2 = IdpStore::new();
        store2.restore(snap);
        assert_eq!(store2.list().len(), 2);
    }

    #[test]
    fn reset_clears_all() {
        let mut store = IdpStore::new();
        store.insert(test_idp(IdpProtocol::Oidc)).unwrap();
        store.reset();
        assert_eq!(store.list().len(), 0);
    }
}
