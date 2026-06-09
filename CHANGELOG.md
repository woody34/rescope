# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-03-16

### Added

- **Authentication**: Password signup/signin/replace/reset, OTP (email + SMS), Magic Link (email + SMS), SAML/SSO with exchange flow
- **Session Management**: JWT session + refresh tokens, token validation, `/v1/auth/me`, force logout with revocation
- **User Management**: Full CRUD, search with filters/pagination, bulk status updates, custom attributes, tenant association, role assignment
- **Tenant Management**: CRUD, SAML/OIDC configuration, domain mapping, search
- **Access Keys**: Create, list, disable, delete — machine-to-machine authentication with hashed storage
- **Permissions & Roles**: CRUD for both, role-permission assignment
- **JWT Templates**: CRUD with active template selection, custom claim injection
- **Connectors**: HTTP connector configuration, invocation for OTP/magic link/password reset delivery
- **Auth Method Configuration**: Enable/disable individual auth methods, configure connector associations
- **Snapshot/Import/Export**: Full state serialization to JSON for reproducible test setups
- **Seed Data**: Load initial state from JSON file on startup (`--seed` flag)
- **Admin UI**: React-based management console with dark theme, user/tenant/role/permission/access key management
- **Swagger Docs**: OpenAPI documentation served at `/docs`
- **Developer Experience**: Startup banner, request logging, console OTP/magic link codes
- **Login History**: `lastLogin` timestamp tracked across all auth methods
- **Bulk Operations**: Select, enable, disable, delete multiple users at once
- **Export**: Download user data as JSON or CSV

### Known Deviations from Descope

- OTP codes and magic link tokens are returned directly in API responses (no email/SMS delivery)
- Password reset tokens are returned in the response body
- Management API authentication is optional (requests without credentials succeed)
- All data is stored in-memory (no persistence across restarts unless using snapshots)
