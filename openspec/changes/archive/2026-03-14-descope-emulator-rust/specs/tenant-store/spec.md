## ADDED Requirements

### Requirement: Tenant storage and retrieval

The tenant store SHALL store tenants keyed by `tenantId` and support retrieval of all tenants.

Each tenant record SHALL include: `id`, `name`, `selfProvisioningDomains`, `customAttributes`, `domains`, and `authType` (`"none"`, `"saml"`, or `"oidc"`).

#### Scenario: Tenant is stored and loadable

- **WHEN** a tenant is inserted into the store
- **THEN** `load_all()` includes that tenant in its results

#### Scenario: Load all returns all tenants

- **WHEN** multiple tenants are stored
- **THEN** `load_all()` returns all of them with correct fields

---

### Requirement: Domain-based tenant lookup for SAML

The tenant store SHALL support finding a tenant by email domain. Given an email address, the store SHALL find the tenant whose `domains` array contains the email's domain, where `authType` is `"saml"` or `"oidc"`.

#### Scenario: Tenant found by email domain

- **WHEN** `find_by_email("user@acme.com")` is called and a tenant has `domains: ["acme.com"]` with `authType: "saml"`
- **THEN** that tenant is returned

#### Scenario: No tenant for email domain returns error

- **WHEN** `find_by_email("user@unknown.com")` is called and no tenant has a matching domain
- **THEN** the operation returns `Err(TenantNotFound)`
