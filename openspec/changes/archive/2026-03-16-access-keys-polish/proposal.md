## Why

The Access Keys page has two UX issues that hurt developer experience: the created-key banner text is hard to read due to light-on-light color choices, and after creating a key there's no guidance on how to actually use it. Providing a ready-to-copy curl example with the real key value and emulator URL would significantly reduce friction for developers testing backend API authentication.

## What Changes

- **Fix cleartext banner readability** — the success banner currently uses light text on a light background. Update colors to ensure high contrast and readability in the dark theme.
- **Add curl test example on key creation** — when a new access key is created, show a formatted, copy-ready curl command that uses the actual cleartext key value and dynamically resolves the emulator's host/port. Example format:
  ```
  curl -s -w '\n%{http_code}\n' http://<emulator-url>/v1/mgmt/user/search \
    -H 'Content-Type: application/json' \
    -H 'Authorization: Bearer <cleartext-key>' \
    -d '{}' | jq
  ```

## Capabilities

### New Capabilities
- `access-key-test-example`: Display a copy-ready curl command after key creation, using the actual key value and emulator URL

### Modified Capabilities
_(none — no existing specs to modify)_

## Impact

- **UI files**: `AccessKeysPage.tsx` (modify cleartext banner styling, add curl example section)
- **API**: No Rust changes needed
- **Tests**: New E2E Playwright tests (POM pattern) for banner readability and curl example display. Verify the curl command contains the actual key and correct emulator URL.
