## Why

The emulator needs to be validated against real Descope behavior before integrating it into other projects. Without parity testing, behavioral drift goes undetected — leading to false confidence during local development and surprises in production.

The existing 11 scenario tests already cover the right flows but are locked to the emulator via escape hatches (`/emulator/reset`, `/emulator/otp/`). Only ~44 lines out of 1,117 are emulator-specific. Refactoring the existing suite to be dual-target is a small change with high leverage.

## What Changes

- Refactor `helpers/platform.ts` to support both emulator and live Descope targets via `PARITY_TARGET` env var
- Replace emulator escape hatches (`/emulator/reset`, `/emulator/otp/`) with standard Descope SDK operations (`deleteAllTestUsers`, `generateOTPForTestUser`, `generateMagicLinkForTestUser`) that work on both targets
- Add a `cleanup()` function that uses `/emulator/reset` on emulator (fast) or SDK deletion on live
- Add a response shape assertion helper for structural parity comparison
- Add a separate `vitest.parity.config.ts` so parity runs don't auto-start the emulator
- Add `test:parity:emulator` and `test:parity:live` npm scripts
- Existing scenario tests continue working as-is via the existing vitest config

## Capabilities

### New Capabilities

- `parity-test-harness`: Target abstraction in `helpers/platform.ts` that makes the existing test suite runnable against both emulator and live Descope

### Modified Capabilities

(none — no new test files; existing scenarios are refactored in place)

## Impact

- **Modified files**: `helpers/platform.ts` (target switching, cleanup, new helpers), ~44 lines across 11 existing test files (swap escape hatches for SDK operations)
- **New files**: `vitest.parity.config.ts`, package.json script additions
- **Dependencies**: No new dependencies
- **Live Descope**: Requires a dedicated test project with management key credentials
- **Emulator**: No code changes required
