use std::collections::HashSet;

/// Refresh token revocation set.
#[derive(Default)]
pub struct RevocationStore {
    revoked: HashSet<String>,
}

impl RevocationStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn revoke(&mut self, token: String) {
        self.revoked.insert(token);
    }

    pub fn is_revoked(&self, token: &str) -> bool {
        self.revoked.contains(token)
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revoked_token_is_detected() {
        let mut store = RevocationStore::new();
        store.revoke("tok1".into());
        assert!(store.is_revoked("tok1"));
    }

    #[test]
    fn non_revoked_token_passes_check() {
        let store = RevocationStore::new();
        assert!(!store.is_revoked("tok2"));
    }

    #[test]
    fn reset_clears_revocation_set() {
        let mut store = RevocationStore::new();
        store.revoke("tok3".into());
        store.reset();
        assert!(!store.is_revoked("tok3"));
    }
}
