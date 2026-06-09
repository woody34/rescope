import type { Locator } from "@playwright/test";
import { BasePage } from "./_BasePage";

/**
 * POM for the IdP emulator login page.
 * This page is served at:
 * - OIDC: /emulator/idp/:id/authorize?...
 * - SAML: /emulator/idp/:id/sso?...
 */
export class IdpLoginPage extends BasePage {
  get locators() {
    return {
      heading: this.page.locator("h1"),
      subtitle: this.page.locator(".subtitle"),
      badge: this.page.locator(".badge"),
      userRows: this.page.locator("table tbody tr"),
      emptyMessage: this.page.locator("td[colspan]"),
      loginButtons: this.page.locator("a.btn"),
    };
  }

  /** Find a specific user row by email or name text content */
  userRow(text: string): Locator {
    return this.page.locator("table tbody tr", { hasText: text });
  }

  /** Click the "Login" button for a specific user row */
  async loginAs(userText: string): Promise<void> {
    const row = this.userRow(userText);
    await row.locator("a.btn").click();
  }

  /** Navigate to OIDC authorize URL */
  async gotoOidc(idpId: string, opts: {
    clientId: string;
    redirectUri: string;
    state?: string;
    nonce?: string;
  }): Promise<this> {
    const params = new URLSearchParams({
      client_id: opts.clientId,
      redirect_uri: opts.redirectUri,
      response_type: "code",
      ...(opts.state && { state: opts.state }),
      ...(opts.nonce && { nonce: opts.nonce }),
    });
    await this.page.goto(`/emulator/idp/${idpId}/authorize?${params}`);
    await this.page.waitForSelector("h1");
    return this;
  }

  /** Navigate to SAML SSO URL */
  async gotoSaml(idpId: string, opts: {
    relayState?: string;
  } = {}): Promise<this> {
    const params = new URLSearchParams({
      ...(opts.relayState && { RelayState: opts.relayState }),
    });
    await this.page.goto(`/emulator/idp/${idpId}/sso?${params}`);
    await this.page.waitForSelector("h1");
    return this;
  }
}
