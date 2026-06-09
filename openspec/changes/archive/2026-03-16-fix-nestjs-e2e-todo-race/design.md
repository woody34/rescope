## Context

The `nestjs-angular-sample` app uses Angular's `[(ngModel)]` two-way binding on the todo input. Playwright's `page.fill()` injects a synthetic `input` event, but Angular's change detection zone doesn't always flush synchronously before the Playwright `page.click()` on the Add button arrives. The result: `addTodo()` reads `this.newTodoText` as `''`, hits the early-return guard `if (!this.newTodoText.trim()) return`, and no HTTP call is made.

This affects 7 of 16 tests — all the ones that add a todo and assert `#todo-table` is visible.

## Goals / Non-Goals

**Goals:**
- Make `TodoTableComponent.addTodo()` reliably receive the typed text when triggered by Playwright's synthetic events
- Guard all 7 affected test specs against the same race on the test side
- Keep the fix minimal — no refactoring of the auth flow, routing, or backend

**Non-Goals:**
- Changing the visual design of the todo UI
- Rearchitecting the component to use reactive forms
- Fixing any issues in the passing tests (they're fine)

## Decisions

### Decision 1: Fix the Angular component (primary fix)

**Chosen**: Replace `[(ngModel)]="newTodoText"` with an explicit `(input)` event handler:
```html
<input [value]="newTodoText" (input)="newTodoText = $any($event.target).value" id="todo-input" />
```

**Why**: `[(ngModel)]` relies on Angular's Forms module event pipeline, which processes `input` events asynchronously within the zone. An explicit `(input)` handler on the element fires synchronously in the same tick, so by the time the browser processes the click, the field is already updated.

**Alternatives considered**:
- *Test-side `waitForFunction` only*: Fixes the test but leaves the component fragile for any other automated testing tool.
- *`pressSequentially()` in tests*: Simulates real keystrokes but is 10–20x slower per test.
- *Reactive forms (`FormControl`)*: Correct long-term but over-engineered for a sample app.

### Decision 2: Add test-side guard as belt-and-suspenders

After `page.fill()`, wait until the input's DOM value matches before clicking:
```ts
await page.fill('#todo-input', 'Task text');
await expect(page.locator('#todo-input')).toHaveValue('Task text');
await page.click('#add-todo-btn');
```

Playwright's `toHaveValue` retries until the value is set, making the intent explicit in the test and adding resiliency with zero performance cost.

## Risks / Trade-offs

- **[Risk] Other `ngModel` inputs may have the same issue** → `todo-table.component.ts` is the only component that gatekeeps on `newTodoText.trim()`. Other inputs don't have write-through-click-on-same-tick patterns in the tests. Low risk.
- **[Risk] `$any($event.target).value` is less type-safe** → Acceptable in a template binding for a sample app. Could use `(input)="onInput($event)"` with a typed handler if desired — but that's a style choice, not a correctness issue.

## Migration Plan

1. Update `todo-table.component.ts` input binding
2. Update each of 7 failing spec files to add `toHaveValue` guard after `fill()`
3. Run `npx nx e2e nestjs-angular-sample` — all 16 tests should pass
4. No deployment steps needed (local dev tool / sample app only)
