use crate::error::EmulatorError;
use std::collections::HashMap;
use uuid::Uuid;

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Names of permissions this role grants.
    pub permission_names: Vec<String>,
    /// Auto-assign this role to new users created without explicit roles.
    pub is_default: bool,
    /// Hidden from tenant admins in the console.
    pub is_hidden: bool,
}

// ─── Store ───────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct RoleStore {
    /// Keyed by role name (unique).
    by_name: HashMap<String, Role>,
}

impl RoleStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(
        &mut self,
        name: String,
        description: String,
        permissions: Vec<String>,
    ) -> Result<Role, EmulatorError> {
        if self.by_name.contains_key(&name) {
            return Err(EmulatorError::RoleAlreadyExists);
        }
        let role = Role {
            id: format!("ROL{}", Uuid::new_v4().as_simple()),
            name: name.clone(),
            description,
            permission_names: permissions,
            is_default: false,
            is_hidden: false,
        };
        self.by_name.insert(name, role.clone());
        Ok(role)
    }

    pub fn load_all(&self) -> Vec<&Role> {
        self.by_name.values().collect()
    }

    pub fn load(&self, name: &str) -> Result<&Role, EmulatorError> {
        self.by_name.get(name).ok_or(EmulatorError::RoleNotFound)
    }

    pub fn exists(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    /// Returns the names of all roles marked as default.
    pub fn default_role_names(&self) -> Vec<String> {
        self.by_name
            .values()
            .filter(|r| r.is_default)
            .map(|r| r.name.clone())
            .collect()
    }

    pub fn update(
        &mut self,
        name: &str,
        new_name: Option<String>,
        description: Option<String>,
        permissions: Option<Vec<String>>,
        is_default: Option<bool>,
        is_hidden: Option<bool>,
    ) -> Result<(), EmulatorError> {
        let mut role = self
            .by_name
            .remove(name)
            .ok_or(EmulatorError::RoleNotFound)?;

        if let Some(d) = description {
            role.description = d;
        }
        if let Some(p) = permissions {
            role.permission_names = p;
        }
        if let Some(v) = is_default {
            role.is_default = v;
        }
        if let Some(v) = is_hidden {
            role.is_hidden = v;
        }

        let key = if let Some(n) = new_name {
            if self.by_name.contains_key(&n) {
                self.by_name.insert(name.to_string(), role);
                return Err(EmulatorError::RoleAlreadyExists);
            }
            role.name = n.clone();
            n
        } else {
            name.to_string()
        };

        self.by_name.insert(key, role);
        Ok(())
    }

    pub fn set_default(&mut self, name: &str, is_default: bool) -> Result<(), EmulatorError> {
        self.by_name
            .get_mut(name)
            .ok_or(EmulatorError::RoleNotFound)?
            .is_default = is_default;
        Ok(())
    }

    pub fn delete(&mut self, name: &str) -> Result<(), EmulatorError> {
        self.by_name
            .remove(name)
            .ok_or(EmulatorError::RoleNotFound)?;
        Ok(())
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Snapshot: return all roles (owned).
    pub fn snapshot(&self) -> Vec<Role> {
        self.by_name.values().cloned().collect()
    }

    /// Restore from snapshot.
    pub fn restore(&mut self, roles: Vec<Role>) {
        self.by_name.clear();
        for r in roles {
            self.by_name.insert(r.name.clone(), r);
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_list() {
        let mut store = RoleStore::new();
        let role = store
            .create("viewer".into(), "Read only".into(), vec!["read".into()])
            .unwrap();
        assert_eq!(role.name, "viewer");
        assert_eq!(store.load_all().len(), 1);
    }

    #[test]
    fn duplicate_name_returns_error() {
        let mut store = RoleStore::new();
        store.create("admin".into(), "".into(), vec![]).unwrap();
        let err = store.create("admin".into(), "".into(), vec![]).unwrap_err();
        assert!(matches!(err, EmulatorError::RoleAlreadyExists));
    }

    #[test]
    fn load_by_name() {
        let mut store = RoleStore::new();
        store
            .create("editor".into(), "Can edit".into(), vec![])
            .unwrap();
        let r = store.load("editor").unwrap();
        assert_eq!(r.description, "Can edit");
    }

    #[test]
    fn load_unknown_returns_not_found() {
        let store = RoleStore::new();
        assert!(matches!(
            store.load("ghost"),
            Err(EmulatorError::RoleNotFound)
        ));
    }

    #[test]
    fn default_role_names_returns_correct_set() {
        let mut store = RoleStore::new();
        store.create("r1".into(), "".into(), vec![]).unwrap();
        store.create("r2".into(), "".into(), vec![]).unwrap();
        store.set_default("r1", true).unwrap();
        let defaults = store.default_role_names();
        assert_eq!(defaults, vec!["r1"]);
    }

    #[test]
    fn update_permissions() {
        let mut store = RoleStore::new();
        store
            .create("r".into(), "".into(), vec!["old".into()])
            .unwrap();
        store
            .update("r", None, None, Some(vec!["new".into()]), None, None)
            .unwrap();
        assert_eq!(store.load("r").unwrap().permission_names, vec!["new"]);
    }

    #[test]
    fn update_name() {
        let mut store = RoleStore::new();
        store.create("old".into(), "".into(), vec![]).unwrap();
        store
            .update("old", Some("new".into()), None, None, None, None)
            .unwrap();
        assert!(store.load("new").is_ok());
        assert!(matches!(
            store.load("old"),
            Err(EmulatorError::RoleNotFound)
        ));
    }

    #[test]
    fn delete_role() {
        let mut store = RoleStore::new();
        store.create("del".into(), "".into(), vec![]).unwrap();
        store.delete("del").unwrap();
        assert!(matches!(
            store.load("del"),
            Err(EmulatorError::RoleNotFound)
        ));
    }

    #[test]
    fn reset_clears_store() {
        let mut store = RoleStore::new();
        store.create("r".into(), "".into(), vec![]).unwrap();
        store.reset();
        assert_eq!(store.load_all().len(), 0);
    }

    #[test]
    fn snapshot_and_restore() {
        let mut store = RoleStore::new();
        store
            .create("viewer".into(), "ro".into(), vec!["read".into()])
            .unwrap();
        let snap = store.snapshot();
        let mut store2 = RoleStore::new();
        store2.restore(snap);
        let r = store2.load("viewer").unwrap();
        assert_eq!(r.permission_names, vec!["read"]);
    }
}
