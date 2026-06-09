import { Injectable, signal, computed } from '@angular/core';
import { environment } from '../../environments/environment';

export interface DescopeSession {
  sessionJwt: string;
  refreshJwt?: string;
  userId?: string;
}

function parseJwtPayload(jwt: string): Record<string, unknown> {
  try {
    return JSON.parse(atob(jwt.split('.')[1].replace(/-/g, '+').replace(/_/g, '/')));
  } catch {
    return {};
  }
}

@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly _session = signal<DescopeSession | null>(null);

  readonly session = this._session.asReadonly();
  readonly isAuthenticated = computed(() => !!this._session());
  readonly jwtClaims = computed(() => {
    const s = this._session();
    return s ? parseJwtPayload(s.sessionJwt) : null;
  });

  setSession(sessionJwt: string, refreshJwt?: string): void {
    const payload = parseJwtPayload(sessionJwt);
    this._session.set({
      sessionJwt,
      refreshJwt,
      userId: payload['sub'] as string,
    });
    localStorage.setItem('DS', sessionJwt);
    if (refreshJwt) localStorage.setItem('DSR', refreshJwt);
  }

  clearSession(): void {
    this._session.set(null);
    localStorage.removeItem('DS');
    localStorage.removeItem('DSR');
  }

  /** Called on app init to restore session from localStorage */
  restore(): void {
    const jwt = localStorage.getItem('DS');
    const rjwt = localStorage.getItem('DSR') ?? undefined;
    if (!jwt) return;
    const payload = parseJwtPayload(jwt);
    const exp = payload['exp'] as number | undefined;
    // Only restore into the signal if not clearly expired.
    // Don't wipe localStorage here — the backend validates freshness.
    // This allows E2E tests to inject tokens that may be near-expiry.
    if (!exp || Date.now() / 1000 < exp) {
      this._session.set({ sessionJwt: jwt, refreshJwt: rjwt, userId: payload['sub'] as string });
    }
    // If expired: leave localStorage alone, signal stays null.
    // The HTTP interceptor falls back to localStorage['DS'] for the token.
  }

  async logout(): Promise<void> {
    const refreshJwt = this._session()?.refreshJwt;
    if (refreshJwt) {
      try {
        await fetch(`${environment.emulatorBaseUrl}/v1/auth/logoutall`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${refreshJwt}`,
          },
        });
      } catch {
        // best effort
      }
    }
    this.clearSession();
  }
}
