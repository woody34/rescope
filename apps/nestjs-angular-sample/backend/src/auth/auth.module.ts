import {
  Module,
  Global,
} from '@nestjs/common';
import { DescopeService } from './descope.service';
import { AuthGuard } from './auth.guard';
import { RolesGuard } from './roles.guard';

@Global()
@Module({
  providers: [DescopeService, AuthGuard, RolesGuard],
  exports: [DescopeService, AuthGuard, RolesGuard],
})
export class AuthModule {}
