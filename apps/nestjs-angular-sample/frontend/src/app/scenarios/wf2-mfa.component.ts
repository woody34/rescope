/**
 * WF2 — MFA (Password + OTP second factor)
 *
 * Custom two-step form: password sign-in → OTP verify.
 * Uses the Descope SDK via direct fetch calls to the emulator.
 */
import { Component, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { AuthService } from '../auth/auth.service';
import { TodoTableComponent } from '../shared/todo-table.component';
import { JwtClaimsComponent } from '../shared/jwt-claims.component';
import { environment } from '../../environments/environment';

type Step = 'password' | 'otp' | 'done';

@Component({
  selector: 'app-wf2',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink, TodoTableComponent, JwtClaimsComponent],
  template: `
    <div class="scenario">
      <nav><a routerLink="/">← Home</a></nav>
      <h2>🔐 WF2 — MFA (Password + OTP Second Factor)</h2>
      <p class="description">Bolt on a second factor without replacing your CIAM stack.</p>

      @switch (step()) {
        @case ('password') {
          <div class="step-box">
            <h3>Step 1: Sign in with password</h3>
            <div class="field">
              <label>Login ID (email)</label>
              <input [(ngModel)]="loginId" id="wf2-login-id" placeholder="user@example.com" />
            </div>
            <div class="field">
              <label>Password</label>
              <input [(ngModel)]="password" type="password" id="wf2-password" placeholder="Password" />
            </div>
            <button (click)="signInWithPassword()" id="wf2-signin-btn">Sign In</button>
            @if (error()) { <p class="error" id="wf2-error">{{ error() }}</p> }
          </div>
        }
        @case ('otp') {
          <div class="step-box">
            <h3>Step 2: Enter OTP (second factor)</h3>
            <p>An OTP has been sent to <strong>{{ loginId }}</strong>.</p>
            <div class="field">
              <label>OTP Code</label>
              <input [(ngModel)]="otpCode" id="wf2-otp-code" placeholder="123456" maxlength="6" />
            </div>
            <button (click)="verifyOtp()" id="wf2-verify-btn">Verify OTP</button>
            @if (error()) { <p class="error" id="wf2-error">{{ error() }}</p> }
          </div>
        }
        @case ('done') {
          <div class="authenticated-section">
            <div class="user-bar" id="wf2-user-bar">
              <span>MFA complete — signed in as <strong>{{ auth.session()?.userId }}</strong></span>
              <button (click)="reset()" id="wf2-logout-btn">Log out</button>
            </div>
            <div class="content-grid">
              <app-todo-table></app-todo-table>
              <app-jwt-claims></app-jwt-claims>
            </div>
          </div>
        }
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
export class Wf2MfaComponent {
  protected readonly auth = inject(AuthService);
  step = signal<Step>('password');
  loginId = '';
  password = '';
  otpCode = '';
  error = signal('');
  private refreshJwt = '';

  async signInWithPassword() {
    this.error.set('');
    try {
      const res = await fetch(`${environment.emulatorBaseUrl}/v1/auth/password/signin`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ loginId: this.loginId, password: this.password }),
      });
      if (!res.ok) { this.error.set(`Sign-in failed (${res.status})`); return; }
      const data = await res.json() as { sessionJwt: string; refreshJwt: string };
      this.refreshJwt = data.refreshJwt;
      // Trigger OTP send for second factor
      await fetch(`${environment.emulatorBaseUrl}/v1/auth/otp/signup-in/email`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ loginId: this.loginId }),
      });
      this.step.set('otp');
    } catch (e) { this.error.set(`Network error: ${e}`); }
  }

  async verifyOtp() {
    this.error.set('');
    try {
      const res = await fetch(`${environment.emulatorBaseUrl}/v1/auth/otp/verify/email`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ loginId: this.loginId, code: this.otpCode }),
      });
      if (!res.ok) { this.error.set(`OTP verify failed (${res.status})`); return; }
      const data = await res.json() as { sessionJwt: string; refreshJwt: string };
      this.auth.setSession(data.sessionJwt, data.refreshJwt);
      this.step.set('done');
    } catch (e) { this.error.set(`Network error: ${e}`); }
  }

  async reset() {
    await this.auth.logout();
    this.step.set('password');
    this.loginId = '';
    this.password = '';
    this.otpCode = '';
  }
}
