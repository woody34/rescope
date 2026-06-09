## ADDED Requirements

### Requirement: Password policy endpoint returns permissive static policy
`GET /v1/auth/password/policy` SHALL return a static JSON object indicating an active policy with `minLength: 6` and `maxLength: 128`. No authentication is required.

#### Scenario: Policy endpoint returns valid response
- **WHEN** `GET /v1/auth/password/policy` is called
- **THEN** response is 200 with `{ "active": true, "minLength": 6, "maxLength": 128 }`
