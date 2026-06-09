import { Component, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { AuthService } from '../auth/auth.service';

@Component({
  selector: 'app-jwt-claims',
  standalone: true,
  imports: [CommonModule],
  template: `
    @if (auth.isAuthenticated()) {
      <div class="claims-panel" id="jwt-claims-panel">
        <h4>🔑 JWT Claims</h4>
        <dl>
          @for (entry of claimsEntries(); track entry[0]) {
            <div class="claim-row">
              <dt>{{ entry[0] }}</dt>
              <dd [attr.data-claim]="entry[0]">{{ entry[1] | json }}</dd>
            </div>
          }
        </dl>
      </div>
    }
  `,
  styles: [`
    .claims-panel { background: #f8f8f8; border: 1px solid #ddd; border-radius: 6px; padding: 1rem; font-size: 0.85rem; }
    h4 { margin: 0 0 0.5rem; }
    dl { margin: 0; }
    .claim-row { display: grid; grid-template-columns: 120px 1fr; gap: 0.25rem; margin-bottom: 0.25rem; }
    dt { font-weight: 600; color: #555; }
    dd { margin: 0; word-break: break-all; color: #333; }
  `],
})
export class JwtClaimsComponent {
  protected readonly auth = inject(AuthService);

  claimsEntries() {
    const claims = this.auth.jwtClaims();
    if (!claims) return [];
    // Show most useful claims first
    const priority = ['sub', 'dct', 'roles', 'tenants', 'iss', 'iat', 'exp'];
    const sorted = [...priority.filter(k => k in claims), ...Object.keys(claims).filter(k => !priority.includes(k))];
    return sorted.map(k => [k, claims[k]] as [string, unknown]);
  }
}
