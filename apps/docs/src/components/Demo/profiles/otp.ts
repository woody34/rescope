import type { DemoProfile } from './types';

export const otpProfile: DemoProfile = {
  id: 'otp',
  label: 'OTP (Email)',
  icon: '📧',
  description: 'One-time password sent to email. See the code appear instantly — no real email sent.',
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
    type: 'otp',
    prefillEmail: 'alice@example.com',
  },
};
