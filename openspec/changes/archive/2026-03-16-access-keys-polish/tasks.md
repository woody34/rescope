## 1. Banner Readability Fix

- [x] 1.1 Update cleartext banner background color to use dark-theme-compatible color (e.g., `var(--color-surface-elevated)` or darker green)
- [x] 1.2 Update banner text color to high-contrast light text
- [x] 1.3 Update code block background from `white` to dark theme color (e.g., `var(--color-bg-secondary)`)
- [x] 1.4 Update code block text color to light monospace text
- [x] 1.5 Verify visual contrast meets readability standards in dark theme

## 2. Curl Test Example

- [x] 2.1 Add curl example section below the cleartext key in the created-key banner
- [x] 2.2 Dynamically interpolate `window.location.origin` into the curl command for the emulator URL
- [x] 2.3 Dynamically interpolate the `createdKey.cleartext` value into the Authorization header
- [x] 2.4 Format the curl command in a styled code block consistent with the dark theme
- [x] 2.5 Add a "Copy" button for the curl command that copies the full command to clipboard

## 3. E2E Tests (Playwright, POM pattern)

- [x] 3.1 Create or extend `AccessKeysPage` POM class with methods for banner and curl example interactions
- [x] 3.2 Write E2E test: create key and verify banner text is readable (not light-on-light)
- [x] 3.3 Write E2E test: verify curl example is displayed after key creation
- [x] 3.4 Write E2E test: verify curl command contains the actual cleartext key value
- [x] 3.5 Write E2E test: verify curl command contains the correct emulator URL
- [x] 3.6 Write E2E test: verify copy button copies the curl command to clipboard
