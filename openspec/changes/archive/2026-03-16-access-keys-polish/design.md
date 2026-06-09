## Context

The `AccessKeysPage.tsx` currently shows a success banner after key creation with light text on a light background (hard to read in the dark theme). The banner includes copy functionality but no guidance on how to use the key. Developers have to manually construct API calls, which adds friction.

## Goals / Non-Goals

**Goals:**
- Fix the cleartext banner contrast for readability in the dark theme
- Show a copy-ready curl example that uses the actual key value and dynamically resolved emulator URL/port
- Full E2E test coverage for the new UI elements

**Non-Goals:**
- Restructuring the access keys page layout
- Adding new API endpoints
- Key activation/re-enabling (separate concern)

## Decisions

### 1. Banner Color Fix
**Decision:** Replace the current `--color-success-bg` / `white` / `--color-success-text` background/text combination with proper dark-theme-compatible colors. Use a darker background with high-contrast text colors for both the banner message and the code block.

**Rationale:** The current light green banner with light text fails WCAG contrast requirements in the dark theme. The code block currently uses `white` background which clashes with the dark theme.

### 2. Dynamic Emulator URL via `window.location.origin`
**Decision:** Use `window.location.origin` (already used in `api.ts` for the `BASE` URL) to construct the curl example with the actual emulator host/port.

**Rationale:** User explicitly requested pulling the actual URL instead of hardcoding `localhost:4600`. Since the UI is served from the emulator itself, `window.location.origin` gives the correct base URL.

**Alternative considered:** Reading from a config endpoint (rejected — unnecessary complexity when origin already works).

### 3. Curl Template Format
**Decision:** Show a pre-formatted, syntax-highlighted curl command in a code block with a copy button. Using the user search endpoint as the example since it's the simplest management API call that demonstrates authentication:
```
curl -s -w '\n%{http_code}\n' <origin>/v1/mgmt/user/search \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer <cleartext>' \
  -d '{}' | jq
```

**Rationale:** This exactly matches the format the user provided. It's a real, runnable command that validates the key works.

## Risks / Trade-offs

- **Curl example may not work on Windows** → Mitigation: acceptable, as this is a dev tool primarily used on macOS/Linux. Could add a note or PowerShell alternative in the future.
