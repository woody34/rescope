# rescope-wasm

WASM build of rescope-core for running the emulator engine in the browser. Used by the docs site demo and the playground.

## Commands

| Task | Command (from repo root) |
|------|---------|
| Build (web target) | `cd apps/rescope-wasm && wasm-pack build --target web --release` |
| Build (no-modules, for docs) | `npm run docs:build:wasm` |

## Conventions

- Rust crate with `cdylib` output for wasm-bindgen
- Depends on `rescope-core` with `default-features = false` (no native-crypto, no ring/rcgen)
- Uses `getrandom` and `uuid` with `js` feature for browser-compatible randomness
- Excluded from the Cargo workspace (built separately via wasm-pack)
- Not a standard Nx project; built manually or via docs build script
