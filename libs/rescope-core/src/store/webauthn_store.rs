//! WebAuthn (passkey) ceremonies: the options handed to the browser, the
//! credentials users register, and the checks applied to what the browser sends
//! back.
//!
//! The emulator checks what a relying party must check: the challenge it issued,
//! the ceremony type, the RP id hash and the user-presence flag, and on sign-in
//! the assertion signature against the stored public key (ES256 or RS256). It
//! does not check attestation statements, as Descope does not for `none`.

use crate::error::EmulatorError;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// How long a started ceremony waits for its finish (seconds).
const TRANSACTION_TTL_SECS: u64 = 300;

/// COSE algorithm ids the emulator offers and verifies.
const COSE_ES256: i64 = -7;
const COSE_RS256: i64 = -257;

/// Authenticator data flags.
const FLAG_USER_PRESENT: u8 = 0x01;
const FLAG_ATTESTED_CREDENTIAL: u8 = 0x40;

/// Which ceremony a transaction belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ceremony {
    /// Register a passkey for a new user.
    SignUp,
    /// Sign in with an existing passkey.
    SignIn,
    /// Add a passkey to a signed-in user.
    Update,
}

impl Ceremony {
    fn client_data_type(self) -> &'static str {
        match self {
            Ceremony::SignIn => "webauthn.get",
            Ceremony::SignUp | Ceremony::Update => "webauthn.create",
        }
    }
}

/// A started ceremony, waiting for the browser's response.
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub ceremony: Ceremony,
    pub login_id: String,
    /// Empty for a sign-up, whose user does not exist yet.
    pub user_id: String,
    pub challenge: Vec<u8>,
    pub rp_id: String,
    pub created_at: u64,
}

/// The public key of a registered credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialKey {
    Es256 { x: Vec<u8>, y: Vec<u8> },
    Rs256 { n: Vec<u8>, e: Vec<u8> },
}

/// A passkey a user registered.
#[derive(Debug, Clone)]
pub struct Credential {
    pub id: Vec<u8>,
    pub user_id: String,
    pub key: CredentialKey,
    pub sign_count: u32,
}

/// In-memory WebAuthn state: credentials by user, and open ceremonies.
#[derive(Default)]
pub struct WebAuthnStore {
    credentials: HashMap<String, Vec<Credential>>,
    transactions: HashMap<String, Transaction>,
}

impl WebAuthnStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// True when the user has at least one passkey.
    pub fn has_credentials(&self, user_id: &str) -> bool {
        self.credentials
            .get(user_id)
            .is_some_and(|creds| !creds.is_empty())
    }

    pub fn credentials_of(&self, user_id: &str) -> &[Credential] {
        self.credentials
            .get(user_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Open a ceremony and return the options the browser passes to
    /// `navigator.credentials.create` / `.get`, as the JSON string Descope sends.
    #[allow(clippy::too_many_arguments)]
    pub fn begin(
        &mut self,
        ceremony: Ceremony,
        login_id: &str,
        user_id: &str,
        display_name: &str,
        rp_id: &str,
        now: u64,
        transaction_id: String,
        challenge: Vec<u8>,
    ) -> (String, String) {
        self.transactions
            .retain(|_, t| now.saturating_sub(t.created_at) < TRANSACTION_TTL_SECS);

        let existing: Vec<Value> = self
            .credentials_of(user_id)
            .iter()
            .map(|c| json!({ "type": "public-key", "id": URL_SAFE_NO_PAD.encode(&c.id) }))
            .collect();
        let challenge_b64 = URL_SAFE_NO_PAD.encode(&challenge);
        let options = match ceremony {
            Ceremony::SignIn => json!({
                "publicKey": {
                    "challenge": challenge_b64,
                    "rpId": rp_id,
                    "allowCredentials": existing,
                    "userVerification": "preferred",
                    "timeout": 60000
                }
            }),
            Ceremony::SignUp | Ceremony::Update => json!({
                "publicKey": {
                    "challenge": challenge_b64,
                    "rp": { "id": rp_id, "name": rp_id },
                    "user": {
                        "id": URL_SAFE_NO_PAD.encode(user_handle(user_id, login_id)),
                        "name": login_id,
                        "displayName": if display_name.is_empty() { login_id } else { display_name }
                    },
                    "pubKeyCredParams": [
                        { "type": "public-key", "alg": COSE_ES256 },
                        { "type": "public-key", "alg": COSE_RS256 }
                    ],
                    "excludeCredentials": existing,
                    "authenticatorSelection": {
                        "userVerification": "preferred",
                        "residentKey": "preferred"
                    },
                    "attestation": "none",
                    "timeout": 60000
                }
            }),
        };

        self.transactions.insert(
            transaction_id.clone(),
            Transaction {
                id: transaction_id.clone(),
                ceremony,
                login_id: login_id.to_string(),
                user_id: user_id.to_string(),
                challenge,
                rp_id: rp_id.to_string(),
                created_at: now,
            },
        );
        (transaction_id, options.to_string())
    }

    /// Remove and return an open ceremony of the expected kind.
    pub fn take_transaction(
        &mut self,
        transaction_id: &str,
        ceremony: Ceremony,
    ) -> Result<Transaction, EmulatorError> {
        match self.transactions.remove(transaction_id) {
            Some(t) if t.ceremony == ceremony => Ok(t),
            Some(t) => {
                self.transactions.insert(transaction_id.to_string(), t);
                Err(EmulatorError::InvalidToken)
            }
            None => Err(EmulatorError::InvalidToken),
        }
    }

    /// Check a `navigator.credentials.create` response and store the passkey for
    /// `user_id` (the transaction's user, or the user a sign-up just created).
    pub fn finish_registration(
        &mut self,
        transaction: &Transaction,
        user_id: &str,
        response_json: &str,
    ) -> Result<(), EmulatorError> {
        let response: Value = serde_json::from_str(response_json)
            .map_err(|_| refused("the WebAuthn response is not JSON"))?;
        check_client_data(&response, transaction)?;

        let attestation = b64_field(&response, "/response/attestationObject")?;
        let attestation: ciborium::Value = ciborium::from_reader(attestation.as_slice())
            .map_err(|_| refused("the attestation object is not CBOR"))?;
        let auth_data = cbor_map_get(&attestation, "authData")
            .and_then(|v| v.as_bytes().cloned())
            .ok_or_else(|| refused("the attestation object has no authData"))?;

        let parsed = parse_auth_data(&auth_data, &transaction.rp_id)?;
        let (credential_id, key) = parsed
            .attested
            .ok_or_else(|| refused("the authenticator data has no credential"))?;

        let creds = self.credentials.entry(user_id.to_string()).or_default();
        creds.retain(|c| c.id != credential_id);
        creds.push(Credential {
            id: credential_id,
            user_id: user_id.to_string(),
            key,
            sign_count: parsed.sign_count,
        });
        Ok(())
    }

    /// Check a `navigator.credentials.get` response against the passkeys of the
    /// transaction's user, and return that user's id.
    pub fn finish_assertion(
        &mut self,
        transaction: &Transaction,
        response_json: &str,
    ) -> Result<String, EmulatorError> {
        let response: Value = serde_json::from_str(response_json)
            .map_err(|_| refused("the WebAuthn response is not JSON"))?;
        let client_data = check_client_data(&response, transaction)?;

        let credential_id = b64_field(&response, "/rawId")?;
        let credential = self
            .credentials
            .get_mut(&transaction.user_id)
            .and_then(|creds| creds.iter_mut().find(|c| c.id == credential_id))
            .ok_or_else(|| refused("the passkey is not registered for this user"))?;

        let auth_data = b64_field(&response, "/response/authenticatorData")?;
        let parsed = parse_auth_data(&auth_data, &transaction.rp_id)?;
        let signature = b64_field(&response, "/response/signature")?;

        let mut signed = auth_data.clone();
        signed.extend_from_slice(&Sha256::digest(&client_data));
        verify_signature(&credential.key, &signed, &signature)?;

        credential.sign_count = parsed.sign_count;
        Ok(credential.user_id.clone())
    }

    /// Drop every credential and open ceremony.
    pub fn reset(&mut self) {
        self.credentials.clear();
        self.transactions.clear();
    }
}

/// The user handle the browser stores with the passkey.
fn user_handle(user_id: &str, login_id: &str) -> Vec<u8> {
    if user_id.is_empty() {
        login_id.as_bytes().to_vec()
    } else {
        user_id.as_bytes().to_vec()
    }
}

fn refused(reason: &str) -> EmulatorError {
    EmulatorError::ValidationError(format!("WebAuthn: {reason}"))
}

fn b64_field(response: &Value, pointer: &str) -> Result<Vec<u8>, EmulatorError> {
    let text = response
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or_else(|| refused(&format!("the response has no {pointer}")))?;
    decode_b64url(text).ok_or_else(|| refused(&format!("{pointer} is not base64url")))
}

/// Descope's SDK sends base64url without padding; accept padded input too.
fn decode_b64url(text: &str) -> Option<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(text.trim_end_matches('=')).ok()
}

/// Check the client data: ceremony type and challenge. Returns its raw bytes.
fn check_client_data(
    response: &Value,
    transaction: &Transaction,
) -> Result<Vec<u8>, EmulatorError> {
    let raw = b64_field(response, "/response/clientDataJSON")?;
    let client_data: Value =
        serde_json::from_slice(&raw).map_err(|_| refused("clientDataJSON is not JSON"))?;

    if client_data["type"] != transaction.ceremony.client_data_type() {
        return Err(refused("the client data is for another ceremony"));
    }
    let challenge = client_data["challenge"]
        .as_str()
        .and_then(decode_b64url)
        .ok_or_else(|| refused("the client data has no challenge"))?;
    if challenge != transaction.challenge {
        return Err(refused("the challenge does not match"));
    }
    Ok(raw)
}

struct AuthData {
    sign_count: u32,
    attested: Option<(Vec<u8>, CredentialKey)>,
}

fn parse_auth_data(data: &[u8], rp_id: &str) -> Result<AuthData, EmulatorError> {
    if data.len() < 37 {
        return Err(refused("the authenticator data is too short"));
    }
    if data[..32] != Sha256::digest(rp_id.as_bytes())[..] {
        return Err(refused("the passkey belongs to another relying party"));
    }
    let flags = data[32];
    if flags & FLAG_USER_PRESENT == 0 {
        return Err(refused("the user was not present"));
    }
    let sign_count = u32::from_be_bytes([data[33], data[34], data[35], data[36]]);

    if flags & FLAG_ATTESTED_CREDENTIAL == 0 {
        return Ok(AuthData {
            sign_count,
            attested: None,
        });
    }
    // aaguid (16) + credential id length (2)
    let rest = &data[37..];
    if rest.len() < 18 {
        return Err(refused("the attested credential data is too short"));
    }
    let id_len = u16::from_be_bytes([rest[16], rest[17]]) as usize;
    let rest = &rest[18..];
    if rest.len() < id_len {
        return Err(refused("the credential id is cut short"));
    }
    let credential_id = rest[..id_len].to_vec();
    let cose: ciborium::Value = ciborium::from_reader(&rest[id_len..])
        .map_err(|_| refused("the credential public key is not CBOR"))?;
    Ok(AuthData {
        sign_count,
        attested: Some((credential_id, parse_cose_key(&cose)?)),
    })
}

fn cbor_map_get<'a>(map: &'a ciborium::Value, key: &str) -> Option<&'a ciborium::Value> {
    map.as_map()?
        .iter()
        .find(|(k, _)| k.as_text() == Some(key))
        .map(|(_, v)| v)
}

fn cose_get(map: &ciborium::Value, label: i64) -> Option<&ciborium::Value> {
    map.as_map()?
        .iter()
        .find(|(k, _)| {
            k.as_integer()
                .and_then(|i| i64::try_from(i).ok())
                .is_some_and(|i| i == label)
        })
        .map(|(_, v)| v)
}

fn cose_int(map: &ciborium::Value, label: i64) -> Option<i64> {
    cose_get(map, label)?
        .as_integer()
        .and_then(|i| i64::try_from(i).ok())
}

fn cose_bytes(map: &ciborium::Value, label: i64) -> Result<Vec<u8>, EmulatorError> {
    cose_get(map, label)
        .and_then(|v| v.as_bytes().cloned())
        .ok_or_else(|| refused("the credential public key is incomplete"))
}

fn parse_cose_key(cose: &ciborium::Value) -> Result<CredentialKey, EmulatorError> {
    match cose_int(cose, 3) {
        Some(COSE_ES256) => Ok(CredentialKey::Es256 {
            x: cose_bytes(cose, -2)?,
            y: cose_bytes(cose, -3)?,
        }),
        Some(COSE_RS256) => Ok(CredentialKey::Rs256 {
            n: cose_bytes(cose, -1)?,
            e: cose_bytes(cose, -2)?,
        }),
        _ => Err(refused(
            "the passkey uses an algorithm the emulator does not verify",
        )),
    }
}

fn verify_signature(
    key: &CredentialKey,
    signed: &[u8],
    signature: &[u8],
) -> Result<(), EmulatorError> {
    match key {
        CredentialKey::Es256 { x, y } => {
            use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
            let mut point = Vec::with_capacity(65);
            point.push(0x04);
            point.extend_from_slice(x);
            point.extend_from_slice(y);
            let key = VerifyingKey::from_sec1_bytes(&point)
                .map_err(|_| refused("the stored public key is not a P-256 point"))?;
            let signature =
                Signature::from_der(signature).map_err(|_| refused("the signature is not DER"))?;
            key.verify(signed, &signature)
                .map_err(|_| refused("the signature does not verify"))
        }
        CredentialKey::Rs256 { n, e } => {
            use rsa::pkcs1v15::{Signature, VerifyingKey};
            use rsa::signature::Verifier;
            use rsa::{BigUint, RsaPublicKey};
            let public = RsaPublicKey::new(BigUint::from_bytes_be(n), BigUint::from_bytes_be(e))
                .map_err(|_| refused("the stored public key is not an RSA key"))?;
            let key = VerifyingKey::<Sha256>::new(public);
            let signature = Signature::try_from(signature)
                .map_err(|_| refused("the signature is malformed"))?;
            key.verify(signed, &signature)
                .map_err(|_| refused("the signature does not verify"))
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub mod test_support {
    //! A software authenticator, so tests can run real ceremonies.

    use super::*;
    use p256::ecdsa::{signature::Signer, Signature, SigningKey};

    pub struct SoftAuthenticator {
        pub key: SigningKey,
        pub credential_id: Vec<u8>,
        pub sign_count: u32,
    }

    impl Default for SoftAuthenticator {
        fn default() -> Self {
            Self::new()
        }
    }

    impl SoftAuthenticator {
        pub fn new() -> Self {
            Self {
                key: SigningKey::random(&mut rand_core_compat::OsRng),
                credential_id: (0u8..16).collect(),
                sign_count: 0,
            }
        }

        fn client_data(kind: &str, options: &Value) -> Vec<u8> {
            json!({
                "type": kind,
                "challenge": options["publicKey"]["challenge"],
                "origin": "http://localhost:4200"
            })
            .to_string()
            .into_bytes()
        }

        fn auth_data(&mut self, rp_id: &str, attested: bool) -> Vec<u8> {
            self.sign_count += 1;
            let mut data = Sha256::digest(rp_id.as_bytes()).to_vec();
            data.push(
                FLAG_USER_PRESENT
                    | 0x04
                    | if attested {
                        FLAG_ATTESTED_CREDENTIAL
                    } else {
                        0
                    },
            );
            data.extend_from_slice(&self.sign_count.to_be_bytes());
            if attested {
                data.extend_from_slice(&[0u8; 16]);
                data.extend_from_slice(&(self.credential_id.len() as u16).to_be_bytes());
                data.extend_from_slice(&self.credential_id);
                let point = self.key.verifying_key().to_encoded_point(false);
                let cose = ciborium::Value::Map(vec![
                    (1.into(), 2.into()),
                    (3.into(), COSE_ES256.into()),
                    ((-1).into(), 1.into()),
                    (
                        (-2).into(),
                        ciborium::Value::Bytes(point.x().unwrap().to_vec()),
                    ),
                    (
                        (-3).into(),
                        ciborium::Value::Bytes(point.y().unwrap().to_vec()),
                    ),
                ]);
                ciborium::into_writer(&cose, &mut data).unwrap();
            }
            data
        }

        /// What the browser returns from `navigator.credentials.create(options)`.
        pub fn create(&mut self, options_json: &str) -> String {
            let options: Value = serde_json::from_str(options_json).unwrap();
            let rp_id = options["publicKey"]["rp"]["id"]
                .as_str()
                .unwrap()
                .to_string();
            let attestation = ciborium::Value::Map(vec![
                ("fmt".into(), "none".into()),
                ("attStmt".into(), ciborium::Value::Map(vec![])),
                (
                    "authData".into(),
                    ciborium::Value::Bytes(self.auth_data(&rp_id, true)),
                ),
            ]);
            let mut attestation_bytes = Vec::new();
            ciborium::into_writer(&attestation, &mut attestation_bytes).unwrap();
            let id = URL_SAFE_NO_PAD.encode(&self.credential_id);
            json!({
                "id": id,
                "rawId": id,
                "type": "public-key",
                "response": {
                    "attestationObject": URL_SAFE_NO_PAD.encode(attestation_bytes),
                    "clientDataJSON": URL_SAFE_NO_PAD.encode(Self::client_data("webauthn.create", &options))
                }
            })
            .to_string()
        }

        /// What the browser returns from `navigator.credentials.get(options)`.
        pub fn get(&mut self, options_json: &str) -> String {
            let options: Value = serde_json::from_str(options_json).unwrap();
            let rp_id = options["publicKey"]["rpId"].as_str().unwrap().to_string();
            let auth_data = self.auth_data(&rp_id, false);
            let client_data = Self::client_data("webauthn.get", &options);
            let mut signed = auth_data.clone();
            signed.extend_from_slice(&Sha256::digest(&client_data));
            let signature: Signature = self.key.sign(&signed);
            let id = URL_SAFE_NO_PAD.encode(&self.credential_id);
            json!({
                "id": id,
                "rawId": id,
                "type": "public-key",
                "response": {
                    "authenticatorData": URL_SAFE_NO_PAD.encode(auth_data),
                    "clientDataJSON": URL_SAFE_NO_PAD.encode(client_data),
                    "signature": URL_SAFE_NO_PAD.encode(signature.to_der().as_bytes()),
                    "userHandle": null
                }
            })
            .to_string()
        }
    }

    mod rand_core_compat {
        pub use p256::elliptic_curve::rand_core::OsRng;
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::SoftAuthenticator;
    use super::*;

    fn begin(store: &mut WebAuthnStore, ceremony: Ceremony, user_id: &str) -> (String, String) {
        store.begin(
            ceremony,
            "pat@example.com",
            user_id,
            "Pat",
            "localhost",
            1_000,
            format!("tx-{ceremony:?}"),
            vec![7u8; 32],
        )
    }

    #[test]
    fn a_registered_passkey_signs_its_user_in() {
        let mut store = WebAuthnStore::new();
        let mut authenticator = SoftAuthenticator::new();

        let (tx, options) = begin(&mut store, Ceremony::Update, "U1");
        let transaction = store.take_transaction(&tx, Ceremony::Update).unwrap();
        store
            .finish_registration(&transaction, "U1", &authenticator.create(&options))
            .unwrap();
        assert!(store.has_credentials("U1"));

        let (tx, options) = begin(&mut store, Ceremony::SignIn, "U1");
        let options_value: Value = serde_json::from_str(&options).unwrap();
        assert_eq!(
            options_value["publicKey"]["allowCredentials"][0]["id"],
            URL_SAFE_NO_PAD.encode(&authenticator.credential_id)
        );
        let transaction = store.take_transaction(&tx, Ceremony::SignIn).unwrap();
        let user_id = store
            .finish_assertion(&transaction, &authenticator.get(&options))
            .unwrap();

        assert_eq!(user_id, "U1");
    }

    #[test]
    fn a_signature_from_another_key_is_refused() {
        let mut store = WebAuthnStore::new();
        let mut registered = SoftAuthenticator::new();
        let (tx, options) = begin(&mut store, Ceremony::Update, "U1");
        let transaction = store.take_transaction(&tx, Ceremony::Update).unwrap();
        store
            .finish_registration(&transaction, "U1", &registered.create(&options))
            .unwrap();

        let mut impostor = SoftAuthenticator::new();
        let (tx, options) = begin(&mut store, Ceremony::SignIn, "U1");
        let transaction = store.take_transaction(&tx, Ceremony::SignIn).unwrap();

        assert!(store
            .finish_assertion(&transaction, &impostor.get(&options))
            .is_err());
    }

    #[test]
    fn a_response_to_another_challenge_is_refused() {
        let mut store = WebAuthnStore::new();
        let mut authenticator = SoftAuthenticator::new();
        let (_, options) = begin(&mut store, Ceremony::Update, "U1");
        let stale = authenticator.create(&options);

        let (tx, _) = store.begin(
            Ceremony::Update,
            "pat@example.com",
            "U1",
            "Pat",
            "localhost",
            1_000,
            "tx-2".into(),
            vec![9u8; 32],
        );
        let transaction = store.take_transaction(&tx, Ceremony::Update).unwrap();

        assert!(store
            .finish_registration(&transaction, "U1", &stale)
            .is_err());
        assert!(!store.has_credentials("U1"));
    }

    #[test]
    fn a_passkey_for_another_relying_party_is_refused() {
        let mut store = WebAuthnStore::new();
        let mut authenticator = SoftAuthenticator::new();
        let (tx, options) = begin(&mut store, Ceremony::Update, "U1");
        let other_rp = options.replace("\"id\":\"localhost\"", "\"id\":\"evil.test\"");
        let transaction = store.take_transaction(&tx, Ceremony::Update).unwrap();

        assert!(store
            .finish_registration(&transaction, "U1", &authenticator.create(&other_rp))
            .is_err());
    }

    #[test]
    fn a_transaction_finishes_only_its_own_ceremony_once() {
        let mut store = WebAuthnStore::new();
        let (tx, _) = begin(&mut store, Ceremony::SignIn, "U1");

        assert!(store.take_transaction(&tx, Ceremony::Update).is_err());
        assert!(store.take_transaction(&tx, Ceremony::SignIn).is_ok());
        assert!(store.take_transaction(&tx, Ceremony::SignIn).is_err());
    }
}
