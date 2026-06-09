## ADDED Requirements

### Requirement: Enchanted link test token can be generated for test users
`POST /v1/mgmt/tests/generate/enchantedlink` is covered by `user-mgmt-extended/spec.md`. This spec covers the consumption side — the token generated is consumable via the existing `POST /v1/auth/magiclink/verify` endpoint.

#### Scenario: Token from generate/enchantedlink is verifiable
- **WHEN** the token returned by `generate/enchantedlink` is submitted to `magiclink/verify`
- **THEN** response returns valid `sessionJwt` and `refreshJwt` for the test user
