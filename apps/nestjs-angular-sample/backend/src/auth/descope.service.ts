import { Injectable, UnauthorizedException } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import DescopeClient from '@descope/node-sdk';

export interface DescopeToken {
  sub: string;
  iss: string;
  iat: number;
  exp: number;
  dct?: string;
  roles?: string[];
  tenants?: Record<string, { roles?: string[] }>;
  [key: string]: unknown;
}

@Injectable()
export class DescopeService {
  private readonly client: ReturnType<typeof DescopeClient>;
  private readonly baseUrl: string;

  constructor(private readonly config: ConfigService) {
    this.baseUrl = this.config.get<string>(
      'EMULATOR_BASE_URL',
      'http://localhost:4600',
    );
    this.client = DescopeClient({
      projectId: this.config.get<string>(
        'EMULATOR_PROJECT_ID',
        'emulator-project',
      ),
      managementKey: this.config.get<string>(
        'EMULATOR_MANAGEMENT_KEY',
        'emulator-key',
      ),
      baseUrl: this.baseUrl,
    });
  }

  async validateSession(jwt: string): Promise<DescopeToken> {
    const result = await this.client.validateSession(jwt);
    if (!result || !result.token) {
      throw new UnauthorizedException('Invalid or expired session token');
    }

    // Call the emulator's validate endpoint to check force-logout revocation.
    // The Node SDK only does JWKS-based cryptographic verification; it does not
    // check per-user revocation timestamps set by management force-logout.
    const emulatorRes = await fetch(`${this.baseUrl}/v1/auth/validate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ sessionJwt: jwt }),
    });
    if (!emulatorRes.ok) {
      throw new UnauthorizedException('Session revoked or expired');
    }

    return result.token as DescopeToken;
  }
}
