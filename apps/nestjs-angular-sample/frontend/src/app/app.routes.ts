import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: '', loadComponent: () => import('./scenarios/home.component').then(m => m.HomeComponent) },
  { path: 'wf1', loadComponent: () => import('./scenarios/wf1-passwordless.component').then(m => m.Wf1PasswordlessComponent) },
  { path: 'wf2', loadComponent: () => import('./scenarios/wf2-mfa.component').then(m => m.Wf2MfaComponent) },
  { path: 'wf3', loadComponent: () => import('./scenarios/wf3-tenancy.component').then(m => m.Wf3TenancyComponent) },
  { path: 'wf4', loadComponent: () => import('./scenarios/wf4-fraud.component').then(m => m.Wf4FraudComponent) },
  { path: 'wf5', loadComponent: () => import('./scenarios/wf5-access-keys.component').then(m => m.Wf5AccessKeysComponent) },
  { path: 'wf6', loadComponent: () => import('./scenarios/wf6-cli-auth.component').then(m => m.Wf6CliAuthComponent) },
  { path: 'wf7', loadComponent: () => import('./scenarios/wf7-marketplace.component').then(m => m.Wf7MarketplaceComponent) },
];
