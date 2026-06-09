## ADDED Requirements

### Requirement: Reliable todo input value propagation
The `TodoTableComponent` SHALL propagate input field values to the component state synchronously via the DOM `input` event, so that any click on the Add button — including synthetic clicks from test automation — reads the current typed value.

#### Scenario: Playwright fill then click adds a todo
- **WHEN** a Playwright test calls `page.fill('#todo-input', 'some text')` followed by `page.click('#add-todo-btn')`
- **THEN** `addTodo()` receives the non-empty text, makes `POST /api/todos`, and the response is appended to the todos list

#### Scenario: Human typing adds a todo
- **WHEN** a user types text into the todo input and clicks Add
- **THEN** the todo is submitted and appears in the table (existing behavior preserved)

#### Scenario: Empty input does not submit
- **WHEN** the Add button is clicked with an empty or whitespace-only input value
- **THEN** no HTTP request is made and the input field remains focused (existing behavior preserved)

### Requirement: E2E tests guard against fill-click race
Each e2e test that calls `page.fill('#todo-input', ...)` SHALL assert `toHaveValue(...)` before calling `page.click('#add-todo-btn')`, ensuring the value is present in the DOM before the click is dispatched.

#### Scenario: toHaveValue passes before click
- **WHEN** `page.fill('#todo-input', 'Task text')` completes
- **THEN** `await expect(page.locator('#todo-input')).toHaveValue('Task text')` resolves before the click fires

#### Scenario: All 7 affected specs pass end-to-end
- **WHEN** `npx nx e2e nestjs-angular-sample` is run with all three servers (emulator, NestJS, Angular) running
- **THEN** all 16 tests pass including wf1, wf2, wf3, wf4, wf6, wf7-test1, and wf7-test2
