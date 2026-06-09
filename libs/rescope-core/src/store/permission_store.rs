use crate::error::EmulatorError;
use std::collections::HashMap;
use uuid::Uuid;

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub description: String,
}

// ─── Store ───────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct PermissionStore {
    /// Keyed by permission name (unique).
    by_name: HashMap<String, Permission>,
}

impl PermissionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(
        &mut self,
        name: String,
        description: String,
    ) -> Result<Permission, EmulatorError> {
        if self.by_name.contains_key(&name) {
            return Err(EmulatorError::PermissionAlreadyExists);
        }
        let perm = Permission {
            id: format!("PERM{}", Uuid::new_v4().as_simple()),
            name: name.clone(),
            description,
        };
        self.by_name.insert(name, perm.clone());
        Ok(perm)
    }

    pub fn load_all(&self) -> Vec<&Permission> {
        self.by_name.values().collect()
    }

    pub fn load(&self, name: &str) -> Result<&Permission, EmulatorError> {
        self.by_name
            .get(name)
            .ok_or(EmulatorError::PermissionNotFound)
    }

    pub fn exists(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    pub fn update(
        &mut self,
        name: &str,
        new_name: Option<String>,
        description: Option<String>,
    ) -> Result<(), EmulatorError> {
        // Remove old entry
        let mut perm = self
            .by_name
            .remove(name)
            .ok_or(EmulatorError::PermissionNotFound)?;

        if let Some(d) = description {
            perm.description = d;
        }
        let key = if let Some(n) = new_name {
            if self.by_name.contains_key(&n) {
                // Restore old on conflict
                self.by_name.insert(name.to_string(), perm);
                return Err(EmulatorError::PermissionAlreadyExists);
            }
            perm.name = n.clone();
            n
        } else {
            name.to_string()
        };

        self.by_name.insert(key, perm);
        Ok(())
    }

    pub fn delete(&mut self, name: &str) -> Result<(), EmulatorError> {
        self.by_name
            .remove(name)
            .ok_or(EmulatorError::PermissionNotFound)?;
        Ok(())
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Snapshot: return all permissions (owned).
    pub fn snapshot(&self) -> Vec<Permission> {
        self.by_name.values().cloned().collect()
    }

    /// Restore from snapshot.
    pub fn restore(&mut self, permissions: Vec<Permission>) {
        self.by_name.clear();
        for p in permissions {
            self.by_name.insert(p.name.clone(), p);
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_list() {
        let mut store = PermissionStore::new();
        let p = store
            .create("read:reports".into(), "Can view reports".into())
            .unwrap();
        assert_eq!(p.name, "read:reports");
        assert_eq!(store.load_all().len(), 1);
    }

    #[test]
    fn duplicate_name_returns_error() {
        let mut store = PermissionStore::new();
        store.create("admin".into(), "".into()).unwrap();
        let err = store.create("admin".into(), "".into()).unwrap_err();
        assert!(matches!(err, EmulatorError::PermissionAlreadyExists));
    }

    #[test]
    fn load_by_name() {
        let mut store = PermissionStore::new();
        store
            .create("write:users".into(), "Manage users".into())
            .unwrap();
        let p = store.load("write:users").unwrap();
        assert_eq!(p.description, "Manage users");
    }

    #[test]
    fn load_unknown_returns_not_found() {
        let store = PermissionStore::new();
        assert!(matches!(
            store.load("ghost"),
            Err(EmulatorError::PermissionNotFound)
        ));
    }

    #[test]
    fn update_description() {
        let mut store = PermissionStore::new();
        store.create("p1".into(), "old desc".into()).unwrap();
        store.update("p1", None, Some("new desc".into())).unwrap();
        assert_eq!(store.load("p1").unwrap().description, "new desc");
    }

    #[test]
    fn update_name() {
        let mut store = PermissionStore::new();
        store.create("old-name".into(), "".into()).unwrap();
        store
            .update("old-name", Some("new-name".into()), None)
            .unwrap();
        assert!(store.load("new-name").is_ok());
        assert!(matches!(
            store.load("old-name"),
            Err(EmulatorError::PermissionNotFound)
        ));
    }

    #[test]
    fn delete_permission() {
        let mut store = PermissionStore::new();
        store.create("del-me".into(), "".into()).unwrap();
        store.delete("del-me").unwrap();
        assert!(matches!(
            store.load("del-me"),
            Err(EmulatorError::PermissionNotFound)
        ));
    }

    #[test]
    fn delete_unknown_returns_not_found() {
        let mut store = PermissionStore::new();
        assert!(matches!(
            store.delete("ghost"),
            Err(EmulatorError::PermissionNotFound)
        ));
    }

    #[test]
    fn reset_clears_store() {
        let mut store = PermissionStore::new();
        store.create("p".into(), "".into()).unwrap();
        store.reset();
        assert_eq!(store.load_all().len(), 0);
    }

    #[test]
    fn snapshot_and_restore() {
        let mut store = PermissionStore::new();
        store.create("x".into(), "desc".into()).unwrap();
        let snap = store.snapshot();
        let mut store2 = PermissionStore::new();
        store2.restore(snap);
        assert_eq!(store2.load("x").unwrap().description, "desc");
    }
}
