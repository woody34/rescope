use crate::error::EmulatorError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectorType {
    GenericHttp,
    Smtp,
    Twilio,
    Sendgrid,
    AwsSes,
    Slack,
    Datadog,
    AuditWebhook,
    Recaptcha,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connector {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub connector_type: ConnectorType,
    /// Type-specific configuration fields. For GenericHttp this includes
    /// baseUrl, authType, bearerToken, basicUsername, basicPassword, etc.
    pub config: Value,
}

// ─── Store ───────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct ConnectorStore {
    by_id: HashMap<String, Connector>,
}

impl ConnectorStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(
        &mut self,
        name: String,
        connector_type: ConnectorType,
        config: Value,
    ) -> Result<Connector, EmulatorError> {
        let connector = Connector {
            id: format!("CON{}", Uuid::new_v4().as_simple()),
            name,
            connector_type,
            config,
        };
        self.by_id.insert(connector.id.clone(), connector.clone());
        Ok(connector)
    }

    pub fn load_all(&self) -> Vec<&Connector> {
        self.by_id.values().collect()
    }

    pub fn load(&self, id: &str) -> Result<&Connector, EmulatorError> {
        self.by_id.get(id).ok_or(EmulatorError::ConnectorNotFound)
    }

    pub fn update(
        &mut self,
        id: &str,
        name: Option<String>,
        config: Option<Value>,
    ) -> Result<(), EmulatorError> {
        let connector = self
            .by_id
            .get_mut(id)
            .ok_or(EmulatorError::ConnectorNotFound)?;
        if let Some(n) = name {
            connector.name = n;
        }
        if let Some(c) = config {
            connector.config = c;
        }
        Ok(())
    }

    pub fn delete(&mut self, id: &str) -> Result<(), EmulatorError> {
        self.by_id
            .remove(id)
            .ok_or(EmulatorError::ConnectorNotFound)?;
        Ok(())
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn snapshot(&self) -> Vec<Connector> {
        self.by_id.values().cloned().collect()
    }

    pub fn restore(&mut self, connectors: Vec<Connector>) {
        self.by_id.clear();
        for c in connectors {
            self.by_id.insert(c.id.clone(), c);
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn http_config() -> Value {
        json!({ "baseUrl": "https://example.com", "authType": "bearer", "bearerToken": "secret" })
    }

    #[test]
    fn create_and_list() {
        let mut store = ConnectorStore::new();
        let c = store
            .create(
                "my-webhook".into(),
                ConnectorType::GenericHttp,
                http_config(),
            )
            .unwrap();
        assert!(!c.id.is_empty());
        assert_eq!(store.load_all().len(), 1);
    }

    #[test]
    fn load_by_id() {
        let mut store = ConnectorStore::new();
        let c = store
            .create("wh".into(), ConnectorType::GenericHttp, http_config())
            .unwrap();
        let loaded = store.load(&c.id).unwrap();
        assert_eq!(loaded.name, "wh");
    }

    #[test]
    fn load_unknown_returns_not_found() {
        let store = ConnectorStore::new();
        assert!(matches!(
            store.load("ghost"),
            Err(EmulatorError::ConnectorNotFound)
        ));
    }

    #[test]
    fn update_name() {
        let mut store = ConnectorStore::new();
        let c = store
            .create("old".into(), ConnectorType::GenericHttp, http_config())
            .unwrap();
        store.update(&c.id, Some("new".into()), None).unwrap();
        assert_eq!(store.load(&c.id).unwrap().name, "new");
    }

    #[test]
    fn delete_connector() {
        let mut store = ConnectorStore::new();
        let c = store
            .create("del".into(), ConnectorType::GenericHttp, http_config())
            .unwrap();
        store.delete(&c.id).unwrap();
        assert!(matches!(
            store.load(&c.id),
            Err(EmulatorError::ConnectorNotFound)
        ));
    }

    #[test]
    fn snapshot_and_restore() {
        let mut store = ConnectorStore::new();
        store
            .create("c1".into(), ConnectorType::Slack, json!({}))
            .unwrap();
        let snap = store.snapshot();
        let mut store2 = ConnectorStore::new();
        store2.restore(snap);
        assert_eq!(store2.load_all().len(), 1);
        assert!(matches!(
            store2.load_all()[0].connector_type,
            ConnectorType::Slack
        ));
    }
}
