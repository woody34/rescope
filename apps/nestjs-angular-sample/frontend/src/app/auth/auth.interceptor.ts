import { HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import { AuthService } from './auth.service';
import { environment } from '../../environments/environment';

/**
 * Attaches the session JWT as Authorization: Bearer to all /api/* requests.
 * Reads from AuthService signal first, falls back to localStorage 'DS' key
 * (same storage key used by Descope SDKs) — this ensures E2E-injected
 * tokens work even when AuthService.restore() rejects them due to expiry.
 */
export const authInterceptor: HttpInterceptorFn = (req, next) => {
  if (!req.url.includes(environment.apiBaseUrl) && !req.url.startsWith('/api')) {
    return next(req);
  }
  const auth = inject(AuthService);
  const jwt = auth.session()?.sessionJwt ?? localStorage.getItem('DS');
  if (!jwt) return next(req);

  return next(
    req.clone({ setHeaders: { Authorization: `Bearer ${jwt}` } }),
  );
};
