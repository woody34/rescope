use crate::error::EmulatorError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ClaimType {
    Dynamic,
    Static,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtClaim {
    /// The claim key to inject into the JWT.
    pub key: String,
    /// Dynamic = resolve from user field path; Static = literal value.
    pub claim_type: ClaimType,
    /// For Dynamic: a dot-separated field path on the User struct (e.g. "customAttributes.plan").
    /// For Static: the literal JSON value as a string (e.g. "staging").
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum AuthorizationClaimsFormat {
    /// Roles as a flat array: `"roles": ["viewer"]`
    #[default]
    Flat,
    /// Roles nested under tenant: `"tenants": {"t1": {"roles": ["viewer"]}}`
    Nested,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtTemplate {
    pub id: String,
    pub name: String,
    pub authorization_claims_format: AuthorizationClaimsFormat,
    pub custom_claims: Vec<JwtClaim>,
    /// Override the `sub` claim with a user attribute path.
    pub subject_override: Option<String>,
    /// If true, include a `jti` (JWT ID) in every token.
    pub include_jti: bool,
    /// Only one template may be active at a time.
    pub is_active: bool,
}

// ─── Built-in presets ────────────────────────────────────────────────────────

pub fn preset_basic_user() -> JwtTemplate {
    JwtTemplate {
        id: "preset-basic-user".into(),
        name: "Basic User JWT".into(),
        authorization_claims_format: AuthorizationClaimsFormat::Flat,
        custom_claims: vec![],
        subject_override: None,
        include_jti: false,
        is_active: false,
    }
}

pub fn preset_oidc() -> JwtTemplate {
    JwtTemplate {
        id: "preset-oidc".into(),
        name: "OIDC".into(),
        authorization_claims_format: AuthorizationClaimsFormat::Flat,
        custom_claims: vec![
            JwtClaim {
                key: "name".into(),
                claim_type: ClaimType::Dynamic,
                value: "name".into(),
            },
            JwtClaim {
                key: "email".into(),
                claim_type: ClaimType::Dynamic,
                value: "email".into(),
            },
            JwtClaim {
                key: "email_verified".into(),
                claim_type: ClaimType::Dynamic,
                value: "verifiedEmail".into(),
            },
        ],
        subject_override: None,
        include_jti: true,
        is_active: false,
    }
}

pub fn preset_aws() -> JwtTemplate {
    JwtTemplate {
        id: "preset-aws".into(),
        name: "AWS".into(),
        authorization_claims_format: AuthorizationClaimsFormat::Nested,
        custom_claims: vec![],
        subject_override: None,
        include_jti: false,
        is_active: false,
    }
}

pub fn preset_hasura() -> JwtTemplate {
    JwtTemplate {
        id: "preset-hasura".into(),
        name: "Hasura".into(),
        authorization_claims_format: AuthorizationClaimsFormat::Nested,
        custom_claims: vec![
            JwtClaim {
                key: "x-hasura-role".into(),
                claim_type: ClaimType::Dynamic,
                value: "roleNames".into(),
            },
            JwtClaim {
                key: "x-hasura-user-id".into(),
                claim_type: ClaimType::Dynamic,
                value: "userId".into(),
            },
        ],
        subject_override: None,
        include_jti: false,
        is_active: false,
    }
}

pub fn all_presets() -> Vec<JwtTemplate> {
    vec![
        preset_basic_user(),
        preset_oidc(),
        preset_aws(),
        preset_hasura(),
    ]
}

// ─── Claim evaluation ────────────────────────────────────────────────────────

/// Resolve a dot-path against a JSON value.
/// e.g. "customAttributes.plan" → user_json["customAttributes"]["plan"]
pub fn resolve_field_path(user_json: &Value, path: &str) -> Value {
    let mut current = user_json;
    for segment in path.split('.') {
        match current {
            Value::Object(map) => {
                // Try camelCase first, then as-is
                current = map
                    .get(segment)
                    .unwrap_or_else(|| map.get(segment).unwrap_or(&Value::Null));
                if current == &Value::Null {
                    return Value::Null;
                }
            }
            _ => return Value::Null,
        }
    }
    current.clone()
}

/// Evaluate all claims in a template against the serialized user value.
/// Returns a map of claim key → JSON value.
pub fn evaluate_template_claims(
    template: &JwtTemplate,
    user_json: &Value,
) -> HashMap<String, Value> {
    let mut result = HashMap::new();
    for claim in &template.custom_claims {
        let value = match claim.claim_type {
            ClaimType::Static => {
                // Parse as JSON if possible, otherwise treat as string
                serde_json::from_str(&claim.value).unwrap_or(Value::String(claim.value.clone()))
            }
            ClaimType::Dynamic => resolve_field_path(user_json, &claim.value),
        };
        result.insert(claim.key.clone(), value);
    }
    result
}

// ─── Store ───────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct JwtTemplateStore {
    by_id: HashMap<String, JwtTemplate>,
}

impl JwtTemplateStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, mut template: JwtTemplate) -> Result<JwtTemplate, EmulatorError> {
        // Assign a new ID if not set or if it's a preset being copied
        if template.id.is_empty() || template.id.starts_with("preset-") {
            template.id = format!("JWT{}", Uuid::new_v4().as_simple());
        }
        if self.by_id.contains_key(&template.id) {
            return Err(EmulatorError::JwtTemplateAlreadyExists);
        }
        // Deactivate others if this one is active
        if template.is_active {
            self.deactivate_all();
        }
        self.by_id.insert(template.id.clone(), template.clone());
        Ok(template)
    }

    pub fn load_all(&self) -> Vec<&JwtTemplate> {
        self.by_id.values().collect()
    }

    pub fn load(&self, id: &str) -> Result<&JwtTemplate, EmulatorError> {
        self.by_id.get(id).ok_or(EmulatorError::JwtTemplateNotFound)
    }

    pub fn active(&self) -> Option<&JwtTemplate> {
        self.by_id.values().find(|t| t.is_active)
    }

    pub fn set_active(&mut self, id: &str) -> Result<(), EmulatorError> {
        if !self.by_id.contains_key(id) {
            return Err(EmulatorError::JwtTemplateNotFound);
        }
        self.deactivate_all();
        self.by_id.get_mut(id).unwrap().is_active = true;
        Ok(())
    }

    fn deactivate_all(&mut self) {
        for t in self.by_id.values_mut() {
            t.is_active = false;
        }
    }

    pub fn update(&mut self, id: &str, updated: JwtTemplate) -> Result<(), EmulatorError> {
        if !self.by_id.contains_key(id) {
            return Err(EmulatorError::JwtTemplateNotFound);
        }
        let mut t = updated;
        t.id = id.to_string();
        if t.is_active {
            self.deactivate_all();
        }
        self.by_id.insert(id.to_string(), t);
        Ok(())
    }

    pub fn delete(&mut self, id: &str) -> Result<(), EmulatorError> {
        self.by_id
            .remove(id)
            .ok_or(EmulatorError::JwtTemplateNotFound)?;
        Ok(())
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn snapshot(&self) -> Vec<JwtTemplate> {
        self.by_id.values().cloned().collect()
    }

    pub fn restore(&mut self, templates: Vec<JwtTemplate>) {
        self.by_id.clear();
        for t in templates {
            self.by_id.insert(t.id.clone(), t);
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn basic_template(name: &str) -> JwtTemplate {
        JwtTemplate {
            id: String::new(),
            name: name.to_string(),
            authorization_claims_format: AuthorizationClaimsFormat::Flat,
            custom_claims: vec![],
            subject_override: None,
            include_jti: false,
            is_active: false,
        }
    }

    #[test]
    fn create_and_list() {
        let mut store = JwtTemplateStore::new();
        let t = store.create(basic_template("my-template")).unwrap();
        assert!(!t.id.is_empty());
        assert_eq!(store.load_all().len(), 1);
    }

    #[test]
    fn set_active_deactivates_others() {
        let mut store = JwtTemplateStore::new();
        let t1 = store.create(basic_template("t1")).unwrap();
        let t2 = store.create(basic_template("t2")).unwrap();
        store.set_active(&t1.id).unwrap();
        store.set_active(&t2.id).unwrap();
        assert!(!store.load(&t1.id).unwrap().is_active);
        assert!(store.load(&t2.id).unwrap().is_active);
    }

    #[test]
    fn active_returns_correct_template() {
        let mut store = JwtTemplateStore::new();
        let t = store.create(basic_template("active-one")).unwrap();
        store.set_active(&t.id).unwrap();
        let active = store.active().unwrap();
        assert_eq!(active.name, "active-one");
    }

    #[test]
    fn evaluate_static_claim() {
        let template = JwtTemplate {
            id: "t1".into(),
            name: "test".into(),
            authorization_claims_format: AuthorizationClaimsFormat::Flat,
            custom_claims: vec![JwtClaim {
                key: "env".into(),
                claim_type: ClaimType::Static,
                value: "staging".into(),
            }],
            subject_override: None,
            include_jti: false,
            is_active: false,
        };
        let user_json = json!({});
        let claims = evaluate_template_claims(&template, &user_json);
        assert_eq!(claims["env"], Value::String("staging".into()));
    }

    #[test]
    fn evaluate_dynamic_claim_from_custom_attribute() {
        let template = JwtTemplate {
            id: "t1".into(),
            name: "test".into(),
            authorization_claims_format: AuthorizationClaimsFormat::Flat,
            custom_claims: vec![JwtClaim {
                key: "plan".into(),
                claim_type: ClaimType::Dynamic,
                value: "customAttributes.plan".into(),
            }],
            subject_override: None,
            include_jti: false,
            is_active: false,
        };
        let user_json = json!({ "customAttributes": { "plan": "enterprise" } });
        let claims = evaluate_template_claims(&template, &user_json);
        assert_eq!(claims["plan"], Value::String("enterprise".into()));
    }

    #[test]
    fn evaluate_dynamic_claim_undefined_field_returns_null() {
        let template = JwtTemplate {
            id: "t1".into(),
            name: "test".into(),
            authorization_claims_format: AuthorizationClaimsFormat::Flat,
            custom_claims: vec![JwtClaim {
                key: "plan".into(),
                claim_type: ClaimType::Dynamic,
                value: "customAttributes.plan".into(),
            }],
            subject_override: None,
            include_jti: false,
            is_active: false,
        };
        let user_json = json!({ "customAttributes": {} });
        let claims = evaluate_template_claims(&template, &user_json);
        assert_eq!(claims["plan"], Value::Null);
    }

    #[test]
    fn all_presets_have_unique_names() {
        let presets = all_presets();
        let names: Vec<_> = presets.iter().map(|p| &p.name).collect();
        let unique: std::collections::HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len());
    }

    #[test]
    fn snapshot_and_restore() {
        let mut store = JwtTemplateStore::new();
        store.create(basic_template("t1")).unwrap();
        let snap = store.snapshot();
        let mut store2 = JwtTemplateStore::new();
        store2.restore(snap);
        assert_eq!(store2.load_all().len(), 1);
    }
}
