use crate::error::EmulatorError;
use rand::Rng;
use std::collections::HashMap;

/// Generates a 6-digit numeric OTP code.
pub fn generate_otp_code() -> String {
    let code: u32 = rand::thread_rng().gen_range(100_000..=999_999);
    code.to_string()
}

/// Pending OTP store, keyed by userId.
/// Codes support peek (non-destructive read) for the emulator escape hatch,
/// and consume (verify + delete) for the verify endpoint.
#[derive(Default)]
pub struct OtpStore {
    /// userId → 6-digit code string
    pending: HashMap<String, String>,
}

impl OtpStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store (or overwrite) an OTP code for a user.
    pub fn store(&mut self, user_id: &str, code: String) {
        self.pending.insert(user_id.to_string(), code);
    }

    /// Non-destructive read — returns the pending code without consuming it.
    /// Used by GET /emulator/otp/:loginId.
    pub fn peek(&self, user_id: &str) -> Option<&str> {
        self.pending.get(user_id).map(|s| s.as_str())
    }

    /// Consume a code — verifies it matches and removes it.
    /// Returns Ok(()) on success; Err(InvalidToken) on mismatch or absence.
    pub fn consume(&mut self, user_id: &str, code: &str) -> Result<(), EmulatorError> {
        match self.pending.get(user_id) {
            Some(stored) if stored == code => {
                self.pending.remove(user_id);
                Ok(())
            }
            _ => Err(EmulatorError::InvalidToken),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Return all pending OTP codes as a map of userId → code.
    /// Used by GET /emulator/otps for test harness introspection.
    pub fn list_all(&self) -> std::collections::HashMap<String, String> {
        self.pending.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_code_is_six_digits() {
        let code = generate_otp_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        let n: u32 = code.parse().unwrap();
        assert!(n >= 100_000 && n <= 999_999);
    }

    #[test]
    fn store_and_peek_returns_code() {
        let mut store = OtpStore::new();
        store.store("u1", "123456".into());
        assert_eq!(store.peek("u1"), Some("123456"));
    }

    #[test]
    fn peek_does_not_consume() {
        let mut store = OtpStore::new();
        store.store("u1", "123456".into());
        let _ = store.peek("u1");
        assert_eq!(store.peek("u1"), Some("123456"));
    }

    #[test]
    fn consume_correct_code_succeeds_and_removes() {
        let mut store = OtpStore::new();
        store.store("u1", "123456".into());
        store.consume("u1", "123456").unwrap();
        assert!(store.peek("u1").is_none());
    }

    #[test]
    fn consume_wrong_code_fails() {
        let mut store = OtpStore::new();
        store.store("u1", "123456".into());
        let err = store.consume("u1", "000000").unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
        // Code is still there after failed consume
        assert!(store.peek("u1").is_some());
    }

    #[test]
    fn consume_after_consume_fails() {
        let mut store = OtpStore::new();
        store.store("u1", "123456".into());
        store.consume("u1", "123456").unwrap();
        let err = store.consume("u1", "123456").unwrap_err();
        assert!(matches!(err, EmulatorError::InvalidToken));
    }

    #[test]
    fn peek_nonexistent_returns_none() {
        let store = OtpStore::new();
        assert!(store.peek("ghost").is_none());
    }

    #[test]
    fn store_overwrites_existing_code() {
        let mut store = OtpStore::new();
        store.store("u1", "111111".into());
        store.store("u1", "222222".into());
        assert_eq!(store.peek("u1"), Some("222222"));
    }

    #[test]
    fn reset_clears_all_codes() {
        let mut store = OtpStore::new();
        store.store("u1", "123456".into());
        store.reset();
        assert!(store.peek("u1").is_none());
    }
}
