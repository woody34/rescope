## ADDED Requirements

### Requirement: Password sign-up

`POST /v1/auth/password/signup` SHALL accept `{ loginId, password, user? }`. If no user with `loginId` exists, it SHALL create the user, hash and store the password, and return an `AuthenticationResponse` with `sessionJwt`, `refreshJwt`, and `user`.

#### Scenario: New user signs up successfully

- **WHEN** `POST /v1/auth/password/signup` is called with a unique loginId and password
- **THEN** the response is HTTP 200 with `sessionJwt`, `refreshJwt`, and `user` fields

#### Scenario: Sign-up with existing loginId is rejected

- **WHEN** `POST /v1/auth/password/signup` is called with a loginId that already exists
- **THEN** the response is HTTP 400 with `{ "ok": false, "errorCode": "E062108" }`

---

### Requirement: Password sign-in

`POST /v1/auth/password/signin` SHALL accept `{ loginId, password }`, verify the password against the stored bcrypt hash, and return an `AuthenticationResponse`.

#### Scenario: Valid credentials return session tokens

- **WHEN** `POST /v1/auth/password/signin` is called with correct loginId and password
- **THEN** the response is HTTP 200 with valid `sessionJwt` and `refreshJwt`

#### Scenario: Wrong password is rejected

- **WHEN** `POST /v1/auth/password/signin` is called with correct loginId but wrong password
- **THEN** the response is HTTP 400 with `{ "ok": false, "errorCode": "E011003" }`

#### Scenario: Unknown loginId is rejected

- **WHEN** `POST /v1/auth/password/signin` is called with a loginId that does not exist
- **THEN** the response is HTTP 400 with `{ "ok": false, "errorCode": "E062108" }`

---

### Requirement: Password replace

`POST /v1/auth/password/replace` SHALL accept `{ loginId, oldPassword, newPassword }`, verify the old password, replace the stored hash with a hash of `newPassword`, and return an `AuthenticationResponse` with new session tokens.

#### Scenario: Password is replaced successfully

- **WHEN** `POST /v1/auth/password/replace` is called with correct old password
- **THEN** the response is HTTP 200 and subsequent sign-in with `newPassword` succeeds

#### Scenario: Wrong old password is rejected

- **WHEN** `POST /v1/auth/password/replace` is called with incorrect old password
- **THEN** the response is HTTP 400 with `{ "ok": false }`

---

### Requirement: Password reset send (no-op)

`POST /v1/auth/password/reset` SHALL verify the user exists and return `{ "ok": true }`. No email is sent. The response SHALL include a `maskedEmail` field matching the Descope `PasswordResetResponse` shape for SDK compatibility.

#### Scenario: Reset returns success for existing user

- **WHEN** `POST /v1/auth/password/reset` is called for an existing user
- **THEN** the response is HTTP 200 with `{ "ok": true, "maskedEmail": "..." }`

#### Scenario: Reset fails for unknown user

- **WHEN** `POST /v1/auth/password/reset` is called for a non-existent loginId
- **THEN** the response is HTTP 400 with `{ "ok": false }`

---

### Requirement: Password update from reset token

`POST /v1/auth/password/update` SHALL accept `{ loginId, newPassword, token }`, consume the reset token from the token store, hash `newPassword`, and store it for the user.

#### Scenario: Password is updated with valid reset token

- **WHEN** `POST /v1/auth/password/update` is called with a valid reset token
- **THEN** the response is HTTP 200 and subsequent sign-in with `newPassword` succeeds

#### Scenario: Invalid or already-used reset token is rejected

- **WHEN** `POST /v1/auth/password/update` is called with an invalid token
- **THEN** the response is HTTP 400 with `{ "ok": false }`

---

### Requirement: Magic link sign-in initiation

`POST /v1/auth/magiclink/signin/email` SHALL accept `{ loginId, URI }`, verify the user exists, generate a magic link token, store it in the token store, and return `{ "ok": true, "maskedEmail": "..." }`. No email is sent.

#### Scenario: Magic link is initiated for existing user

- **WHEN** `POST /v1/auth/magiclink/signin/email` is called for an existing user
- **THEN** the response is HTTP 200 with `{ "ok": true }` and a token is stored in the token store

#### Scenario: Magic link initiation fails for unknown user

- **WHEN** `POST /v1/auth/magiclink/signin/email` is called for a non-existent loginId
- **THEN** the response is HTTP 400 with `{ "ok": false }`

---

### Requirement: Magic link verification

`POST /v1/auth/magiclink/verify` SHALL accept `{ token }`, consume the token from the token store, find the associated user, and return an `AuthenticationResponse`. Tokens are single-use.

#### Scenario: Valid magic link token returns session

- **WHEN** `POST /v1/auth/magiclink/verify` is called with a valid token
- **THEN** the response is HTTP 200 with `sessionJwt`, `refreshJwt`, and `user`

#### Scenario: Magic link token is single-use

- **WHEN** the same magic link token is used twice
- **THEN** the second call returns HTTP 400 with `{ "ok": false }`

#### Scenario: Invalid token is rejected

- **WHEN** `POST /v1/auth/magiclink/verify` is called with a token not in the store
- **THEN** the response is HTTP 400 with `{ "ok": false }`

---

### Requirement: Magic link email update

`POST /v1/auth/magiclink/update/email` SHALL accept `{ loginId, email, token? }`, verify the refresh token if provided, and return `{ "ok": true }`. No email is sent.

#### Scenario: Email update magic link returns success

- **WHEN** `POST /v1/auth/magiclink/update/email` is called with a valid refresh token
- **THEN** the response is HTTP 200 with `{ "ok": true }`

---

### Requirement: SAML / SSO start

`POST /v1/auth/saml/start` SHALL accept `{ tenant, redirectUrl }` where `tenant` is an email or tenant ID. It SHALL look up the user, verify they belong to a tenant with `authType: "saml"` or `"oidc"`, generate a SAML auth code, store it, and return `{ "url": "{redirectUrl}?code={authCode}" }`. No IdP redirect occurs — the code is returned directly to the application.

#### Scenario: SAML start for SSO-configured user returns URL with code

- **WHEN** `POST /v1/auth/saml/start` is called for a user in a SAML tenant
- **THEN** the response is HTTP 200 with `{ "url": "...?code=<hex64>" }`

#### Scenario: SAML start for user without SSO tenant is rejected

- **WHEN** `POST /v1/auth/saml/start` is called for a user not in a SAML/OIDC tenant
- **THEN** the response is HTTP 400 with `{ "ok": false }`

---

### Requirement: SAML / SSO exchange

`POST /v1/auth/saml/exchange` SHALL accept `{ code }`, consume the SAML auth code, find the associated user, and return an `AuthenticationResponse`. Codes are single-use.

#### Scenario: Valid SAML code returns session tokens

- **WHEN** `POST /v1/auth/saml/exchange` is called with a valid SAML auth code
- **THEN** the response is HTTP 200 with `sessionJwt`, `refreshJwt`, and `user`

#### Scenario: SAML code is single-use

- **WHEN** the same SAML code is exchanged twice
- **THEN** the second call returns HTTP 400 with `{ "ok": false }`

---

### Requirement: OTP phone update (no-op SMS)

`POST /v1/auth/otp/update/phone/sms` SHALL accept `{ loginId, phone, options? }`, update the user's `phone` field, and optionally add the phone to `loginIds` when `options.addToLoginIDs` is true. No SMS is sent. Returns `{ "ok": true }`.

#### Scenario: Phone is updated on the user record

- **WHEN** `POST /v1/auth/otp/update/phone/sms` is called with a valid loginId and phone
- **THEN** the user's `phone` field is updated and the response is HTTP 200 `{ "ok": true }`

#### Scenario: Phone added to loginIds when flag is set

- **WHEN** the request includes `options: { addToLoginIDs: true }`
- **THEN** the phone number is added to the user's `loginIds` array

---

### Requirement: Authentication response shape

All successful authentication endpoints (`password/signup`, `password/signin`, `password/replace`, `magiclink/verify`, `saml/exchange`, `refresh`) SHALL return a response matching the Descope `JWTResponse` shape:

```json
{
  "sessionJwt": "<signed JWT>",
  "refreshJwt": "<signed JWT>",
  "user": { /* UserResponse */ },
  "cookieDomain": "",
  "cookiePath": "/",
  "cookieMaxAge": 3600,
  "cookieExpiration": <unix timestamp>,
  "firstSeen": false
}
```

#### Scenario: Auth response contains all required fields

- **WHEN** any successful authentication endpoint is called
- **THEN** the response JSON contains `sessionJwt`, `refreshJwt`, `user`, `cookiePath`, `cookieMaxAge`, and `cookieExpiration`
