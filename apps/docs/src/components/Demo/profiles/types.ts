// DemoProfile type definitions

export interface SeedUser {
  loginId: string;
  email: string;
  name: string;
  password?: string;
  roleNames?: string[];
  userTenants?: { tenantId: string; roleNames: string[] }[];
}

export interface SeedRole {
  name: string;
  description?: string;
}

export interface SeedTenant {
  id: string;
  name: string;
}

export interface DemoSeed {
  users: SeedUser[];
  roles?: SeedRole[];
  tenants?: SeedTenant[];
}

export type LoginUIType = 'password' | 'otp' | 'magic-link';

export interface LoginUI {
  type: LoginUIType;
  prefillEmail?: string;
  prefillPassword?: string;
}

export interface DemoProfile {
  id: string;
  label: string;
  icon: string;
  description: string;
  seed: DemoSeed;
  loginUI: LoginUI;
}
