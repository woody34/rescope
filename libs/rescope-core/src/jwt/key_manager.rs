use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rsa::{
    pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey},
    RsaPrivateKey, RsaPublicKey,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Manages the RSA key pair used for JWT signing and JWKS.
#[derive(Clone)]
pub struct KeyManager {
    pub encoding_key: jsonwebtoken::EncodingKey,
    pub decoding_key: jsonwebtoken::DecodingKey,
    pub kid: String,
    /// Pre-computed JWK for the JWKS endpoint.
    pub jwk: serde_json::Value,
    /// PKCS8 PEM of the private key (used for snapshot export).
    pub private_pem: String,
}

impl KeyManager {
    /// Generate a fresh 2048-bit RSA key pair.
    pub fn generate() -> Result<Arc<Self>> {
        let mut rng = rand::thread_rng();
        let private_key =
            RsaPrivateKey::new(&mut rng, 2048).context("Failed to generate RSA key pair")?;
        Self::from_private_key(private_key)
    }

    /// Load key pair from a PKCS8 PEM file.
    pub fn from_pem_file(path: &str) -> Result<Arc<Self>> {
        let pem = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read key file: {path}"))?;
        let private_key = RsaPrivateKey::from_pkcs8_pem(&pem)
            .with_context(|| format!("Invalid PKCS8 PEM in: {path}"))?;
        Self::from_private_key(private_key)
    }

    fn from_private_key(private_key: RsaPrivateKey) -> Result<Arc<Self>> {
        let public_key = RsaPublicKey::from(&private_key);

        // Export to PEM for jsonwebtoken
        let private_pem = private_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .context("Failed to serialize private key to PEM")?;
        let public_pem = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .context("Failed to serialize public key to PEM")?;

        let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(private_pem.as_bytes())
            .context("Failed to create EncodingKey")?;
        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(public_pem.as_bytes())
            .context("Failed to create DecodingKey")?;

        // kid: first 16 hex chars of SHA-256 of DER-encoded public key
        let der = public_key
            .to_public_key_der()
            .context("Failed to DER-encode public key")?;
        let hash = Sha256::digest(der.as_bytes());
        let kid = hex::encode(&hash[..8]); // 8 bytes → 16 hex chars

        // Build JWK from raw public key components
        use rsa::traits::PublicKeyParts;
        let n = URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
        let e = URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());
        let jwk = serde_json::json!({
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": kid,
            "n": n,
            "e": e
        });

        Ok(Arc::new(Self {
            encoding_key,
            decoding_key,
            kid,
            jwk,
            private_pem: private_pem.to_string(),
        }))
    }

    /// Reconstruct a KeyManager from a PKCS8 PEM string (e.g., from snapshot import).
    pub fn from_private_pem(pem: &str) -> Result<Arc<Self>> {
        let private_key =
            RsaPrivateKey::from_pkcs8_pem(pem).context("Invalid PKCS8 PEM in snapshot")?;
        Self::from_private_key(private_key)
    }

    pub fn jwks(&self) -> serde_json::Value {
        serde_json::json!({ "keys": [self.jwk.clone()] })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct TestClaims {
        sub: String,
        exp: u64,
    }

    fn test_claims() -> TestClaims {
        TestClaims {
            sub: "test-user".into(),
            exp: u64::MAX,
        }
    }

    #[test]
    fn generates_key_pair() {
        let km = KeyManager::generate().expect("key generation failed");
        assert_eq!(km.kid.len(), 16);
    }

    #[test]
    fn kid_is_16_hex_chars() {
        let km = KeyManager::generate().unwrap();
        assert!(km.kid.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(km.kid.len(), 16);
    }

    #[test]
    fn signs_and_verifies_token() {
        let km = KeyManager::generate().unwrap();
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(km.kid.clone());

        let token = encode(&header, &test_claims(), &km.encoding_key).expect("encode failed");

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = false;
        let data =
            decode::<TestClaims>(&token, &km.decoding_key, &validation).expect("decode failed");
        assert_eq!(data.claims.sub, "test-user");
    }

    #[test]
    fn jwks_contains_correct_fields() {
        let km = KeyManager::generate().unwrap();
        let jwks = km.jwks();
        let key = &jwks["keys"][0];
        assert_eq!(key["kty"], "RSA");
        assert_eq!(key["use"], "sig");
        assert_eq!(key["alg"], "RS256");
        assert_eq!(key["kid"], km.kid.as_str());
        assert!(key["n"].is_string());
        assert!(key["e"].is_string());
    }

    #[test]
    fn different_key_cannot_verify() {
        let km1 = KeyManager::generate().unwrap();
        let km2 = KeyManager::generate().unwrap();

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(km1.kid.clone());
        let token = encode(&header, &test_claims(), &km1.encoding_key).unwrap();

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = false;
        let result = decode::<TestClaims>(&token, &km2.decoding_key, &validation);
        assert!(result.is_err());
    }

    #[test]
    fn from_pem_file_invalid_path_errors() {
        let result = KeyManager::from_pem_file("/nonexistent/key.pem");
        assert!(result.is_err());
        let msg = result.err().unwrap().to_string();
        assert!(msg.contains("/nonexistent/key.pem"));
    }
}
