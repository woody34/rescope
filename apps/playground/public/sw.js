// sw.js — Service Worker for the Rescope WASM Playground
//
// Uses importScripts + lazy WASM initialization.
// Intercepts API fetch patterns and routes them to the WASM emulator.
// WASM is loaded lazily on first API request (survives SW restarts).

let wasmReady = false;
let initPromise = null;

async function ensureWasm() {
  if (wasmReady) return true;
  if (initPromise) return initPromise;

  initPromise = (async () => {
    try {
      importScripts('/wasm/rescope_wasm.js');
      await wasm_bindgen({ module_or_path: '/wasm/rescope_wasm_bg.wasm' });
      wasm_bindgen.init(JSON.stringify({ project_id: 'playground' }));
      wasmReady = true;
      console.log('[SW] WASM emulator initialized');
      return true;
    } catch (e) {
      console.error('[SW] WASM init failed:', e);
      initPromise = null; // allow retry
      return false;
    }
  })();
  return initPromise;
}

// ── Install: pre-load WASM so first request is fast ───────────────────────────
self.addEventListener('install', (event) => {
  event.waitUntil(ensureWasm().then(() => self.skipWaiting()));
});

// ── Activate: claim all clients immediately ───────────────────────────────────
self.addEventListener('activate', (event) => {
  event.waitUntil(self.clients.claim());
});

// ── Fetch: intercept API requests ─────────────────────────────────────────────
const API_PATTERNS = [
  /^\/v1\/auth\//,
  /^\/v1\/mgmt\//,
  /^\/v2\/mgmt\//,
  /^\/emulator\//,
  /^\/\.well-known\//,
  /^\/health$/,
];

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  if (url.origin !== self.location.origin) return;

  const isApi = API_PATTERNS.some(p => p.test(url.pathname));
  if (!isApi) return;

  event.respondWith(handleWasmRequest(event.request));
});

async function handleWasmRequest(request) {
  const ready = await ensureWasm();
  if (!ready) {
    return new Response(
      JSON.stringify({ error: 'WASM failed to initialize' }),
      { status: 503, headers: { 'content-type': 'application/json' } }
    );
  }

  try {
    const url = new URL(request.url);
    const body = (request.method !== 'GET' && request.method !== 'HEAD')
      ? await request.text()
      : null;

    const wasmReq = JSON.stringify({
      method: request.method,
      path: url.pathname,
      headers: Object.fromEntries(request.headers.entries()),
      body,
    });

    const responseJson = wasm_bindgen.handle_request(wasmReq);
    const resp = JSON.parse(responseJson);

    return new Response(resp.body, {
      status: resp.status,
      headers: new Headers(resp.headers || {}),
    });
  } catch (e) {
    console.error('[SW] WASM error:', e);
    return new Response(
      JSON.stringify({ error: `WASM error: ${e.message}` }),
      { status: 500, headers: { 'content-type': 'application/json' } }
    );
  }
}
