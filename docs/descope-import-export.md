# Descope Import / Export — Capabilities Reference

> Research compiled: 2026-03-14
> Source: [Descope Docs — Managing Environments](https://docs.descope.com/managing-environments), [CLI Reference](https://docs.descope.com/cli/descope), [Terraform Provider](https://docs.descope.com/managing-environments/terraform)

---

## Overview

Descope provides three mechanisms for moving project configuration between environments:

1. **Project Snapshot** — ZIP-based export/import of all project config (primary mechanism)
2. **Clone** — Full project duplication including secrets (console or CLI)
3. **Terraform / Pulumi Providers** — IaC-based declarative management

The primary CLI tool is:

```bash
descope project snapshot export <projectId> [-p path]
descope project snapshot import <projectId> [-p path] [--secrets-input secrets.json]
descope project snapshot validate <projectId> [-p path] [--secrets-output missing.json]
```

---

## ✅ What IS Included in a Snapshot Export

| Category                          | Details                                                                    |
| --------------------------------- | -------------------------------------------------------------------------- |
| **Flows**                         | All visual authentication flows                                            |
| **Styles / Theme**                | UI customizations, screen builder layouts, widgets, conditional components |
| **Project Settings**              | Token expiration, inactivity timeouts, general config                      |
| **Authentication Method Configs** | OTP, magic link, password policies, SSO settings, social login configs     |
| **Email & Messaging Templates**   | Custom templates used during auth flows                                    |
| **Connector Configurations**      | HTTP, SMTP, Twilio, etc. — structure only, secrets stripped                |
| **OAuth Provider Configurations** | Social/OIDC provider setup — structure only, secrets stripped              |
| **Roles & Permissions**           | Authorization definitions                                                  |
| **Custom Attributes**             | User and tenant attribute schemas                                          |
| **Screens**                       | Screen builder components and conditional layouts                          |
| **JWT Templates**                 | Token structure and custom claims config                                   |

---

## ❌ What Is NOT Included in a Snapshot Export

| Category                                 | Reason / How to Handle                                                                                                    |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| **Connector & OAuth Secrets / API Keys** | Stripped and replaced with `PLACEHOLDER_VALUE`. Must be re-supplied manually via `--secrets-input` or by editing the ZIP. |
| **Users / User Data**                    | Not part of a snapshot. Requires the **User Management API** separately (`POST /v1/mgmt/user/import`).                    |
| **Tenants**                              | Separate entities managed via the Tenant Management API — not in the snapshot.                                            |
| **Access Keys & Management Keys**        | Runtime/admin credentials — never included.                                                                               |
| **Active Sessions / JWTs**               | Ephemeral — not backed up or transferred.                                                                                 |
| **Audit Logs**                           | Historical event data — must be streamed or retrieved via Audit API separately.                                           |
| **Password Hashes**                      | Cannot be retrieved from Descope backend APIs. Recommend triggering password resets on user migration.                    |

---

## Secret Placeholder Behavior

When a connector or OAuth provider has secrets configured, the export replaces them with `PLACEHOLDER_VALUE`:

```json
{
  "configuration": {
    "authentication": {
      "bearerToken": "PLACEHOLDER_VALUE",
      "method": "bearerToken"
    },
    "baseUrl": "https://example.com"
  }
}
```

**During import, the behavior depends on what you put in place of `PLACEHOLDER_VALUE`:**

| Value                             | Behavior on Import                              |
| --------------------------------- | ----------------------------------------------- |
| `"PLACEHOLDER_VALUE"` (unchanged) | Destination project keeps its existing secret   |
| `"my-actual-secret"` (replaced)   | Overrides the destination project's secret      |
| `null` or `""` (cleared)          | Clears the configured secret in the destination |

---

## Clone vs. Export Difference

|          | **Clone**                        | **Export / Import**       |
| -------- | -------------------------------- | ------------------------- |
| Secrets  | ✅ Copied                        | ❌ Stripped (PLACEHOLDER) |
| Users    | ✅ Copied                        | ❌ Not included           |
| Tenants  | ✅ Copied                        | ❌ Not included           |
| Use Case | Full duplication (dev → staging) | CI/CD config promotion    |

---

## Migrating Users Separately

Users must be handled outside the snapshot workflow:

- **Export**: Use `GET /v1/mgmt/user/search` or console CSV export
- **Import**: Use `POST /v1/mgmt/user/import` — supports pre-hashed passwords for select hash algorithms
- **Note**: Descope cannot retrieve raw password hashes for users it manages. Migrating users from Descope → Descope typically requires password resets.

---

## Emulator Implementation Notes

When implementing snapshot support in the emulator:

1. **ZIP structure** — The snapshot is a ZIP file with subdirectories per entity type (e.g., `flows/`, `connectors/`, `styles/`, `themes/`)
2. **API endpoints** — `/v1/mgmt/project/export` and `/v1/mgmt/project/import` accept/return ZIP payloads (not plain JSON)
3. **Secrets** — Must replace secret values with `PLACEHOLDER_VALUE` on export
4. **Users and tenants** are separate API surfaces — not part of the snapshot payload
5. **Selective import** — Flows and styles can be imported independently via the Flow & Style Management API (`/v1/mgmt/flow/import`, `/v1/mgmt/theme/import`)

---

## Related Descope API Endpoints

| Operation               | Endpoint                          |
| ----------------------- | --------------------------------- |
| Export project snapshot | `POST /v1/mgmt/project/export`    |
| Import project snapshot | `POST /v1/mgmt/project/import`    |
| Export flow             | `POST /v1/mgmt/flow/export`       |
| Import flow             | `POST /v1/mgmt/flow/import`       |
| Export theme            | `POST /v1/mgmt/theme/export`      |
| Import theme            | `POST /v1/mgmt/theme/import`      |
| Import users (batch)    | `POST /v1/mgmt/user/create/batch` |
| Search/export users     | `POST /v1/mgmt/user/search`       |
