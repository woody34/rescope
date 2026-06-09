use crate::{
    error::EmulatorError,
    types::{User, UserTenant},
};
use std::collections::HashMap;
use uuid::Uuid;

const MAX_TOKENS: usize = 10_000;

/// In-memory user store with four O(1) indices.
#[derive(Default)]
pub struct UserStore {
    /// Primary storage: userId → User
    by_user_id: HashMap<String, User>,
    /// loginId (any in loginIds) → userId
    by_login_id: HashMap<String, String>,
    /// email → userId
    by_email: HashMap<String, String>,
    /// phone → userId
    by_phone: HashMap<String, String>,
}

impl UserStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a user into all indices. Fails if any loginId already exists.
    pub fn insert(&mut self, user: User) -> Result<(), EmulatorError> {
        for lid in &user.login_ids {
            if self.by_login_id.contains_key(lid.as_str()) {
                return Err(EmulatorError::UserAlreadyExists);
            }
        }
        let uid = user.user_id.clone();
        if let Some(email) = &user.email {
            if !email.is_empty() {
                self.by_email.insert(email.clone(), uid.clone());
            }
        }
        if let Some(phone) = &user.phone {
            if !phone.is_empty() {
                self.by_phone.insert(phone.clone(), uid.clone());
            }
        }
        for lid in &user.login_ids {
            self.by_login_id.insert(lid.clone(), uid.clone());
        }
        self.by_user_id.insert(uid, user);
        Ok(())
    }

    pub fn load(&self, login_id: &str) -> Result<&User, EmulatorError> {
        // Exact loginId match
        if let Some(uid) = self.by_login_id.get(login_id) {
            return self.by_user_id.get(uid).ok_or(EmulatorError::UserNotFound);
        }
        // Fallback: match by email (case-insensitive)
        let lower = login_id.to_lowercase();
        if let Some(uid) = self.by_email.get(&lower) {
            return self.by_user_id.get(uid).ok_or(EmulatorError::UserNotFound);
        }
        // Fallback: bare username → find loginId starting with "{login_id}@"
        // Supports e2e patterns like "alice" → "alice@example.com"
        let prefix = format!("{}@", lower);
        for (lid, uid) in &self.by_login_id {
            if lid.to_lowercase().starts_with(&prefix) {
                return self.by_user_id.get(uid).ok_or(EmulatorError::UserNotFound);
            }
        }
        Err(EmulatorError::UserNotFound)
    }

    pub fn load_mut(&mut self, login_id: &str) -> Result<&mut User, EmulatorError> {
        let uid = self.resolve_user_id(login_id)?;
        self.by_user_id
            .get_mut(&uid)
            .ok_or(EmulatorError::UserNotFound)
    }

    /// Resolve a login_id to a user_id using the same fallback chain as load().
    fn resolve_user_id(&self, login_id: &str) -> Result<String, EmulatorError> {
        if let Some(uid) = self.by_login_id.get(login_id) {
            return Ok(uid.clone());
        }
        let lower = login_id.to_lowercase();
        if let Some(uid) = self.by_email.get(&lower) {
            return Ok(uid.clone());
        }
        let prefix = format!("{}@", lower);
        for (lid, uid) in &self.by_login_id {
            if lid.to_lowercase().starts_with(&prefix) {
                return Ok(uid.clone());
            }
        }
        Err(EmulatorError::UserNotFound)
    }

    pub fn load_by_user_id(&self, user_id: &str) -> Result<&User, EmulatorError> {
        self.by_user_id
            .get(user_id)
            .ok_or(EmulatorError::UserNotFound)
    }

    pub fn load_by_user_id_mut(&mut self, user_id: &str) -> Result<&mut User, EmulatorError> {
        self.by_user_id
            .get_mut(user_id)
            .ok_or(EmulatorError::UserNotFound)
    }

    /// Record a login event for the user, updating `last_login` to now.
    pub fn record_login(&mut self, login_id: &str) -> Result<(), EmulatorError> {
        let user = self.load_mut(login_id)?;
        user.last_login = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        Ok(())
    }

    /// Record a login event by user ID (for handlers that resolve via token → user_id).
    pub fn record_login_by_user_id(&mut self, user_id: &str) -> Result<(), EmulatorError> {
        let user = self.load_by_user_id_mut(user_id)?;
        user.last_login = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        Ok(())
    }

    /// Full replace of all mutable fields.
    pub fn update(&mut self, login_id: &str, update: UserUpdate) -> Result<&User, EmulatorError> {
        let user = self.load_mut(login_id)?;
        user.email = update.email.clone();
        user.phone = update.phone.clone();
        user.name = update.name;
        user.given_name = update.given_name;
        user.middle_name = update.middle_name;
        user.family_name = update.family_name;
        user.picture = update.picture.or(Some(String::new()));
        user.verified_email = update.verified_email.unwrap_or(false);
        user.verified_phone = update.verified_phone.unwrap_or(false);
        user.role_names = update.role_names.unwrap_or_default();
        if let Some(attrs) = update.custom_attributes {
            user.custom_attributes = attrs;
        }
        if let Some(tenants) = update.user_tenants {
            user.user_tenants = tenants;
        }
        // rebuild email/phone indices
        let uid = user.user_id.clone();
        if let Some(email) = &update.email {
            if !email.is_empty() {
                self.by_email.insert(email.clone(), uid.clone());
            }
        }
        if let Some(phone) = &update.phone {
            if !phone.is_empty() {
                self.by_phone.insert(phone.clone(), uid.clone());
            }
        }
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Partial merge — only provided fields are changed.
    pub fn patch(&mut self, login_id: &str, patch: UserPatch) -> Result<&User, EmulatorError> {
        let user = self.load_mut(login_id)?;
        if let Some(v) = patch.email {
            user.email = Some(v);
        }
        if let Some(v) = patch.phone {
            user.phone = Some(v);
        }
        if let Some(v) = patch.name {
            user.name = Some(v);
        }
        if let Some(v) = patch.given_name {
            user.given_name = Some(v);
        }
        if let Some(v) = patch.middle_name {
            user.middle_name = Some(v);
        }
        if let Some(v) = patch.family_name {
            user.family_name = Some(v);
        }
        if let Some(v) = patch.picture {
            user.picture = Some(v);
        }
        if let Some(v) = patch.verified_email {
            user.verified_email = v;
        }
        if let Some(v) = patch.verified_phone {
            user.verified_phone = v;
        }
        if let Some(v) = patch.role_names {
            user.role_names = v;
        }
        if let Some(v) = patch.custom_attributes {
            user.custom_attributes = v;
        }
        if let Some(v) = patch.user_tenants {
            user.user_tenants = v;
        }
        let uid = user.user_id.clone();
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Store a bcrypt hash as the active password.
    pub fn set_password(&mut self, login_id: &str, hash: String) -> Result<(), EmulatorError> {
        let user = self.load_mut(login_id)?;
        user._password_hash = Some(hash);
        user.password = true;
        Ok(())
    }

    /// Verify the stored bcrypt hash. Returns Ok(()) on match.
    pub fn check_password(&self, login_id: &str, plain: &str) -> Result<(), EmulatorError> {
        let user = self.load(login_id)?;
        let hash = user
            ._password_hash
            .as_deref()
            .ok_or(EmulatorError::InvalidCredentials)?;
        bcrypt::verify(plain, hash)
            .map_err(|_| EmulatorError::InvalidCredentials)?
            .then_some(())
            .ok_or(EmulatorError::InvalidCredentials)
    }

    pub fn delete_by_login_id(&mut self, login_id: &str) {
        let uid = match self.by_login_id.get(login_id).cloned() {
            Some(u) => u,
            None => return,
        };
        self.remove_by_uid(&uid);
    }

    pub fn delete_by_user_id(&mut self, user_id: &str) {
        self.remove_by_uid(user_id);
    }

    pub fn delete_all_test_users(&mut self) {
        let test_uids: Vec<String> = self
            .by_user_id
            .values()
            .filter(|u| u._is_test_user)
            .map(|u| u.user_id.clone())
            .collect();
        for uid in test_uids {
            self.remove_by_uid(&uid);
        }
    }

    fn remove_by_uid(&mut self, uid: &str) {
        if let Some(user) = self.by_user_id.remove(uid) {
            for lid in &user.login_ids {
                self.by_login_id.remove(lid.as_str());
            }
            if let Some(email) = &user.email {
                self.by_email.remove(email.as_str());
            }
            if let Some(phone) = &user.phone {
                self.by_phone.remove(phone.as_str());
            }
        }
    }

    pub fn add_tenant(&mut self, login_id: &str, tenant: UserTenant) -> Result<(), EmulatorError> {
        let user = self.load_mut(login_id)?;
        if !user
            .user_tenants
            .iter()
            .any(|t| t.tenant_id == tenant.tenant_id)
        {
            user.user_tenants.push(tenant);
        }
        Ok(())
    }

    /// Remove a user from a tenant. Idempotent — no-op if not in the tenant.
    pub fn remove_tenant(
        &mut self,
        login_id: &str,
        tenant_id: &str,
    ) -> Result<&User, EmulatorError> {
        let user = self.load_mut(login_id)?;
        user.user_tenants.retain(|t| t.tenant_id != tenant_id);
        let uid = user.user_id.clone();
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Replace role names on a user's tenant entry. Returns 404 if user is not in that tenant.
    pub fn set_tenant_roles(
        &mut self,
        login_id: &str,
        tenant_id: &str,
        role_names: Vec<String>,
    ) -> Result<&User, EmulatorError> {
        let user = self.load_mut(login_id)?;
        let tenant = user
            .user_tenants
            .iter_mut()
            .find(|t| t.tenant_id == tenant_id)
            .ok_or(EmulatorError::TenantNotFound)?;
        tenant.role_names = role_names;
        let uid = user.user_id.clone();
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Disable a user — blocks all auth flows.
    pub fn disable(&mut self, login_id: &str) -> Result<&User, EmulatorError> {
        let user = self.load_mut(login_id)?;
        user.status = "disabled".to_string();
        let uid = user.user_id.clone();
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Enable a previously disabled user.
    pub fn enable(&mut self, login_id: &str) -> Result<&User, EmulatorError> {
        let user = self.load_mut(login_id)?;
        user.status = "enabled".to_string();
        let uid = user.user_id.clone();
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Set status to any valid value (enabled, disabled, invited).
    pub fn set_status(&mut self, login_id: &str, status: &str) -> Result<&User, EmulatorError> {
        let user = self.load_mut(login_id)?;
        user.status = status.to_string();
        let uid = user.user_id.clone();
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Search with optional filters — all filters AND-composed, within-list OR.
    pub fn search(&self, query: &SearchQuery) -> Vec<&User> {
        let mut results: Vec<&User> = self
            .by_user_id
            .values()
            .filter(|u| {
                // A targeted identifier lookup (by email / loginId / phone) must
                // surface test users even without with_test_user: real Descope
                // resolves seeded test users this way, and a backend admin-bootstrap
                // existence check (query by email) depends on it. Broad listings
                // still hide test users unless with_test_user is set.
                let targeted_lookup = query.emails.as_ref().is_some_and(|e| !e.is_empty())
                    || query.login_ids.as_ref().is_some_and(|l| !l.is_empty())
                    || query.phones.as_ref().is_some_and(|p| !p.is_empty());
                if !query.with_test_user && !targeted_lookup && u._is_test_user {
                    return false;
                }
                if let Some(emails) = &query.emails {
                    if !emails.is_empty()
                        && !u.email.as_deref().is_some_and(|e| {
                            emails.iter().any(|f| f.to_lowercase() == e.to_lowercase())
                        })
                    {
                        return false;
                    }
                }
                if let Some(phones) = &query.phones {
                    if !phones.is_empty()
                        && !u
                            .phone
                            .as_deref()
                            .is_some_and(|p| phones.iter().any(|f| f == p))
                    {
                        return false;
                    }
                }
                if let Some(attrs) = &query.custom_attributes {
                    for (k, v) in attrs {
                        if u.custom_attributes.get(k) != Some(v) {
                            return false;
                        }
                    }
                }
                // ── New filters ──
                if let Some(login_ids) = &query.login_ids {
                    if !login_ids.is_empty()
                        && !u.login_ids.iter().any(|lid| login_ids.contains(lid))
                    {
                        return false;
                    }
                }
                if let Some(statuses) = &query.statuses {
                    if !statuses.is_empty() && !statuses.contains(&u.status) {
                        return false;
                    }
                }
                if let Some(tenant_ids) = &query.tenant_ids {
                    if !tenant_ids.is_empty()
                        && !u
                            .user_tenants
                            .iter()
                            .any(|t| tenant_ids.contains(&t.tenant_id))
                    {
                        return false;
                    }
                }
                if let Some(role_names) = &query.role_names {
                    if !role_names.is_empty() {
                        let has_project_role = u.role_names.iter().any(|r| role_names.contains(r));
                        let has_tenant_role = u
                            .user_tenants
                            .iter()
                            .any(|t| t.role_names.iter().any(|r| role_names.contains(r)));
                        if !has_project_role && !has_tenant_role {
                            return false;
                        }
                    }
                }
                if let Some(text) = &query.text {
                    if !text.is_empty() {
                        let t = text.to_lowercase();
                        let matches_login = u
                            .login_ids
                            .iter()
                            .any(|lid| lid.to_lowercase().contains(&t));
                        let matches_name = u
                            .name
                            .as_deref()
                            .is_some_and(|n| n.to_lowercase().contains(&t));
                        let matches_email = u
                            .email
                            .as_deref()
                            .is_some_and(|e| e.to_lowercase().contains(&t));
                        let matches_phone = u
                            .phone
                            .as_deref()
                            .is_some_and(|p| p.to_lowercase().contains(&t));
                        if !matches_login && !matches_name && !matches_email && !matches_phone {
                            return false;
                        }
                    }
                }
                if let Some(after) = query.created_after {
                    if u.created_time < after {
                        return false;
                    }
                }
                if let Some(before) = query.created_before {
                    if u.created_time > before {
                        return false;
                    }
                }
                true
            })
            .collect();

        // ── Sort ──
        if let Some(sort) = &query.sort {
            results.sort_by(|a, b| {
                let cmp = match sort.field.as_str() {
                    "name" | "displayName" => {
                        let a_val = a.name.as_deref().unwrap_or("");
                        let b_val = b.name.as_deref().unwrap_or("");
                        a_val.to_lowercase().cmp(&b_val.to_lowercase())
                    }
                    "email" => {
                        let a_val = a.email.as_deref().unwrap_or("");
                        let b_val = b.email.as_deref().unwrap_or("");
                        a_val.to_lowercase().cmp(&b_val.to_lowercase())
                    }
                    "phone" => {
                        let a_val = a.phone.as_deref().unwrap_or("");
                        let b_val = b.phone.as_deref().unwrap_or("");
                        a_val.cmp(b_val)
                    }
                    "status" => a.status.cmp(&b.status),
                    "loginId" => {
                        let a_val = a.login_ids.first().map(|s| s.as_str()).unwrap_or("");
                        let b_val = b.login_ids.first().map(|s| s.as_str()).unwrap_or("");
                        a_val.to_lowercase().cmp(&b_val.to_lowercase())
                    }
                    // default: createdTime
                    _ => a.created_time.cmp(&b.created_time),
                };
                if sort.desc {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
        } else {
            // Default sort: createdTime descending
            results.sort_by_key(|u| std::cmp::Reverse(u.created_time));
        }

        results
    }

    pub fn all_users(&self) -> Vec<&User> {
        self.by_user_id.values().collect()
    }

    pub fn count(&self) -> usize {
        self.by_user_id.len()
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Serialize all users for export.
    pub fn snapshot(&self) -> Vec<User> {
        self.by_user_id.values().cloned().collect()
    }

    /// Restore users from a snapshot (replaces all current users).
    pub fn restore(&mut self, users: Vec<User>) {
        self.reset();
        for user in users {
            // Best-effort; skip duplicates
            let _ = self.insert(user);
        }
    }

    /// Rename a loginId. Fails if `new_login_id` is already taken or `old_login_id` doesn't exist.
    pub fn update_login_id(&mut self, old: &str, new: &str) -> Result<&User, EmulatorError> {
        if self.by_login_id.contains_key(new) {
            return Err(EmulatorError::UserAlreadyExists);
        }
        let uid = self
            .by_login_id
            .remove(old)
            .ok_or(EmulatorError::UserNotFound)?;
        self.by_login_id.insert(new.to_string(), uid.clone());
        let user = self
            .by_user_id
            .get_mut(&uid)
            .ok_or(EmulatorError::UserNotFound)?;
        user.login_ids.retain(|l| l != old);
        user.login_ids.push(new.to_string());
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Replace global role_names entirely.
    pub fn set_roles(
        &mut self,
        login_id: &str,
        roles: Vec<String>,
    ) -> Result<&User, EmulatorError> {
        let user = self.load_mut(login_id)?;
        user.role_names = roles;
        let uid = user.user_id.clone();
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Remove specific roles from global role_names. Idempotent.
    pub fn remove_roles(
        &mut self,
        login_id: &str,
        roles: &[String],
    ) -> Result<&User, EmulatorError> {
        let user = self.load_mut(login_id)?;
        user.role_names.retain(|r| !roles.contains(r));
        let uid = user.user_id.clone();
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Append roles without duplicates.
    pub fn add_roles(
        &mut self,
        login_id: &str,
        roles: Vec<String>,
    ) -> Result<&User, EmulatorError> {
        let user = self.load_mut(login_id)?;
        for r in roles {
            if !user.role_names.contains(&r) {
                user.role_names.push(r);
            }
        }
        let uid = user.user_id.clone();
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Update a single custom attribute key on a user.
    pub fn update_custom_attribute(
        &mut self,
        login_id: &str,
        key: String,
        value: serde_json::Value,
    ) -> Result<&User, EmulatorError> {
        let user = self.load_mut(login_id)?;
        user.custom_attributes.insert(key, value);
        let uid = user.user_id.clone();
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Update picture URL.
    pub fn update_picture(
        &mut self,
        login_id: &str,
        picture: String,
    ) -> Result<&User, EmulatorError> {
        let user = self.load_mut(login_id)?;
        user.picture = Some(picture);
        let uid = user.user_id.clone();
        Ok(self.by_user_id.get(&uid).unwrap())
    }

    /// Insert multiple users in order. Non-transactional: stops on first conflict but retains prior inserts.
    pub fn batch_insert(&mut self, users: Vec<User>) -> Result<Vec<&User>, EmulatorError> {
        let mut inserted_uids = Vec::new();
        for user in users {
            let uid = user.user_id.clone();
            self.insert(user)?;
            inserted_uids.push(uid);
        }
        Ok(inserted_uids
            .iter()
            .map(|uid| self.by_user_id.get(uid).unwrap())
            .collect())
    }
}

// ─── Builder helpers ──────────────────────────────────────────────────────────

pub fn new_user_id() -> String {
    format!("U{}", Uuid::new_v4().as_simple())
}

#[derive(Default)]
pub struct UserUpdate {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub verified_email: Option<bool>,
    pub verified_phone: Option<bool>,
    pub role_names: Option<Vec<String>>,
    pub custom_attributes: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub user_tenants: Option<Vec<UserTenant>>,
}

#[derive(Default)]
pub struct UserPatch {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub verified_email: Option<bool>,
    pub verified_phone: Option<bool>,
    pub role_names: Option<Vec<String>>,
    pub custom_attributes: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub user_tenants: Option<Vec<UserTenant>>,
}

#[derive(Default)]
pub struct SearchQuery {
    pub emails: Option<Vec<String>>,
    pub phones: Option<Vec<String>>,
    pub custom_attributes: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub with_test_user: bool,
    pub page: usize,
    pub limit: usize,
    // ── New filter fields ──
    pub login_ids: Option<Vec<String>>,
    pub statuses: Option<Vec<String>>,
    pub tenant_ids: Option<Vec<String>>,
    pub role_names: Option<Vec<String>>,
    pub text: Option<String>,
    pub sort: Option<SortSpec>,
    pub created_after: Option<u64>,
    pub created_before: Option<u64>,
}

#[derive(Default, Clone)]
pub struct SortSpec {
    pub field: String,
    pub desc: bool,
}

// ─── const for cap ─────────────────────────────────────────────────────────────
// Re-exported so token_store can use the same cap philosophy
pub const _USER_STORE_MAX: usize = MAX_TOKENS;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::User;

    fn make_user(login_id: &str) -> User {
        let mut u = User::default();
        u.user_id = new_user_id();
        u.login_ids = vec![login_id.to_string()];
        u.email = Some(format!("{}@example.com", login_id));
        u.status = "enabled".into();
        u
    }

    fn make_test_user(login_id: &str) -> User {
        let mut u = make_user(login_id);
        u._is_test_user = true;
        u
    }

    #[test]
    fn create_and_load_by_login_id() {
        let mut store = UserStore::new();
        let user = make_user("alice");
        store.insert(user).unwrap();
        let loaded = store.load("alice").unwrap();
        assert_eq!(loaded.login_ids[0], "alice");
    }

    #[test]
    fn load_by_user_id() {
        let mut store = UserStore::new();
        let user = make_user("bob");
        let uid = user.user_id.clone();
        store.insert(user).unwrap();
        let loaded = store.load_by_user_id(&uid).unwrap();
        assert_eq!(loaded.user_id, uid);
    }

    #[test]
    fn duplicate_login_id_is_rejected() {
        let mut store = UserStore::new();
        store.insert(make_user("carol")).unwrap();
        let err = store.insert(make_user("carol")).unwrap_err();
        assert!(matches!(err, EmulatorError::UserAlreadyExists));
    }

    #[test]
    fn load_nonexistent_user_returns_not_found() {
        let store = UserStore::new();
        let err = store.load("nobody").unwrap_err();
        assert!(matches!(err, EmulatorError::UserNotFound));
    }

    #[test]
    fn delete_by_login_id_removes_user() {
        let mut store = UserStore::new();
        store.insert(make_user("dave")).unwrap();
        store.delete_by_login_id("dave");
        assert!(matches!(
            store.load("dave"),
            Err(EmulatorError::UserNotFound)
        ));
    }

    #[test]
    fn delete_nonexistent_is_idempotent() {
        let mut store = UserStore::new();
        store.delete_by_login_id("ghost"); // should not panic
    }

    #[test]
    fn delete_all_test_users_keeps_regular_users() {
        let mut store = UserStore::new();
        store.insert(make_user("regular")).unwrap();
        store.insert(make_test_user("test1")).unwrap();
        store.insert(make_test_user("test2")).unwrap();

        store.delete_all_test_users();

        assert!(store.load("regular").is_ok());
        assert!(store.load("test1").is_err());
        assert!(store.load("test2").is_err());
    }

    #[test]
    fn update_clears_unprovided_phone() {
        let mut store = UserStore::new();
        let mut u = make_user("eve");
        u.phone = Some("555-0100".into());
        store.insert(u).unwrap();

        store
            .update(
                "eve",
                UserUpdate {
                    ..UserUpdate::default()
                },
            )
            .unwrap();
        let user = store.load("eve").unwrap();
        assert!(user.phone.is_none());
    }

    #[test]
    fn patch_preserves_existing_phone() {
        let mut store = UserStore::new();
        let mut u = make_user("frank");
        u.phone = Some("555-0101".into());
        store.insert(u).unwrap();

        store.patch("frank", UserPatch::default()).unwrap();
        let user = store.load("frank").unwrap();
        assert_eq!(user.phone.as_deref(), Some("555-0101"));
    }

    #[test]
    fn search_by_email_returns_match() {
        let mut store = UserStore::new();
        let mut u = make_user("grace");
        u.email = Some("grace@example.com".into());
        store.insert(u).unwrap();

        let results = store.search(&SearchQuery {
            emails: Some(vec!["grace@example.com".into()]),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_excludes_test_users_by_default() {
        let mut store = UserStore::new();
        store.insert(make_user("real")).unwrap();
        store.insert(make_test_user("fake")).unwrap();

        let results = store.search(&SearchQuery::default());
        assert!(results.iter().all(|u| !u._is_test_user));
    }

    #[test]
    fn search_includes_test_users_when_flag_set() {
        let mut store = UserStore::new();
        store.insert(make_user("real2")).unwrap();
        store.insert(make_test_user("fake2")).unwrap();

        let results = store.search(&SearchQuery {
            with_test_user: true,
            ..Default::default()
        });
        assert!(results.iter().any(|u| u._is_test_user));
    }

    // Why: a backend admin-bootstrap existence check (query by email) searches by
    //      email WITHOUT withTestUser. Real Descope returns test users for targeted
    //      identifier lookups, so an e2e-seeded test user must be findable by
    //      email — otherwise the caller assumes the user is new and attempts a
    //      duplicate signup, which fails with "Already exists" and aborts the
    //      add-user flow (observed as e2e shards timing out).
    // Decision: a targeted emails/login_ids/phones filter surfaces test users
    //           even without with_test_user; broad listings still hide them.
    #[test]
    fn search_by_email_includes_test_users_for_targeted_lookup() {
        let mut store = UserStore::new();
        let mut u = make_test_user("seed");
        u.email = Some("seed@example.com".into());
        store.insert(u).unwrap();

        let results = store.search(&SearchQuery {
            emails: Some(vec!["seed@example.com".into()]),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
        assert!(results[0]._is_test_user);
    }

    #[test]
    fn add_tenant_is_idempotent() {
        let mut store = UserStore::new();
        store.insert(make_user("hugo")).unwrap();
        let tenant = UserTenant {
            tenant_id: "t1".into(),
            tenant_name: "Acme".into(),
            role_names: vec![],
        };
        store.add_tenant("hugo", tenant.clone()).unwrap();
        store.add_tenant("hugo", tenant).unwrap();
        let user = store.load("hugo").unwrap();
        assert_eq!(user.user_tenants.len(), 1);
    }

    #[test]
    fn set_and_check_password_succeeds() {
        let mut store = UserStore::new();
        store.insert(make_user("ivan")).unwrap();
        let hash = bcrypt::hash("secret", 4).unwrap();
        store.set_password("ivan", hash).unwrap();
        store.check_password("ivan", "secret").unwrap();
    }

    #[test]
    fn wrong_password_fails() {
        let mut store = UserStore::new();
        store.insert(make_user("julia")).unwrap();
        let hash = bcrypt::hash("correct", 4).unwrap();
        store.set_password("julia", hash).unwrap();
        let err = store.check_password("julia", "wrong").unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidCredentials));
    }

    #[test]
    fn enable_and_disable_user() {
        let mut store = UserStore::new();
        store.insert(make_user("kim")).unwrap();
        let user = store.disable("kim").unwrap();
        assert_eq!(user.status, "disabled");
        let user = store.enable("kim").unwrap();
        assert_eq!(user.status, "enabled");
    }

    #[test]
    fn disable_unknown_user_returns_not_found() {
        let mut store = UserStore::new();
        let err = store.disable("nobody").unwrap_err();
        assert!(matches!(err, EmulatorError::UserNotFound));
    }

    #[test]
    fn remove_tenant_removes_entry() {
        let mut store = UserStore::new();
        store.insert(make_user("lena")).unwrap();
        let tenant = UserTenant {
            tenant_id: "t1".into(),
            tenant_name: "Corp".into(),
            role_names: vec![],
        };
        store.add_tenant("lena", tenant).unwrap();
        store.remove_tenant("lena", "t1").unwrap();
        let user = store.load("lena").unwrap();
        assert!(user.user_tenants.is_empty());
    }

    #[test]
    fn remove_tenant_is_idempotent() {
        let mut store = UserStore::new();
        store.insert(make_user("lena2")).unwrap();
        // User is not in any tenant — should not panic
        store.remove_tenant("lena2", "t1").unwrap();
    }

    #[test]
    fn set_tenant_roles_replaces_roles() {
        let mut store = UserStore::new();
        store.insert(make_user("mario")).unwrap();
        let tenant = UserTenant {
            tenant_id: "t2".into(),
            tenant_name: "Corp".into(),
            role_names: vec!["viewer".into()],
        };
        store.add_tenant("mario", tenant).unwrap();
        store
            .set_tenant_roles("mario", "t2", vec!["admin".into()])
            .unwrap();
        let user = store.load("mario").unwrap();
        assert_eq!(user.user_tenants[0].role_names, vec!["admin"]);
    }

    #[test]
    fn set_tenant_roles_returns_not_found_for_absent_tenant() {
        let mut store = UserStore::new();
        store.insert(make_user("nadia")).unwrap();
        let err = store
            .set_tenant_roles("nadia", "missing-tenant", vec![])
            .unwrap_err();
        assert!(matches!(err, EmulatorError::TenantNotFound));
    }

    // ── Tests for new methods (tasks 1.1–1.6) ──────────────────────────────

    #[test]
    fn update_login_id_renames_index_and_vec() {
        let mut store = UserStore::new();
        store.insert(make_user("old@example.com")).unwrap();
        store
            .update_login_id("old@example.com", "new@example.com")
            .unwrap();
        // new loginId works
        assert!(store.load("new@example.com").is_ok());
        // old loginId is gone
        assert!(matches!(
            store.load("old@example.com"),
            Err(EmulatorError::UserNotFound)
        ));
        // login_ids vec is updated
        let user = store.load("new@example.com").unwrap();
        assert!(user.login_ids.contains(&"new@example.com".to_string()));
        assert!(!user.login_ids.contains(&"old@example.com".to_string()));
    }

    #[test]
    fn update_login_id_unknown_old_returns_not_found() {
        let mut store = UserStore::new();
        let err = store
            .update_login_id("ghost@x.com", "new@x.com")
            .unwrap_err();
        assert!(matches!(err, EmulatorError::UserNotFound));
    }

    #[test]
    fn update_login_id_new_already_taken_returns_conflict() {
        let mut store = UserStore::new();
        store.insert(make_user("alice@x.com")).unwrap();
        store.insert(make_user("bob@x.com")).unwrap();
        let err = store
            .update_login_id("alice@x.com", "bob@x.com")
            .unwrap_err();
        assert!(matches!(err, EmulatorError::UserAlreadyExists));
    }

    #[test]
    fn set_roles_replaces_role_names() {
        let mut store = UserStore::new();
        let mut u = make_user("charlie");
        u.role_names = vec!["viewer".into()];
        store.insert(u).unwrap();
        store
            .set_roles("charlie", vec!["admin".into(), "editor".into()])
            .unwrap();
        let user = store.load("charlie").unwrap();
        assert_eq!(user.role_names, vec!["admin", "editor"]);
    }

    #[test]
    fn remove_roles_subtracts_specified_roles() {
        let mut store = UserStore::new();
        let mut u = make_user("diana");
        u.role_names = vec!["admin".into(), "viewer".into(), "editor".into()];
        store.insert(u).unwrap();
        store
            .remove_roles("diana", &["admin".to_string(), "editor".to_string()])
            .unwrap();
        let user = store.load("diana").unwrap();
        assert_eq!(user.role_names, vec!["viewer"]);
    }

    #[test]
    fn batch_insert_all_succeed() {
        let mut store = UserStore::new();
        let users = vec![make_user("u1"), make_user("u2"), make_user("u3")];
        let result = store.batch_insert(users).unwrap();
        assert_eq!(result.len(), 3);
        assert!(store.load("u1").is_ok());
        assert!(store.load("u2").is_ok());
        assert!(store.load("u3").is_ok());
    }

    #[test]
    fn batch_insert_stops_on_duplicate_but_persists_prior() {
        let mut store = UserStore::new();
        store.insert(make_user("dup")).unwrap(); // pre-existing
        let users = vec![make_user("before"), make_user("dup")]; // dup is 2nd
        let err = store.batch_insert(users).unwrap_err();
        assert!(matches!(err, EmulatorError::UserAlreadyExists));
        // "before" was inserted before the failure
        assert!(store.load("before").is_ok());
    }

    // ── Search filter tests (tasks 2.1–2.9) ──────────────────────────────

    #[test]
    fn search_by_login_ids_returns_matching_users() {
        let mut store = UserStore::new();
        store.insert(make_user("alice")).unwrap();
        store.insert(make_user("bob")).unwrap();
        store.insert(make_user("carol")).unwrap();

        let results = store.search(&SearchQuery {
            login_ids: Some(vec!["alice".into(), "carol".into()]),
            ..Default::default()
        });
        assert_eq!(results.len(), 2);
        let names: Vec<&str> = results.iter().map(|u| u.login_ids[0].as_str()).collect();
        assert!(names.contains(&"alice"));
        assert!(names.contains(&"carol"));
    }

    #[test]
    fn search_by_statuses_returns_matching_users() {
        let mut store = UserStore::new();
        store.insert(make_user("active")).unwrap();
        let mut disabled = make_user("off");
        disabled.status = "disabled".into();
        store.insert(disabled).unwrap();

        let results = store.search(&SearchQuery {
            statuses: Some(vec!["disabled".into()]),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].login_ids[0], "off");
    }

    #[test]
    fn search_by_tenant_ids_returns_matching_users() {
        let mut store = UserStore::new();
        let mut with_tenant = make_user("tenant_user");
        with_tenant.user_tenants.push(UserTenant {
            tenant_id: "acme".into(),
            tenant_name: "Acme Corp".into(),
            role_names: vec![],
        });
        store.insert(with_tenant).unwrap();
        store.insert(make_user("no_tenant")).unwrap();

        let results = store.search(&SearchQuery {
            tenant_ids: Some(vec!["acme".into()]),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].login_ids[0], "tenant_user");
    }

    #[test]
    fn search_by_role_names_returns_matching_users() {
        let mut store = UserStore::new();
        let mut admin = make_user("admin_user");
        admin.role_names = vec!["admin".into()];
        store.insert(admin).unwrap();

        // User with tenant-level role
        let mut tenant_editor = make_user("tenant_editor");
        tenant_editor.user_tenants.push(UserTenant {
            tenant_id: "t1".into(),
            tenant_name: "T1".into(),
            role_names: vec!["editor".into()],
        });
        store.insert(tenant_editor).unwrap();

        store.insert(make_user("no_role")).unwrap();

        // Search for admin
        let results = store.search(&SearchQuery {
            role_names: Some(vec!["admin".into()]),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].login_ids[0], "admin_user");

        // Search for editor (tenant-level)
        let results = store.search(&SearchQuery {
            role_names: Some(vec!["editor".into()]),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].login_ids[0], "tenant_editor");
    }

    #[test]
    fn search_by_text_matches_name_email_phone() {
        let mut store = UserStore::new();
        let mut u = make_user("john");
        u.name = Some("John Doe".into());
        u.email = Some("john@example.com".into());
        u.phone = Some("+15551234".into());
        store.insert(u).unwrap();
        store.insert(make_user("unrelated")).unwrap();

        // Match by name
        let results = store.search(&SearchQuery {
            text: Some("doe".into()),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);

        // Match by email
        let results = store.search(&SearchQuery {
            text: Some("john@".into()),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);

        // Match by phone
        let results = store.search(&SearchQuery {
            text: Some("5551234".into()),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_by_text_is_case_insensitive() {
        let mut store = UserStore::new();
        let mut u = make_user("alice");
        u.name = Some("Alice Wonderland".into());
        store.insert(u).unwrap();

        let results = store.search(&SearchQuery {
            text: Some("ALICE".into()),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);

        let results = store.search(&SearchQuery {
            text: Some("wonderland".into()),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_by_date_range_filters_correctly() {
        let mut store = UserStore::new();
        let mut old = make_user("old");
        old.created_time = 1000;
        store.insert(old).unwrap();

        let mut mid = make_user("mid");
        mid.created_time = 2000;
        store.insert(mid).unwrap();

        let mut new_user = make_user("new");
        new_user.created_time = 3000;
        store.insert(new_user).unwrap();

        let results = store.search(&SearchQuery {
            created_after: Some(1500),
            created_before: Some(2500),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].login_ids[0], "mid");
    }

    #[test]
    fn search_sort_by_name_ascending() {
        let mut store = UserStore::new();
        let mut c = make_user("charlie");
        c.name = Some("Charlie".into());
        store.insert(c).unwrap();
        let mut a = make_user("alice_sort");
        a.name = Some("Alice".into());
        store.insert(a).unwrap();
        let mut b = make_user("bob_sort");
        b.name = Some("Bob".into());
        store.insert(b).unwrap();

        let results = store.search(&SearchQuery {
            sort: Some(SortSpec {
                field: "name".into(),
                desc: false,
            }),
            ..Default::default()
        });
        let names: Vec<&str> = results.iter().map(|u| u.name.as_deref().unwrap()).collect();
        assert_eq!(names, vec!["Alice", "Bob", "Charlie"]);
    }

    #[test]
    fn search_combined_filters_use_and_semantics() {
        let mut store = UserStore::new();

        // User: enabled + acme tenant
        let mut u1 = make_user("match");
        u1.status = "enabled".into();
        u1.user_tenants.push(UserTenant {
            tenant_id: "acme".into(),
            tenant_name: "Acme".into(),
            role_names: vec![],
        });
        store.insert(u1).unwrap();

        // User: disabled + acme tenant
        let mut u2 = make_user("disabled_acme");
        u2.status = "disabled".into();
        u2.user_tenants.push(UserTenant {
            tenant_id: "acme".into(),
            tenant_name: "Acme".into(),
            role_names: vec![],
        });
        store.insert(u2).unwrap();

        // User: enabled + no tenant
        store.insert(make_user("enabled_no_tenant")).unwrap();

        let results = store.search(&SearchQuery {
            statuses: Some(vec!["enabled".into()]),
            tenant_ids: Some(vec!["acme".into()]),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].login_ids[0], "match");
    }

    // ── user-api-completeness tests ───────────────────────────────────

    #[test]
    fn set_status_accepts_invited() {
        let mut store = UserStore::new();
        store.insert(make_user("invite_me")).unwrap();
        let user = store.set_status("invite_me", "invited").unwrap();
        assert_eq!(user.status, "invited");
    }

    #[test]
    fn add_roles_appends_without_duplicates() {
        let mut store = UserStore::new();
        store.insert(make_user("role_user")).unwrap();
        store.set_roles("role_user", vec!["admin".into()]).unwrap();
        let user = store
            .add_roles("role_user", vec!["editor".into(), "admin".into()])
            .unwrap();
        // admin should not be duplicated
        assert_eq!(user.role_names.len(), 2);
        assert!(user.role_names.contains(&"admin".to_string()));
        assert!(user.role_names.contains(&"editor".to_string()));
    }

    #[test]
    fn update_picture_sets_url() {
        let mut store = UserStore::new();
        store.insert(make_user("pic_user")).unwrap();
        let user = store
            .update_picture("pic_user", "https://example.com/avatar.png".into())
            .unwrap();
        assert_eq!(
            user.picture.as_deref(),
            Some("https://example.com/avatar.png")
        );
    }

    #[test]
    fn update_custom_attribute_merges_single_key() {
        let mut store = UserStore::new();
        store.insert(make_user("attr_user")).unwrap();
        store
            .update_custom_attribute("attr_user", "tier".into(), serde_json::json!("gold"))
            .unwrap();
        let user = store.load("attr_user").unwrap();
        assert_eq!(
            user.custom_attributes.get("tier"),
            Some(&serde_json::json!("gold"))
        );

        // Update same key — should overwrite
        store
            .update_custom_attribute("attr_user", "tier".into(), serde_json::json!("platinum"))
            .unwrap();
        let user = store.load("attr_user").unwrap();
        assert_eq!(
            user.custom_attributes.get("tier"),
            Some(&serde_json::json!("platinum"))
        );
    }
}
