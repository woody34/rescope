use crate::error::EmulatorError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum length for attribute name and machine_name (matches Descope).
const MAX_ATTRIBUTE_NAME_LENGTH: usize = 60;

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AttributeType {
    Text,
    Number,
    Boolean,
    Date,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AttributePermissions {
    /// All users can read and write their own value.
    All,
    /// Only project admins can write the value.
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAttribute {
    /// Human-readable display name.
    pub name: String,
    /// Machine name used as the key in `user.custom_attributes` (camelCase, snake_case, etc.).
    pub machine_name: String,
    pub attribute_type: AttributeType,
    pub permissions: AttributePermissions,
}

// ─── Store ───────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct CustomAttributeStore {
    /// Keyed by machine_name (unique).
    by_machine_name: HashMap<String, CustomAttribute>,
}

impl CustomAttributeStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(
        &mut self,
        name: String,
        machine_name: String,
        attribute_type: AttributeType,
        permissions: AttributePermissions,
    ) -> Result<CustomAttribute, EmulatorError> {
        if name.len() > MAX_ATTRIBUTE_NAME_LENGTH {
            return Err(EmulatorError::ValidationError(format!(
                "Attribute name must be {} characters or fewer (got {})",
                MAX_ATTRIBUTE_NAME_LENGTH,
                name.len()
            )));
        }
        if machine_name.len() > MAX_ATTRIBUTE_NAME_LENGTH {
            return Err(EmulatorError::ValidationError(format!(
                "Machine name must be {} characters or fewer (got {})",
                MAX_ATTRIBUTE_NAME_LENGTH,
                machine_name.len()
            )));
        }
        if !machine_name.starts_with(|c: char| c.is_ascii_alphabetic())
            || !machine_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(EmulatorError::ValidationError(
                "Machine name must start with a letter and contain only letters, numbers, and underscores".into()
            ));
        }
        if self.by_machine_name.contains_key(&machine_name) {
            return Err(EmulatorError::CustomAttributeAlreadyExists);
        }
        let attr = CustomAttribute {
            name,
            machine_name: machine_name.clone(),
            attribute_type,
            permissions,
        };
        self.by_machine_name.insert(machine_name, attr.clone());
        Ok(attr)
    }

    pub fn load_all(&self) -> Vec<&CustomAttribute> {
        self.by_machine_name.values().collect()
    }

    pub fn load(&self, machine_name: &str) -> Result<&CustomAttribute, EmulatorError> {
        self.by_machine_name
            .get(machine_name)
            .ok_or(EmulatorError::CustomAttributeNotFound)
    }

    pub fn exists(&self, machine_name: &str) -> bool {
        self.by_machine_name.contains_key(machine_name)
    }

    pub fn delete(&mut self, machine_name: &str) -> Result<(), EmulatorError> {
        self.by_machine_name
            .remove(machine_name)
            .ok_or(EmulatorError::CustomAttributeNotFound)?;
        Ok(())
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn snapshot(&self) -> Vec<CustomAttribute> {
        self.by_machine_name.values().cloned().collect()
    }

    pub fn restore(&mut self, attrs: Vec<CustomAttribute>) {
        self.by_machine_name.clear();
        for a in attrs {
            self.by_machine_name.insert(a.machine_name.clone(), a);
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_list() {
        let mut store = CustomAttributeStore::new();
        store
            .create(
                "Plan".into(),
                "plan".into(),
                AttributeType::Text,
                AttributePermissions::All,
            )
            .unwrap();
        assert_eq!(store.load_all().len(), 1);
    }

    #[test]
    fn duplicate_machine_name_returns_error() {
        let mut store = CustomAttributeStore::new();
        store
            .create(
                "Plan".into(),
                "plan".into(),
                AttributeType::Text,
                AttributePermissions::All,
            )
            .unwrap();
        let err = store
            .create(
                "Plan2".into(),
                "plan".into(),
                AttributeType::Text,
                AttributePermissions::All,
            )
            .unwrap_err();
        assert!(matches!(err, EmulatorError::CustomAttributeAlreadyExists));
    }

    #[test]
    fn load_by_machine_name() {
        let mut store = CustomAttributeStore::new();
        store
            .create(
                "Score".into(),
                "score".into(),
                AttributeType::Number,
                AttributePermissions::Admin,
            )
            .unwrap();
        let a = store.load("score").unwrap();
        assert_eq!(a.name, "Score");
        assert!(matches!(a.attribute_type, AttributeType::Number));
    }

    #[test]
    fn delete_attribute() {
        let mut store = CustomAttributeStore::new();
        store
            .create(
                "X".into(),
                "x".into(),
                AttributeType::Boolean,
                AttributePermissions::All,
            )
            .unwrap();
        store.delete("x").unwrap();
        assert!(matches!(
            store.load("x"),
            Err(EmulatorError::CustomAttributeNotFound)
        ));
    }

    #[test]
    fn snapshot_and_restore() {
        let mut store = CustomAttributeStore::new();
        store
            .create(
                "Flag".into(),
                "flag".into(),
                AttributeType::Boolean,
                AttributePermissions::All,
            )
            .unwrap();
        let snap = store.snapshot();
        let mut store2 = CustomAttributeStore::new();
        store2.restore(snap);
        assert_eq!(store2.load_all().len(), 1);
        assert_eq!(store2.load("flag").unwrap().name, "Flag");
    }

    #[test]
    fn name_exceeding_max_length_rejected() {
        let mut store = CustomAttributeStore::new();
        let long_name = "a".repeat(61);
        let err = store
            .create(
                long_name,
                "ok".into(),
                AttributeType::Text,
                AttributePermissions::All,
            )
            .unwrap_err();
        assert!(matches!(err, EmulatorError::ValidationError(_)));
    }

    #[test]
    fn machine_name_exceeding_max_length_rejected() {
        let mut store = CustomAttributeStore::new();
        let long_key = "a".repeat(61);
        let err = store
            .create(
                "Valid".into(),
                long_key,
                AttributeType::Text,
                AttributePermissions::All,
            )
            .unwrap_err();
        assert!(matches!(err, EmulatorError::ValidationError(_)));
    }

    #[test]
    fn machine_name_invalid_pattern_rejected() {
        let mut store = CustomAttributeStore::new();
        // starts with digit
        let err = store
            .create(
                "Test".into(),
                "1invalid".into(),
                AttributeType::Text,
                AttributePermissions::All,
            )
            .unwrap_err();
        assert!(matches!(err, EmulatorError::ValidationError(_)));
        // contains special chars
        let err2 = store
            .create(
                "Test".into(),
                "invalid-name!".into(),
                AttributeType::Text,
                AttributePermissions::All,
            )
            .unwrap_err();
        assert!(matches!(err2, EmulatorError::ValidationError(_)));
    }

    #[test]
    fn machine_name_camel_case_accepted() {
        let mut store = CustomAttributeStore::new();
        store
            .create(
                "Change Password".into(),
                "changePassword".into(),
                AttributeType::Boolean,
                AttributePermissions::All,
            )
            .unwrap();
        assert_eq!(
            store.load("changePassword").unwrap().name,
            "Change Password"
        );
    }

    #[test]
    fn machine_name_at_max_length_accepted() {
        let mut store = CustomAttributeStore::new();
        let exact_key = "a".repeat(60);
        store
            .create(
                "Valid Name".into(),
                exact_key,
                AttributeType::Text,
                AttributePermissions::All,
            )
            .unwrap();
        assert_eq!(store.load_all().len(), 1);
    }
}
