import { Component } from '@angular/core';
import { RouterLink, RouterOutlet } from '@angular/router';

const SCENARIOS = [
  { route: 'wf1', title: 'WF1 — Passwordless', desc: 'OTP or magic link sign-in/sign-up', icon: '🔑' },
  { route: 'wf2', title: 'WF2 — MFA', desc: 'Password + OTP second factor', icon: '🔐' },
  { route: 'wf3', title: 'WF3 — B2B Tenancy', desc: 'Tenant select, dct JWT claim', icon: '🏢' },
  { route: 'wf4', title: 'WF4 — Fraud Prevention', desc: 'Account disable/enable, force logout', icon: '🛡️' },
  { route: 'wf5', title: 'WF5 — M2M / Access Keys', desc: 'Service account key lifecycle', icon: '🤖' },
  { route: 'wf6', title: 'WF6 — CLI Auth', desc: 'Magic link as PKCE proxy', icon: '💻' },
  { route: 'wf7', title: 'WF7 — Marketplace', desc: 'Tenant + multi-user + service key', icon: '🛒' },
];

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [RouterLink],
  template: `
    <div class="home">
      <h1>Descope Emulator — Sample App</h1>
      <p class="subtitle">Select a workflow scenario to demo</p>
      <div class="grid" id="scenario-grid">
        @for (s of scenarios; track s.route) {
          <a [routerLink]="s.route" class="card" [id]="s.route + '-card'" [attr.data-testid]="s.route + '-card'">
            <span class="icon">{{ s.icon }}</span>
            <strong>{{ s.title }}</strong>
            <span class="desc">{{ s.desc }}</span>
          </a>
        }
      </div>
    </div>
  `,
  styles: [`
    .home { max-width: 900px; margin: 2rem auto; padding: 1rem; font-family: sans-serif; }
    h1 { font-size: 1.8rem; margin-bottom: 0.25rem; }
    .subtitle { color: #666; margin-bottom: 2rem; }
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 1rem; }
    .card { display: flex; flex-direction: column; gap: 0.25rem; padding: 1rem; border: 1px solid #ddd; border-radius: 8px;
            text-decoration: none; color: inherit; transition: box-shadow 0.2s; }
    .card:hover { box-shadow: 0 2px 8px rgba(0,0,0,0.12); }
    .icon { font-size: 1.5rem; }
    .desc { font-size: 0.8rem; color: #777; }
  `],
})
export class HomeComponent {
  readonly scenarios = SCENARIOS;
}
