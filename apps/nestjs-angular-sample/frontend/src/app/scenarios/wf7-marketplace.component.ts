/**
 * WF7 — Marketplace / Inbound App Integrations
 */
import { Component, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { AuthService } from '../auth/auth.service';
import { TodoTableComponent } from '../shared/todo-table.component';
import { JwtClaimsComponent } from '../shared/jwt-claims.component';
import { environment } from '../../environments/environment';

@Component({
  selector: 'app-wf7',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink, TodoTableComponent, JwtClaimsComponent],
  template: `
    <div class="scenario">
      <nav><a routerLink="/">← Home</a></nav>
      <h2>🛒 WF7 — Marketplace &amp; Integrations</h2>
      <p class="description">SaaS platform with tenant-scoped users and third-party service keys.</p>

      @if (!auth.isAuthenticated()) {
        <div class="step-box">
          <h3>Sign in (marketplace admin)</h3>
          <div class="field"><label>Login ID</label><input [(ngModel)]="loginId" id="wf7-login-id" /></div>
          <div class="field"><label>Password</label><input [(ngModel)]="password" type="password" id="wf7-password" /></div>
          <div class="field"><label>Tenant ID</label><input [(ngModel)]="tenantId" id="wf7-tenant-id" placeholder="marketplace-vendor-xxx" /></div>
          <button (click)="signInAndSelectTenant()" id="wf7-signin-btn">Sign In &amp; Select Tenant</button>
          @if (error()) { <p class="error" id="wf7-error">{{ error() }}</p> }
        </div>
      } @else {
        <div class="authenticated-section">
          <div class="user-bar" id="wf7-user-bar">
            <span>Tenant: <strong>{{ tenantId }}</strong> | User: {{ auth.session()?.userId }}</span>
            <button (click)="logout()" id="wf7-logout-btn">Log out</button>
          </div>
          <div class="content-grid">
            <app-todo-table></app-todo-table>
            <app-jwt-claims></app-jwt-claims>
          </div>
        </div>
      }
    </div>
  `,
  styles: [`
    .scenario { max-width: 900px; margin: 2rem auto; padding: 1rem; font-family: sans-serif; }
    nav { margin-bottom: 1rem; }
    .description { color: #666; margin-bottom: 1.5rem; }
    .step-box { max-width: 400px; padding: 1.5rem; border: 1px solid #ddd; border-radius: 8px; }
    .field { margin-bottom: 1rem; display: flex; flex-direction: column; gap: 0.25rem; }
    input { padding: 0.5rem; border: 1px solid #ccc; border-radius: 4px; }
    button { padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer; background: #0070f3; color: white; border: none; }
    .error { color: red; margin-top: 0.5rem; }
    .user-bar { display: flex; align-items: center; gap: 1rem; margin-bottom: 1rem; padding: 0.75rem; background: #eef; border-radius: 6px; }
    .content-grid { display: grid; grid-template-columns: 1fr 300px; gap: 1rem; align-items: start; }
  `],
})
export class Wf7MarketplaceComponent {
  protected readonly auth = inject(AuthService);
  loginId = ''; password = ''; tenantId = '';
  error = signal('');
  private refreshJwt = '';

  async signInAndSelectTenant() {
    this.error.set('');
    const signinRes = await fetch(`${environment.emulatorBaseUrl}/v1/auth/password/signin`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ loginId: this.loginId, password: this.password }),
    });
    if (!signinRes.ok) { this.error.set(`Sign-in failed (${signinRes.status})`); return; }
    const signinData = await signinRes.json() as { sessionJwt: string; refreshJwt: string };
    this.refreshJwt = signinData.refreshJwt;

    const selectRes = await fetch(`${environment.emulatorBaseUrl}/v1/auth/tenant/select`, {
      method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${this.refreshJwt}` },
      body: JSON.stringify({ tenant: this.tenantId }),
    });
    if (!selectRes.ok) { this.error.set(`Tenant select failed (${selectRes.status})`); return; }
    const selectData = await selectRes.json() as { sessionJwt: string };
    this.auth.setSession(selectData.sessionJwt, this.refreshJwt);
  }

  async logout() { await this.auth.logout(); }
}
