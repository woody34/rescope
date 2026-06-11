# request-logging

Per-request terminal logging for the emulator HTTP server: categorization, log levels, line format, body capture/truncation, and env-var configuration.

## ADDED Requirements

### Requirement: One log line per completed request

The emulator SHALL emit exactly one log line per HTTP request, on completion, under the `rescope::http` tracing target. The line SHALL contain, in order: a category tag, the HTTP method, the request path (including any query string, as received), the response status code, and the latency in milliseconds.

#### Scenario: Successful API request

- **WHEN** a client sends `POST /v1/auth/otp/signin/email` and the handler responds `200`
- **THEN** exactly one line matching `[API] POST /v1/auth/otp/signin/email 200 <n>ms` is logged under target `rescope::http`

#### Scenario: Query string preserved

- **WHEN** a client sends `GET /emulator/otp/user@example.com?channel=sms`
- **THEN** the logged path is `/emulator/otp/user@example.com?channel=sms`

### Requirement: Requests are categorized by path prefix

The emulator SHALL classify every request into exactly one category from its path: `[API]` for paths starting with `/v1/`, `/v2/keys/`, `/.well-known/`, or `/oauth/`; `[EMU]` for paths starting with `/emulator/` and for `/health`; `[DOCS]` for `/docs` and `/openapi.json`; `[UI]` for all other paths.

#### Scenario: Descope API surface is API

- **WHEN** requests arrive for `/v1/auth/password/signin`, `/v2/keys/emulator-project`, `/.well-known/jwks.json`
- **THEN** each is logged with the `[API]` tag

#### Scenario: Emulator escape hatches are EMU

- **WHEN** requests arrive for `/emulator/reset` and `/health`
- **THEN** each is logged with the `[EMU]` tag

#### Scenario: Docs endpoints are DOCS

- **WHEN** requests arrive for `/docs` and `/openapi.json`
- **THEN** each is logged with the `[DOCS]` tag

#### Scenario: Everything else is UI

- **WHEN** requests arrive for `/` and `/assets/index-Bx2.js`
- **THEN** each is logged with the `[UI]` tag

### Requirement: Log level is determined by category

The emulator SHALL log `[API]` and `[EMU]` requests at INFO level and `[UI]` and `[DOCS]` requests at DEBUG level, with one exception: `/health` SHALL log at DEBUG level. Consequently, with the default env filter (`rescope=info`), only API and emulator traffic (excluding `/health`) is visible; `RUST_LOG=rescope=debug` reveals all categories.

#### Scenario: API visible by default

- **WHEN** the emulator runs with its default env filter and serves `POST /v1/auth/password/signin`
- **THEN** the request line is emitted at INFO and appears in output

#### Scenario: UI hidden by default

- **WHEN** the emulator runs with its default env filter and serves `GET /assets/app.js`
- **THEN** the request line is emitted at DEBUG and does not appear in output

#### Scenario: Health checks do not drum at INFO

- **WHEN** a test harness polls `GET /health`
- **THEN** the request line is emitted at DEBUG level despite carrying the `[EMU]` tag

### Requirement: Request bodies are captured for API and EMU requests

For `[API]` and `[EMU]` requests whose `Content-Type` is JSON, form-urlencoded, or `text/*`, the emulator SHALL append the request body to the log line as ` body=<content>`, collapsed to a single line and truncated to the configured cap (default 256 bytes) with a trailing `…` marker when truncated. Bodies with other content types (including binary and multipart) SHALL NOT be logged. `[UI]` and `[DOCS]` request bodies SHALL NOT be logged.

#### Scenario: JSON body appears inline

- **WHEN** a client sends `POST /v1/auth/otp/verify/email` with body `{"loginId":"x@y.com","code":"123456"}`
- **THEN** the log line contains `body={"loginId":"x@y.com","code":"123456"}`

#### Scenario: Long body is truncated

- **WHEN** a client sends an API request with a 10 KiB JSON body
- **THEN** the logged `body=` value is at most the configured cap in length and ends with `…`

#### Scenario: Multiline body collapses to one line

- **WHEN** a client sends pretty-printed JSON containing newlines and indentation
- **THEN** the logged `body=` value contains no newlines

#### Scenario: Binary body is not logged

- **WHEN** a client sends an API request with `Content-Type: application/octet-stream`
- **THEN** the log line contains no `body=` segment

### Requirement: Response bodies are logged only for error responses

The emulator SHALL append the response body as ` resp=<content>` only when the response status is 400 or greater, subject to the same content-type, single-line, and truncation rules as request bodies. Successful responses SHALL NOT have their bodies captured or logged.

#### Scenario: Error response body shown

- **WHEN** a request to `POST /v1/auth/otp/verify/email` yields `401` with body `{"errorCode":"E061102"}`
- **THEN** the log line contains `resp={"errorCode":"E061102"}`

#### Scenario: Success response body omitted

- **WHEN** a request yields `200` with a JSON body
- **THEN** the log line contains no `resp=` segment

### Requirement: Body capture does not alter request handling

Buffering a request body for logging SHALL be transparent to handlers: the handler SHALL receive the complete, byte-identical body. Buffering SHALL be bounded by the truncation cap plus a fixed slack; bodies exceeding the bound SHALL still reach the handler in full, with the log line indicating truncation or omitting the body.

#### Scenario: Handler sees full body

- **WHEN** a client sends a valid signup request with a body longer than the truncation cap
- **THEN** the handler processes it identically to a build without request logging (same status and response)

### Requirement: Authorization headers are never logged

The emulator SHALL NOT include the `Authorization` header value (or any other header values) in request log lines.

#### Scenario: Management call with bearer key

- **WHEN** a client sends `POST /v1/mgmt/user/create` with `Authorization: Bearer emulator-project:emulator-key`
- **THEN** no log line contains `emulator-key`

### Requirement: Body logging is configurable via environment variables

The emulator SHALL disable all body logging (request and response) when `RESCOPE_LOG_BODY=0` is set, and SHALL use `RESCOPE_LOG_BODY_MAX=<n>` as the truncation cap in bytes when set. The default cap SHALL be 256. Configuration SHALL be read once at startup, not per request.

#### Scenario: Bodies disabled

- **WHEN** the emulator starts with `RESCOPE_LOG_BODY=0` and serves an API request with a JSON body that yields a 400
- **THEN** the log line contains neither `body=` nor `resp=`

#### Scenario: Custom cap honored

- **WHEN** the emulator starts with `RESCOPE_LOG_BODY_MAX=1024` and receives a 600-byte JSON body
- **THEN** the full 600-byte body appears untruncated in the log line
