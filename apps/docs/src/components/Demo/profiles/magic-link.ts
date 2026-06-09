import type { DemoProfile } from './types';

export const magicLinkProfile: DemoProfile = {
  id: 'magic-link',
  label: 'Magic Link',
  icon: '🔗',
  description: 'Passwordless sign-in via a one-click link. The token appears here — no email needed.',
  seed: {
    users: [
      {
        loginId: 'alice@example.com',
        email: 'alice@example.com',
        name: 'Alice',
        roleNames: ['user'],
      },
      {
        loginId: 'bob@example.com',
        email: 'bob@example.com',
        name: 'Bob',
        roleNames: ['user'],
      },
    ],
    roles: [{ name: 'user', description: 'Standard user' }],
  },
  loginUI: {
    type: 'magic-link',
    prefillEmail: 'alice@example.com',
  },
};
