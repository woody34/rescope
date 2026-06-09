use crate::{
    error::EmulatorError,
    types::{TokenEntry, TokenType},
};
use rand::RngCore;
use std::{
    collections::{HashMap, VecDeque},
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::warn;

const TOKEN_STORE_MAX: usize = 10_000;

/// Generates a 64-character lowercase hex token (32 random bytes).
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs()
}

/// Single-use token store capped at TOKEN_STORE_MAX entries.
#[derive(Default)]
pub struct TokenStore {
    entries: HashMap<String, TokenEntry>,
    /// Insertion-order queue for eviction
    order: VecDeque<String>,
}

impl TokenStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, token: String, user_id: String, token_type: TokenType) {
        if self.entries.len() >= TOKEN_STORE_MAX {
            // Evict oldest
            if let Some(oldest) = self.order.pop_front() {
                let entry = self.entries.remove(&oldest);
                if let Some(e) = entry {
                    warn!(
                        token_type = ?e.token_type,
                        "Token store cap reached; evicted oldest token"
                    );
                }
            }
        }
        let entry = TokenEntry {
            user_id,
            token_type,
            created_at: now_secs(),
        };
        self.entries.insert(token.clone(), entry);
        self.order.push_back(token);
    }

    /// Consume (remove) a token and return its entry.
    pub fn consume(&mut self, token: &str) -> Result<TokenEntry, EmulatorError> {
        // Remove from order queue too
        if let Some(entry) = self.entries.remove(token) {
            self.order.retain(|t| t != token);
            Ok(entry)
        } else {
            Err(EmulatorError::InvalidToken)
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_token_is_64_hex_chars() {
        let token = generate_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generated_tokens_are_unique() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_ne!(t1, t2);
    }

    #[test]
    fn insert_and_consume() {
        let mut store = TokenStore::new();
        let token = generate_token();
        store.insert(token.clone(), "user-1".into(), TokenType::Magic);
        let entry = store.consume(&token).unwrap();
        assert_eq!(entry.user_id, "user-1");
        assert_eq!(entry.token_type, TokenType::Magic);
    }

    #[test]
    fn token_is_single_use() {
        let mut store = TokenStore::new();
        let token = generate_token();
        store.insert(token.clone(), "user-1".into(), TokenType::Magic);
        store.consume(&token).unwrap();
        let err = store.consume(&token).unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }

    #[test]
    fn consuming_nonexistent_token_fails() {
        let mut store = TokenStore::new();
        let err = store.consume("deadbeef").unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }

    #[test]
    fn cap_evicts_oldest_entry() {
        let mut store = TokenStore::new();
        let first_token = generate_token();
        store.insert(first_token.clone(), "u0".into(), TokenType::Magic);
        // Fill up remaining slots
        for i in 1..TOKEN_STORE_MAX {
            store.insert(generate_token(), format!("u{i}"), TokenType::Magic);
        }
        // Insert one more — should evict the first
        store.insert(generate_token(), "u_new".into(), TokenType::Magic);
        let err = store.consume(&first_token).unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }

    #[test]
    fn reset_clears_all_tokens() {
        let mut store = TokenStore::new();
        let token = generate_token();
        store.insert(token.clone(), "u".into(), TokenType::Saml);
        store.reset();
        let err = store.consume(&token).unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }
}
