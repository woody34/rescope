import type { Locator } from "@playwright/test";
import { BasePage } from "./_BasePage";
import { AccessKeysPage } from "./AccessKeysPage";
import { AuthMethodsPage } from "./AuthMethodsPage";
import { OtpInspectorPage } from "./OtpInspectorPage";
import { ResetPage } from "./ResetPage";
import { RolesPage } from "./RolesPage";
import { SnapshotPage } from "./SnapshotPage";
import { TenantsPage } from "./TenantsPage";
import { UsersPage } from "./UsersPage";

export class DashboardPage extends BasePage {
  /** Playwright goto() relative path (no leading slash, resolves against baseURL) */
  static readonly url = "dashboard";
  /** Full DOM path used in card/sidebar href attributes */
  static readonly path = "/dashboard";

  // ── Selectors ─────────────────────────────────────────────────────────────

  get selectors() {
    return {
      cards: "a.feature-card",
      /** Match by the full /xxx href that React Router renders */
      cardByPath: (path: string) => `a.feature-card[href="${path}"]`,
      sidebarLink: (label: string) => `.sidebar-link:has-text("${label}")`,
    } as const;
  }

  // ── Locators ──────────────────────────────────────────────────────────────

  get locators() {
    const s = this.selectors;
    return {
      cards: this.page.locator(s.cards),
      cardByPath: (path: string): Locator =>
        this.page.locator(s.cardByPath(path)),
      sidebarLink: (label: string): Locator =>
        this.page.locator(s.sidebarLink(label)),
    };
  }

  // ── Navigation ────────────────────────────────────────────────────────────

  async goto(): Promise<this> {
    await this.page.goto(DashboardPage.url);
    await this.locators.cards.first().waitFor();
    return this;
  }

  /**
   * Click a dashboard card by its full DOM path (e.g. UsersPage.path)
   * and return the appropriate strongly-typed POM.
   */
  async clickCard(path: string) {
    await this.locators.cardByPath(path).click();
    await this.page.waitForURL(`**${path}`);
    switch (path) {
      case UsersPage.path:
        return new UsersPage(this.page);
      case AccessKeysPage.path:
        return new AccessKeysPage(this.page);
      case RolesPage.path:
        return new RolesPage(this.page);
      case TenantsPage.path:
        return new TenantsPage(this.page);
      case AuthMethodsPage.path:
        return new AuthMethodsPage(this.page);
      case SnapshotPage.path:
        return new SnapshotPage(this.page);
      case ResetPage.path:
        return new ResetPage(this.page);
      case OtpInspectorPage.path:
        return new OtpInspectorPage(this.page);
      default:
        throw new Error(`Unknown dashboard card path: ${path}`);
    }
  }

  /**
   * Click a sidebar link by its visible label and return the appropriate POM.
   */
  async clickSidebarLink(label: string) {
    await this.locators.sidebarLink(label).click();
    switch (label) {
      case "Users":
        return new UsersPage(this.page);
      case "Access Keys":
        return new AccessKeysPage(this.page);
      case "Roles":
        return new RolesPage(this.page);
      case "Tenants":
        return new TenantsPage(this.page);
      case "Auth Methods":
        return new AuthMethodsPage(this.page);
      case "Snapshot":
        return new SnapshotPage(this.page);
      case "Reset":
        return new ResetPage(this.page);
      case "OTP Inspector":
        return new OtpInspectorPage(this.page);
      default:
        throw new Error(`Unknown sidebar label: ${label}`);
    }
  }

  // ── State helpers ─────────────────────────────────────────────────────────

  async getCardPaths(): Promise<string[]> {
    const cards = await this.locators.cards.all();
    return Promise.all(
      cards.map((c) => c.getAttribute("href").then((h) => h ?? "")),
    );
  }
}
