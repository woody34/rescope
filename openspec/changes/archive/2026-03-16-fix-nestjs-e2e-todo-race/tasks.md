## 1. Angular Component Fix

- [x] 1.1 In `apps/nestjs-angular-sample/frontend/src/app/shared/todo-table.component.ts`, change the todo input from `[(ngModel)]="newTodoText"` to `[value]="newTodoText" (input)="newTodoText = $any($event.target).value"` so that DOM input events update the field synchronously

## 2. Playwright Test Guards

- [x] 2.1 In `e2e/wf1-passwordless.spec.ts`, add `await expect(page.locator('#todo-input')).toHaveValue('WF1 test task');` between `page.fill()` and `page.click('#add-todo-btn')`
- [x] 2.2 In `e2e/wf2-mfa.spec.ts`, add `await expect(page.locator('#todo-input')).toHaveValue('WF2 test task');` between `page.fill()` and `page.click('#add-todo-btn')`
- [x] 2.3 In `e2e/wf3-b2b-tenancy.spec.ts`, add `await expect(page.locator('#todo-input')).toHaveValue(...)` between `page.fill()` and `page.click('#add-todo-btn')`
- [x] 2.4 In `e2e/wf4-fraud-prevention.spec.ts`, add `await expect(page.locator('#todo-input')).toHaveValue(...)` between `page.fill()` and `page.click('#add-todo-btn')`
- [x] 2.5 In `e2e/wf6-cli-auth.spec.ts`, add `await expect(page.locator('#todo-input')).toHaveValue('WF6 CLI task');` between `page.fill()` and `page.click('#add-todo-btn')`
- [x] 2.6 In `e2e/wf7-marketplace.spec.ts` (first fill/click), add `toHaveValue` guard for the admin user's todo add
- [x] 2.7 In `e2e/wf7-marketplace.spec.ts` (second fill/click), add `toHaveValue` guard for the viewer user's shared tenant todo add

## 3. Verification

- [x] 3.1 Run `npx nx e2e nestjs-angular-sample` and confirm all 16 tests pass (7 previously failing + 9 already passing)
