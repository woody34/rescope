export { passwordProfile } from './password';
export { otpProfile } from './otp';
export { magicLinkProfile } from './magic-link';
export { multiTenantProfile } from './multi-tenant';
export type { DemoProfile, DemoSeed, LoginUI, LoginUIType, SeedUser, SeedRole, SeedTenant } from './types';

import { passwordProfile } from './password';
import { otpProfile } from './otp';
import { magicLinkProfile } from './magic-link';
import { multiTenantProfile } from './multi-tenant';
import type { DemoProfile } from './types';

export const DEMO_PROFILES: DemoProfile[] = [
  passwordProfile,
  otpProfile,
  magicLinkProfile,
  multiTenantProfile,
];
