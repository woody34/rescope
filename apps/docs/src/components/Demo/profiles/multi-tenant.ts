import type { DemoProfile } from './types';

export const multiTenantProfile: DemoProfile = {
  id: 'multi-tenant',
  label: 'Multi-tenant',
  icon: '🏢',
  description: 'Organization-based auth with tenants, roles, and member bindings.',
  seed: {
    users: [
      {
        loginId: 'alice@acme.com',
        email: 'alice@acme.com',
        name: 'Alice (Acme)',
        password: 'Password123!',
        roleNames: ['admin'],
        userTenants: [{ tenantId: 'acme', roleNames: ['owner'] }],
      },
      {
        loginId: 'bob@acme.com',
        email: 'bob@acme.com',
        name: 'Bob (Acme)',
        password: 'Password123!',
        roleNames: ['member'],
        userTenants: [{ tenantId: 'acme', roleNames: ['member'] }],
      },
      {
        loginId: 'carol@globex.com',
        email: 'carol@globex.com',
        name: 'Carol (Globex)',
        password: 'Password123!',
        roleNames: ['admin'],
        userTenants: [{ tenantId: 'globex', roleNames: ['owner'] }],
      },
    ],
    roles: [
      { name: 'admin', description: 'Workspace admin' },
      { name: 'member', description: 'Workspace member' },
      { name: 'owner', description: 'Tenant owner' },
    ],
    tenants: [
      { id: 'acme', name: 'Acme Corp' },
      { id: 'globex', name: 'Globex Inc' },
    ],
  },
  loginUI: {
    type: 'password',
    prefillEmail: 'alice@acme.com',
    prefillPassword: 'Password123!',
  },
};
