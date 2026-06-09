#!/usr/bin/env node
/**
 * Copies the self-hosted Descope web component bundle into Angular's assets.
 * Run automatically via "postinstall" in package.json.
 *
 * The descope-wc bundle fetches its own flow JS from `baseStaticUrl`.
 * By self-hosting it at /assets/descope-wc.js, we avoid hitting Descope's CDN.
 */
const fs = require('fs');
const path = require('path');

const src = path.join(__dirname, '..', 'node_modules', '@descope', 'web-component', 'dist', 'index.js');
const dest = path.join(__dirname, '..', 'src', 'assets', 'descope-wc.js');

if (fs.existsSync(src)) {
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.copyFileSync(src, dest);
  console.log('[postinstall] Copied descope-wc bundle to src/assets/descope-wc.js');
} else {
  console.warn('[postinstall] @descope/web-component not found — descope-wc.js not copied');
}
