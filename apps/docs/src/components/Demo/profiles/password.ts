import type { DemoProfile } from './types';

export const passwordProfile: DemoProfile = {
  id: 'password',
  label: 'Email + Password',
  icon: '🔐',
  description: 'Classic email/password sign-in and sign-up flow.',
  seed: {
    users: [
      {
        loginId: 'alice@example.com',
        email: 'alice@example.com',
        name: 'Alice',
        password: 'Password123!',
        roleNames: ['admin'],
      },
      {
        loginId: 'bob@example.com',
        email: 'bob@example.com',
        name: 'Bob',
        password: 'Password123!',
        roleNames: ['member'],
      },
    ],
    roles: [
      { name: 'admin', description: 'Full access' },
      { name: 'member', description: 'Read-only access' },
    ],
  },
  loginUI: {
    type: 'password',
    prefillEmail: 'alice@example.com',
    prefillPassword: 'Password123!',
  },
};
