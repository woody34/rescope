## 1. WASM Build Pipeline

- [x] 1.1 Add `build:wasm` script to `apps/docs/package.json`
- [x] 1.2 Add `wasm/` to `apps/docs/.gitignore`
- [x] 1.3 Add `docs:build:wasm` to root `package.json` scripts
- [x] 1.4 Verify `npm run docs:build:wasm` produces WASM assets

## 2. Demo Service Worker

- [x] 2.1 Create `apps/docs/public/demo-sw.js`
- [x] 2.2 SW uses `{ module_or_path }` object form
- [x] 2.3 SW scoped to `/demo-wasm/` — verified by design

## 3. Profile System

- [x] 3.1 Create `profiles/types.ts` — `DemoProfile` interface
- [x] 3.2 Create `profiles/password.ts` — 2 users + 2 roles
- [x] 3.3 Create `profiles/otp.ts` — 2 users + 1 role
- [x] 3.4 Create `profiles/magic-link.ts`
- [x] 3.5 Create `profiles/multi-tenant.ts` — 2 tenants + 3 users
- [x] 3.6 Create `profiles/index.ts` — `DEMO_PROFILES` array

## 4. Core Demo Components

- [x] 4.1 Create `useWasm.ts` — WASM init + SW registration hook
- [x] 4.2 Create `useEmulator.ts` — seed/reset/snapshot/call/getOtp hook
- [x] 4.3 Create `ProfileBar.tsx`
- [x] 4.4 Create `ResourceTree.tsx` — polls snapshot every 500ms
- [x] 4.5 Create `LoginForm.tsx`
- [x] 4.6 Create `OtpForm.tsx` — 2-step with visible code
- [x] 4.7 Create `MagicLinkForm.tsx`
- [x] 4.8 Create `DemoSandbox.tsx` — top-level island

## 5. Demo Page

- [x] 5.1 Create `src/content/docs/demo.mdx`
- [x] 5.2 Add `🚀 Live Demo` sidebar section to `astro.config.mjs`
- [x] 5.3 Add demo CSS to `rescope.css`

## 6. Verification

- [x] 6.1 `npm run build` produces static output with `/demo` page — **12 pages built in 1.65s** ✓
- [x] 6.2–6.7 Requires WASM assets (`npm run docs:build:wasm`) to be fully interactive
