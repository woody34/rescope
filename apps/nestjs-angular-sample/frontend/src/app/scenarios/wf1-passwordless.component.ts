/**
 * WF1 — Passwordless (OTP or Magic Link)
 *
 * Uses the self-hosted descope-wc web component to show the sign-up-or-in flow.
 * Once authenticated, loads the todo table and JWT claims panel.
 */
import { Component, OnInit, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { CUSTOM_ELEMENTS_SCHEMA } from '@angular/core';
import { AuthService } from '../auth/auth.service';
import { TodoTableComponent } from '../shared/todo-table.component';
import { JwtClaimsComponent } from '../shared/jwt-claims.component';
import { environment } from '../../environments/environment';

@Component({
  selector: 'app-wf1',
  standalone: true,
  imports: [CommonModule, RouterLink, TodoTableComponent, JwtClaimsComponent],
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  template: `
    <div class="scenario">
      <nav><a routerLink="/">← Home</a></nav>
      <h2>🔑 WF1 — Passwordless (OTP / Magic Link)</h2>
      <p class="description">Consumer apps eliminating passwords via OTP and magic links.</p>

      @if (!auth.isAuthenticated()) {
        <div class="auth-section">
          <descope-wc
            id="wf1-descope-wc"
            [attr.project-id]="projectId"
            flow-id="sign-up-or-in"
            [attr.base-url]="emulatorBaseUrl"
            [attr.base-static-url]="staticBaseUrl"
            (success)="onSuccess($any($event))"
            (error)="onError($any($event))"
          ></descope-wc>
          @if (errorMsg()) {
            <p class="error" id="wf1-error">{{ errorMsg() }}</p>
          }
        </div>
      } @else {
        <div class="authenticated-section">
          <div class="user-bar" id="wf1-user-bar">
            <span>Signed in as <strong>{{ auth.session()?.userId }}</strong></span>
            <button (click)="logout()" id="wf1-logout-btn">Log out</button>
          </div>
          <div class="content-grid">
            <app-todo-table id="wf1-todo-table"></app-todo-table>
            <app-jwt-claims></app-jwt-claims>
          </div>
        </div>
      }
    </div>
  `,
  styles: [scenarioStyles()],
})
export class Wf1PasswordlessComponent implements OnInit {
  protected readonly auth = inject(AuthService);
  protected readonly projectId = environment.descopeProjectId;
  protected readonly emulatorBaseUrl = environment.emulatorBaseUrl;
  protected readonly staticBaseUrl = 'http://localhost:4444';
  protected errorMsg = signal('');

  ngOnInit() {
    this.auth.restore();
    // Dynamically load self-hosted descope-wc bundle
    if (!document.getElementById('descope-wc-script')) {
      const script = document.createElement('script');
      script.id = 'descope-wc-script';
      script.src = '/assets/descope-wc.js';
      script.type = 'module';
      document.head.appendChild(script);
    }
  }

  onSuccess(event: CustomEvent) {
    const detail = event.detail as { sessionJwt?: string; refreshJwt?: string };
    if (detail.sessionJwt) {
      this.auth.setSession(detail.sessionJwt, detail.refreshJwt);
    }
  }

  onError(event: CustomEvent) {
    this.errorMsg.set(`Auth error: ${JSON.stringify(event.detail)}`);
  }

  async logout() {
    await this.auth.logout();
  }
}

function scenarioStyles() {
  return `
    .scenario { max-width: 900px; margin: 2rem auto; padding: 1rem; font-family: sans-serif; }
    nav { margin-bottom: 1rem; }
    h2 { margin-bottom: 0.25rem; }
    .description { color: #666; margin-bottom: 1.5rem; }
    .auth-section { max-width: 480px; }
    .user-bar { display: flex; align-items: center; gap: 1rem; margin-bottom: 1rem; padding: 0.75rem; background: #eef; border-radius: 6px; }
    .content-grid { display: grid; grid-template-columns: 1fr 300px; gap: 1rem; align-items: start; }
    .error { color: red; margin-top: 0.5rem; }
    button { padding: 0.4rem 0.8rem; border-radius: 4px; cursor: pointer; border: 1px solid #ccc; }
  `;
}
