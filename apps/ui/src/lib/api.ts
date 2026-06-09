/**
 * lib/api.ts — typed fetch wrapper for the Descope emulator management API.
 * All requests use the emulator's origin as the base URL.
 */

const BASE = window.location.origin;

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: { "Content-Type": "application/json" },
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(
      err?.errorDescription || err?.error || `HTTP ${res.status}`,
    );
  }
  return res.json() as Promise<T>;
}

const get = <T>(path: string) => request<T>("GET", path);
const post = <T>(path: string, body?: unknown) =>
  request<T>("POST", path, body);
const put = <T>(path: string, body?: unknown) => request<T>("PUT", path, body);

// ─── Types ────────────────────────────────────────────────────────────────────

export interface Permission {
  id: string;
  name: string;
  description: string;
}

export interface Role {
  id: string;
  name: string;
  description: string;
  permissionNames: string[];
  isDefault: boolean;
  isHidden: boolean;
}

export interface OtpConfig {
  enabled: boolean;
  expirationSeconds: number;
}

export interface AuthMethodConfig {
  otp: OtpConfig;
  magicLink: { enabled: boolean; expirationSeconds: number };
  password: {
    enabled: boolean;
    minLength: number;
    requireUppercase: boolean;
    requireNumbers: boolean;
    requireSymbols: boolean;
  };
  totp: { enabled: boolean };
  passkeys: { enabled: boolean };
  oauth: Record<string, { enabled: boolean; clientId?: string }>;
  sso: { enabled: boolean };
  enchantedLink: { enabled: boolean; expirationSeconds: number };
  embeddedLink: { enabled: boolean; expirationSeconds: number };
  notp: { enabled: boolean; expirationSeconds: number };
}

export interface JwtClaim {
  key: string;
  claimType: "dynamic" | "static";
  value: string;
}

export interface JwtTemplate {
  id: string;
  name: string;
  authorizationClaimsFormat: "flat" | "nested";
  customClaims: JwtClaim[];
  subjectOverride?: string;
  includeJti: boolean;
  isActive: boolean;
}

export interface Connector {
  id: string;
  name: string;
  type: "genericHttp" | "smtp" | "sendgrid" | "twilio" | "slack" | "segment";
  config: Record<string, unknown>;
}

export interface CustomAttribute {
  name: string;
  machineName: string;
  attributeType: "text" | "number" | "boolean" | "datetime";
  permissions: "admin" | "memberRead" | "memberWrite" | "all";
}

export interface AccessKey {
  id: string;
  name: string;
  expiresAt?: number;
  permittedIps: string[];
  roleNames: string[];
  keyTenants: { tenantId: string; roleNames: string[] }[];
  status: "active" | "disabled";
}

export interface User {
  userId: string;
  loginIds: string[];
  email?: string;
  phone?: string;
  name?: string;
  givenName?: string;
  familyName?: string;
  status: string;
  verifiedEmail: boolean;
  verifiedPhone: boolean;
  roleNames: string[];
  userTenants: { tenantId: string; tenantName: string; roleNames: string[] }[];
  customAttributes: Record<string, unknown>;
  createdTime: number;
  lastLogin?: number;
}

export interface Tenant {
  id: string;
  name: string;
  domains: string[];
  authType: "none" | "saml" | "oidc";
  samlConfig?: { metadataUrl?: string; entityId?: string; acsUrl?: string };
  oidcConfig?: { discoveryUrl?: string; clientId?: string };
}

export interface EmulatorSnapshot {
  users: User[];
  tenants: Tenant[];
  permissions: Permission[];
  roles: Role[];
  authMethodConfig?: AuthMethodConfig;
  jwtTemplates: JwtTemplate[];
  connectors: Connector[];
  customAttributes: CustomAttribute[];
  accessKeys: AccessKey[];
}

export interface IdpEmulator {
  id: string;
  protocol: "oidc" | "saml";
  displayName: string;
  tenantId: string;
  attributeMapping: Record<string, string>;
}

// ─── Permissions ──────────────────────────────────────────────────────────────

export const api = {
  permissions: {
    list: () =>
      get<{ permissions: Permission[] }>("/v1/mgmt/authz/permission/all"),
    create: (name: string, description: string) =>
      post<{ permission: Permission }>("/v1/mgmt/authz/permission", {
        name,
        description,
      }),
    update: (name: string, newName?: string, description?: string) =>
      post("/v1/mgmt/authz/permission/update", { name, newName, description }),
    delete: (name: string) =>
      post("/v1/mgmt/authz/permission/delete", { name }),
  },

  roles: {
    list: () => get<{ roles: Role[] }>("/v1/mgmt/authz/role/all"),
    create: (name: string, description: string, permissionNames: string[]) =>
      post<{ role: Role }>("/v1/mgmt/authz/role", {
        name,
        description,
        permissionNames,
      }),
    update: (
      name: string,
      newName?: string,
      description?: string,
      permissionNames?: string[],
    ) =>
      post("/v1/mgmt/authz/role/update", {
        name,
        newName,
        description,
        permissionNames,
      }),
    delete: (name: string) => post("/v1/mgmt/authz/role/delete", { name }),
  },

  authMethods: {
    get: () =>
      get<{ authMethods: AuthMethodConfig }>("/v1/mgmt/config/auth-methods"),
    update: (config: Partial<AuthMethodConfig>) =>
      put("/v1/mgmt/config/auth-methods", config),
  },

  jwtTemplates: {
    list: () => get<{ templates: JwtTemplate[] }>("/v1/mgmt/jwt/template/all"),
    create: (template: Omit<JwtTemplate, "id">) =>
      post<{ template: JwtTemplate }>("/v1/mgmt/jwt/template", {
        ...template,
        id: "",
      }),
    update: (id: string, template: Partial<JwtTemplate>) =>
      post("/v1/mgmt/jwt/template/update", { id, ...template }),
    delete: (id: string) => post("/v1/mgmt/jwt/template/delete", { id }),
    setActive: (id: string) => post("/v1/mgmt/jwt/template/set-active", { id }),
    getActive: () =>
      get<{ template: JwtTemplate | null }>("/v1/mgmt/jwt/template/active"),
  },

  connectors: {
    list: () => get<{ connectors: Connector[] }>("/v1/mgmt/connector/all"),
    create: (
      name: string,
      type: Connector["type"],
      config: Record<string, unknown>,
    ) =>
      post<{ connector: Connector }>("/v1/mgmt/connector", {
        name,
        type,
        config,
      }),
    update: (id: string, name?: string, config?: Record<string, unknown>) =>
      post("/v1/mgmt/connector/update", { id, name, config }),
    delete: (id: string) => post("/v1/mgmt/connector/delete", { id }),
  },

  customAttributes: {
    list: () =>
      get<{ attributes: CustomAttribute[] }>("/v1/mgmt/user/attribute/all"),
    create: (attr: Omit<CustomAttribute, "id">) =>
      post<{ attribute: CustomAttribute }>("/v1/mgmt/user/attribute", attr),
    delete: (machineName: string) =>
      post("/v1/mgmt/user/attribute/delete", { machineName }),
  },

  accessKeys: {
    list: () => get<{ keys: AccessKey[] }>("/v1/mgmt/accesskey/all"),
    create: (params: {
      name: string;
      expireTime?: number;
      permittedIps?: string[];
      roleNames?: string[];
      keyTenants?: { tenantId: string; roleNames: string[] }[];
    }) =>
      post<{ key: AccessKey; cleartext: string }>("/v1/mgmt/accesskey", params),
    update: (id: string, params: Partial<AccessKey>) =>
      post("/v1/mgmt/accesskey/update", { id, ...params }),
    delete: (id: string) => post("/v1/mgmt/accesskey/delete", { id }),
    disable: (id: string) => post("/v1/mgmt/accesskey/disable", { id }),
  },

  users: {
    search: (params?: {
      emails?: string[];
      limit?: number;
      withTestUser?: boolean;
      statuses?: string[];
      tenantIds?: string[];
      roleNames?: string[];
      text?: string;
      loginIds?: string[];
    }) => post<{ users: User[] }>("/v2/mgmt/user/search", params ?? {}),
    create: (params: Partial<User> & { loginIds: string[] }) =>
      post<{ user: User }>("/v1/mgmt/user/create", params),
    update: (loginId: string, params: Partial<User>) =>
      post<{ user: User }>("/v1/mgmt/user/update", { loginId, ...params }),
    patch: (loginId: string, params: Partial<User>) =>
      post<{ user: User }>("/v1/mgmt/user/patch", { loginId, ...params }),
    setStatus: (loginId: string, status: "enabled" | "disabled") =>
      post("/v1/mgmt/user/update/status", { loginId, status }),
    addTenant: (loginId: string, tenantId: string, roleNames?: string[]) =>
      post("/v1/mgmt/user/tenant/add", { loginId, tenantId, roleNames }),
    removeTenant: (loginId: string, tenantId: string) =>
      post("/v1/mgmt/user/tenant/remove", { loginId, tenantId }),
    setTenantRoles: (
      loginId: string,
      tenantId: string,
      roleNames: string[],
    ) =>
      post("/v1/mgmt/user/tenant/setRole", {
        loginId,
        tenantId,
        roleNames,
      }),
    delete: (loginId: string) => post("/v1/mgmt/user/delete", { loginId }),
    setPassword: (loginId: string, password: string) =>
      post("/v1/mgmt/user/password/set/active", { loginId, password }),
  },

  tenants: {
    list: () => get<{ tenants: Tenant[] }>("/v1/mgmt/tenant/all"),
    create: (id: string, name: string) =>
      post<{ tenant: Tenant }>("/v1/mgmt/tenant/create", { id, name }),
    update: (id: string, name?: string, domains?: string[]) =>
      post("/v1/mgmt/tenant/update", {
        id,
        name,
        selfProvisioningDomains: domains,
      }),
    delete: (id: string) => post("/v1/mgmt/tenant/delete", { id }),
  },

  snapshot: {
    export: () => get<EmulatorSnapshot>("/emulator/snapshot"),
    import: (snap: EmulatorSnapshot) => post("/emulator/snapshot", snap),
    otps: () => get<{ otps: Record<string, string> }>("/emulator/otps"),
  },

  emulator: {
    reset: () => post("/emulator/reset"),
    health: () => get<{ ok: boolean }>("/health"),
  },

  idps: {
    list: () => get<{ idps: IdpEmulator[] }>("/v1/mgmt/idp/all"),
    create: (params: { protocol: string; displayName: string; tenantId: string; attributeMapping: Record<string, string> }) =>
      post<{ idp: IdpEmulator }>("/v1/mgmt/idp", params),
    update: (id: string, params: Partial<IdpEmulator>) =>
      post("/v1/mgmt/idp/update", { id, ...params }),
    delete: (id: string) => post("/v1/mgmt/idp/delete", { id }),
  },
};
