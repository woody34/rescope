import { SetMetadata } from '@nestjs/common';
import { IS_PUBLIC_KEY } from './auth.guard';
import { ROLES_KEY } from './roles.guard';

export const Public = () => SetMetadata(IS_PUBLIC_KEY, true);
export const Roles = (...roles: string[]) => SetMetadata(ROLES_KEY, roles);
