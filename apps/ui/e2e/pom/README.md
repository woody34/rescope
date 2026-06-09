# POM Rules

Rules for writing and maintaining Page Object Models in this project.

## Structure

Every POM **must** extend `BasePage` and follow this field/section order:

```ts
export class MyPage extends BasePage {
  static readonly url = "/my-page";   // 1. Static URL

  get selectors() { ... }             // 2. Raw CSS/ID strings
  get locators() { ... }              // 3. Playwright Locators derived from selectors

  async goto(): Promise<this> { ... } // 4. Navigation
  // ... action methods               // 5. Actions
  // ... state helpers                // 6. State helpers
}
```

## Rules

### 1. `static readonly url`
- Every POM must have a `static readonly url` pointing to its route.
- `goto()` must use `MyPage.url` (not a hardcoded string).
- `DashboardPage` switch cases use these static constants to make relationships explicit.

### 2. `get selectors()`
- All CSS selectors and element IDs live here — **one source of truth**.
- Dynamic selectors (that take a parameter) are functions: `rowById: (id: string) => \`#row-${id}\``.
- Nothing outside `get locators()` should contain a raw selector string.

### 3. `get locators()`
- Derives every entry from `this.selectors` via `this.page.locator(s.xxx)`.
- Dynamic locators are functions that return `Locator`: `rowById: (id: string): Locator => this.page.locator(s.rowById(id))`.
- **No `this.page.locator()` calls are permitted outside this getter.**

### 4. `goto()` returns `Promise<this>`
- Always `return this` at the end so callers can chain: `const p = await new MyPage(page).goto()`.

### 5. Navigation methods that change pages return a new POM instance
- If a user action results in navigating to a **different** page, return `new TargetPage(this.page)`.
- Use the `DashboardPage` switch pattern — not generic type parameters — to keep relationships readable.
- Return the concrete type, not a generic. TypeScript infers the union automatically.

### 6. No raw selectors in action/helper methods
- Action methods interact exclusively through `this.locators.xxx` or `this.selectors.xxx` (for `.locator(s.xxx)` sub-locators).
- Never write `this.page.locator('#some-id')` in a method body.

## Adding a new page

1. Create `NewPage.ts` extending `BasePage`.
2. Add `static readonly url`.
3. Add all element IDs to `get selectors()`.
4. Derive all locators in `get locators()`.
5. Add a `case NewPage.url:` to `DashboardPage.clickCard()`.
6. Add a `case "New Page Label":` to `DashboardPage.clickSidebarLink()`.
