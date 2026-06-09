/**
 * WF6 — CLI Auth (Magic Link as PKCE proxy)
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
  selector: 'app-wf6',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink, TodoTableComponent, JwtClaimsComponent],
  template: `
    <div class="scenario">
      <nav><a routerLink="/">← Home</a></nav>
      <h2>💻 WF6 — CLI Auth (Magic Link / Token Exchange)</h2>
      <p class="description">Developer tools (CLIs) that authenticate via browser and store a long-lived token.</p>

      @if (!auth.isAuthenticated()) {
        <div class="step-box">
          <h3>Paste magic link token</h3>
          <p class="hint">In a real CLI: user runs <code>myapp login</code>, browser opens, token arrives at callback. Paste the token here to simulate.</p>
          <div class="field"><label>Token</label><textarea [(ngModel)]="tokenInput" id="wf6-token-input" rows="3"></textarea></div>
          <button (click)="verifyToken()" id="wf6-verify-btn">Verify Token &amp; Sign In</button>
          @if (error()) { <p class="error" id="wf6-error">{{ error() }}</p> }
        </div>
      } @else {
        <div class="authenticated-section">
          <div class="user-bar" id="wf6-user-bar">
            <span>CLI session: <strong>{{ auth.session()?.userId }}</strong></span>
            <button (click)="logout()" id="wf6-logout-btn">Log out</button>
          </div>
          <div class="token-display" id="wf6-refresh-token">
            <strong>Stored refresh token (simulates ~/.config/myapp/token):</strong>
            <code>{{ auth.session()?.refreshJwt ?? 'N/A' }}</code>
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
    .step-box { max-width: 500px; padding: 1.5rem; border: 1px solid #ddd; border-radius: 8px; }
    .hint { font-size: 0.85rem; color: #666; margin-bottom: 1rem; }
    .field { margin-bottom: 1rem; display: flex; flex-direction: column; gap: 0.25rem; }
    textarea { padding: 0.5rem; border: 1px solid #ccc; border-radius: 4px; font-family: monospace; font-size: 0.8rem; }
    button { padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer; background: #0070f3; color: white; border: none; }
    .error { color: red; margin-top: 0.5rem; }
    .user-bar { display: flex; align-items: center; gap: 1rem; margin-bottom: 1rem; padding: 0.75rem; background: #eef; border-radius: 6px; }
    .token-display { background: #f5f5f5; border-radius: 6px; padding: 0.75rem; margin-bottom: 1rem; font-size: 0.8rem; }
    code { display: block; word-break: break-all; margin-top: 0.25rem; }
    .content-grid { display: grid; grid-template-columns: 1fr 300px; gap: 1rem; align-items: start; }
  `],
})
export class Wf6CliAuthComponent {
  protected readonly auth = inject(AuthService);
  tokenInput = '';
  error = signal('');

  async verifyToken() {
    this.error.set('');
    const res = await fetch(`${environment.emulatorBaseUrl}/v1/auth/magiclink/verify`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token: this.tokenInput.trim() }),
    });
    if (!res.ok) { this.error.set(`Token verification failed (${res.status})`); return; }
    const data = await res.json() as { sessionJwt: string; refreshJwt: string };
    this.auth.setSession(data.sessionJwt, data.refreshJwt);
  }

  async logout() { await this.auth.logout(); this.tokenInput = ''; }
}
