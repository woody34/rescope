## ADDED Requirements

### Requirement: HTTP requests are logged to stdout
The emulator SHALL log every incoming HTTP request to stdout with the method, path, response status code, and response time.

#### Scenario: Successful request is logged
- **WHEN** a client sends `POST /v1/auth/otp/signup/email` and receives a 200 response
- **THEN** stdout SHALL contain a log line including `POST`, `/v1/auth/otp/signup/email`, `200`, and a duration in milliseconds

#### Scenario: Failed request is logged
- **WHEN** a client sends `POST /v1/auth/password/signin` with invalid credentials and receives a 401 response
- **THEN** stdout SHALL contain a log line including `POST`, `/v1/auth/password/signin`, and `401`

#### Scenario: Request logging can be suppressed
- **WHEN** the environment variable `RUST_LOG` is set to `rescope=info,tower_http=off`
- **THEN** HTTP request log lines SHALL NOT appear in stdout, but other emulator log lines SHALL still appear
