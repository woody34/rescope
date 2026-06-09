/**
 * WF5 — M2M / AI Agent (Access Keys)
 * No user JWT — admin-only management of service account access keys.
 */
import { Component, OnInit, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { environment } from '../../environments/environment';

interface AccessKey { id: string; name: string; status?: string; }

const MGMT_AUTH = `Bearer emulator-project:emulator-key`;

@Component({
  selector: 'app-wf5',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink],
  template: `
    <div class="scenario">
      <nav><a routerLink="/">← Home</a></nav>
      <h2>🤖 WF5 — M2M / AI Agent (Access Keys)</h2>
      <p class="description">Service account key lifecycle — create, list, disable, delete.</p>

      <div class="toolbar">
        <input [(ngModel)]="newKeyName" id="wf5-key-name" placeholder="Key name (e.g. ai-agent-prod)" />
        <button (click)="createKey()" id="wf5-create-key-btn">Create Key</button>
        <button (click)="loadKeys()" id="wf5-refresh-btn" class="secondary">Refresh</button>
      </div>

      @if (lastCreatedCleartext()) {
        <div class="cleartext-box" id="wf5-cleartext">
          <strong>⚠️ Access key (shown once):</strong>
          <code>{{ lastCreatedCleartext() }}</code>
        </div>
      }

      @if (loading()) {
        <p>Loading...</p>
      } @else if (keys().length === 0) {
        <p class="empty">No access keys. Create one above.</p>
      } @else {
        <table id="wf5-key-table">
          <thead><tr><th>Name</th><th>ID</th><th>Status</th><th>Actions</th></tr></thead>
          <tbody>
            @for (key of keys(); track key.id) {
              <tr [attr.data-key-id]="key.id">
                <td>{{ key.name }}</td>
                <td class="id-cell">{{ key.id }}</td>
                <td><span class="badge" [class.active]="key.status !== 'inactive'">{{ key.status ?? 'active' }}</span></td>
                <td class="actions">
                  <button (click)="disableKey(key.id)" [attr.data-testid]="'disable-' + key.id">Disable</button>
                  <button (click)="deleteKey(key.id)" [attr.data-testid]="'delete-' + key.id" class="danger">Delete</button>
                </td>
              </tr>
            }
          </tbody>
        </table>
      }

      @if (error()) { <p class="error" id="wf5-error">{{ error() }}</p> }
    </div>
  `,
  styles: [`
    .scenario { max-width: 900px; margin: 2rem auto; padding: 1rem; font-family: sans-serif; }
    nav { margin-bottom: 1rem; }
    .description { color: #666; margin-bottom: 1.5rem; }
    .toolbar { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
    input { padding: 0.5rem; border: 1px solid #ccc; border-radius: 4px; flex: 1; }
    button { padding: 0.4rem 0.8rem; border-radius: 4px; cursor: pointer; background: #0070f3; color: white; border: none; }
    button.secondary { background: #888; }
    button.danger { background: #c00; }
    .cleartext-box { background: #fffde7; border: 1px solid #f9a825; border-radius: 6px; padding: 0.75rem; margin-bottom: 1rem; font-size: 0.85rem; }
    code { display: block; margin-top: 0.25rem; word-break: break-all; }
    table { width: 100%; border-collapse: collapse; }
    th, td { padding: 0.5rem; border: 1px solid #ddd; }
    .id-cell { font-size: 0.75rem; color: #888; }
    .badge { padding: 0.15rem 0.4rem; border-radius: 4px; background: #fee; font-size: 0.8rem; }
    .badge.active { background: #efe; }
    .actions { display: flex; gap: 0.25rem; }
    .error { color: red; }
    .empty { color: #888; font-style: italic; }
  `],
})
export class Wf5AccessKeysComponent implements OnInit {
  keys = signal<AccessKey[]>([]);
  newKeyName = '';
  lastCreatedCleartext = signal('');
  loading = signal(false);
  error = signal('');
  private readonly base = environment.emulatorBaseUrl;

  ngOnInit() { this.loadKeys(); }

  async loadKeys() {
    this.loading.set(true);
    const res = await fetch(`${this.base}/v1/mgmt/accesskey/all`, { headers: { Authorization: MGMT_AUTH } });
    const body = await res.json() as { keys: AccessKey[] };
    this.keys.set(body.keys ?? []);
    this.loading.set(false);
  }

  async createKey() {
    if (!this.newKeyName.trim()) return;
    this.error.set('');
    const res = await fetch(`${this.base}/v1/mgmt/accesskey`, {
      method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: MGMT_AUTH },
      body: JSON.stringify({ name: this.newKeyName, expireTime: 0, roleNames: [] }),
    });
    if (!res.ok) { this.error.set(`Create failed (${res.status})`); return; }
    const body = await res.json() as { key: AccessKey; cleartext: string };
    this.lastCreatedCleartext.set(body.cleartext);
    this.newKeyName = '';
    await this.loadKeys();
  }

  async disableKey(id: string) {
    await fetch(`${this.base}/v1/mgmt/accesskey/disable`, {
      method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: MGMT_AUTH },
      body: JSON.stringify({ id }),
    });
    await this.loadKeys();
  }

  async deleteKey(id: string) {
    await fetch(`${this.base}/v1/mgmt/accesskey/delete`, {
      method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: MGMT_AUTH },
      body: JSON.stringify({ id }),
    });
    this.lastCreatedCleartext.set('');
    await this.loadKeys();
  }
}
