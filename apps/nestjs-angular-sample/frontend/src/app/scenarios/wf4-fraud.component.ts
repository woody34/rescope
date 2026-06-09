/**
 * WF4 — Fraud Prevention
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
  selector: 'app-wf4',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink, TodoTableComponent, JwtClaimsComponent],
  template: `
    <div class="scenario">
      <nav><a routerLink="/">← Home</a></nav>
      <h2>🛡️ WF4 — Fraud Prevention</h2>
      <p class="description">Account disable/re-enable and session revocation on security events.</p>

      @if (!auth.isAuthenticated()) {
        <div class="step-box">
          <h3>Sign in with password</h3>
          <div class="field"><label>Login ID</label><input [(ngModel)]="loginId" id="wf4-login-id" /></div>
          <div class="field"><label>Password</label><input [(ngModel)]="password" type="password" id="wf4-password" /></div>
          <button (click)="signIn()" id="wf4-signin-btn">Sign In</button>
          @if (error()) { <p class="error" id="wf4-error">{{ error() }}</p> }
        </div>
      } @else {
        <div class="authenticated-section">
          <div class="user-bar" id="wf4-user-bar">
            <span>Signed in as <strong>{{ auth.session()?.userId }}</strong></span>
            <span class="status-badge" [class.disabled]="accountDisabled()" id="wf4-account-status">
              {{ accountDisabled() ? '🔴 Disabled' : '🟢 Active' }}
            </span>
            <button (click)="logout()" id="wf4-logout-btn">Log out</button>
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
    .status-badge { padding: 0.25rem 0.6rem; border-radius: 12px; background: #efe; font-size: 0.85rem; }
    .status-badge.disabled { background: #fee; }
    .content-grid { display: grid; grid-template-columns: 1fr 300px; gap: 1rem; align-items: start; }
  `],
})
export class Wf4FraudComponent {
  protected readonly auth = inject(AuthService);
  loginId = ''; password = '';
  error = signal('');
  accountDisabled = signal(false);

  async signIn() {
    this.error.set('');
    const res = await fetch(`${environment.emulatorBaseUrl}/v1/auth/password/signin`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ loginId: this.loginId, password: this.password }),
    });
    if (res.status === 403) { this.error.set('Account is disabled.'); this.accountDisabled.set(true); return; }
    if (!res.ok) { this.error.set(`Sign-in failed (${res.status})`); return; }
    const data = await res.json() as { sessionJwt: string; refreshJwt: string };
    this.auth.setSession(data.sessionJwt, data.refreshJwt);
    this.accountDisabled.set(false);
  }

  async logout() { await this.auth.logout(); this.accountDisabled.set(false); }
}
