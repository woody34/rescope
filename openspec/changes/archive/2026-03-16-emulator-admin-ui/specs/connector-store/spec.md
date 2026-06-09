## ADDED Requirements

### Requirement: Connectors are stored with type and configuration
The emulator SHALL maintain a `ConnectorStore` with connector entities (`id`, `name`, `type`, `config: JsonObject`). Supported types: `generic-http`, `smtp`, `twilio`, `sendgrid`, `aws-ses`, `slack`, `datadog`, `audit-webhook`, `recaptcha`. Management API: `POST /v1/mgmt/connector`, `GET /v1/mgmt/connector/all`, `POST /v1/mgmt/connector/update`, `DELETE /v1/mgmt/connector/delete`.

#### Scenario: Create and list a generic HTTP connector
- **WHEN** `POST /v1/mgmt/connector` is called with `{name: "my-webhook", type: "generic-http", config: {baseUrl: "https://example.com", authType: "bearer", bearerToken: "secret"}}`
- **THEN** response is `200 OK` with the connector `id`
- **WHEN** `GET /v1/mgmt/connector/all` is called
- **THEN** the connector appears in the list

### Requirement: Auth method configs reference connectors by ID
Auth method config fields that reference a connector (e.g. `otp.email_connector`, `password.reset_connector`) SHALL store the connector ID. When a connector-dependent auth action is triggered (OTP issue, password reset email) the emulator SHALL look up the connector and invoke it.

#### Scenario: Connector is invoked when OTP is issued (passthrough mode)
- **WHEN** an OTP is issued for a user with email
- **WHEN** `otp.email_connector` is set to a valid connector ID
- **THEN** the emulator makes an outbound HTTP call to the connector's `baseUrl` (in passthrough mode, fire-and-forget)
- **THEN** the OTP code is still stored and retrievable via `GET /emulator/otp/:loginId`

#### Scenario: In passthrough mode, connector failure does not block auth
- **WHEN** `otp.email_connector` references a connector with an unreachable `baseUrl`
- **WHEN** an OTP sign-in is initiated
- **THEN** the flow succeeds; the OTP code is stored even if the HTTP call fails
- **THEN** the failure is logged at `WARN` level

### Requirement: Connector secrets are included in snapshot export
- **WHEN** `GET /emulator/snapshot` is called
- **THEN** connector secrets (bearer tokens, passwords, etc.) ARE included in the snapshot (unlike real Descope which strips them)
- **NOTE**: This is emulator-specific behavior — enables full local round-trips without re-supplying secrets

#### Scenario: Connector round-trips through snapshot
- **WHEN** a connector is created with a bearer token
- **WHEN** the snapshot is exported and then imported to a fresh emulator
- **THEN** `GET /v1/mgmt/connector/all` returns the connector with its configuration intact
