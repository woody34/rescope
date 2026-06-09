## Why

7 of the 16 `nestjs-angular-sample` e2e tests fail because Playwright's `page.fill()` + `page.click()` sequence races with Angular's `ngModel` change detection. By the time the Add button is clicked, `this.newTodoText` is still `''`, so `addTodo()` returns early without making any HTTP call — silently, with no error surfaced to the test.

## What Changes

- **Angular component**: Replace `[(ngModel)]="newTodoText"` with an explicit `(input)` event binding in `TodoTableComponent` so DOM input events reliably update the component field regardless of zone timing.
- **Playwright tests**: Add a `waitForFunction` assertion after `fill()` to confirm Angular has picked up the value before the button is clicked, as a belt-and-suspenders guard across all 7 affected specs.

## Capabilities

### New Capabilities
- `nestjs-e2e-todo-reliability`: Reliable Playwright interactions with the Angular todo section — `fill()` + `click()` consistently triggers `POST /api/todos`, table renders, and subsequent assertions pass.

### Modified Capabilities
<!-- none -->

## Impact

- `apps/nestjs-angular-sample/frontend/src/app/shared/todo-table.component.ts` — input binding change
- `apps/nestjs-angular-sample/e2e/wf1-passwordless.spec.ts`
- `apps/nestjs-angular-sample/e2e/wf2-mfa.spec.ts`
- `apps/nestjs-angular-sample/e2e/wf3-b2b-tenancy.spec.ts`
- `apps/nestjs-angular-sample/e2e/wf4-fraud-prevention.spec.ts`
- `apps/nestjs-angular-sample/e2e/wf6-cli-auth.spec.ts`
- `apps/nestjs-angular-sample/e2e/wf7-marketplace.spec.ts`
- No API, schema, or dependency changes.
