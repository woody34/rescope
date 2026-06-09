use crate::types::User;
use std::collections::HashMap;

/// Minimal in-memory user store — validates that sync RwLock-based stores work in WASM.
pub struct UserStore {
    users: HashMap<String, User>,
}

impl UserStore {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn create(&mut self, user: User) {
        self.users.insert(user.user_id.clone(), user);
    }

    pub fn list(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    pub fn count(&self) -> usize {
        self.users.len()
    }
}
