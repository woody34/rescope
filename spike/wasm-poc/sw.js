// sw.js — Service Worker that loads WASM and intercepts API requests
//
// This is the core of the zero-install playground pattern:
// 1. On install, load the WASM module and initialize state
// 2. On fetch, intercept Descope API patterns and route to WASM
// 3. All other requests pass through to the network

let wasmModule = null;

// ── Install: load WASM and initialize ─────────────────────────────────
self.addEventListener('install', (event) => {
  console.log('[SW] Installing...');
  event.waitUntil(
    (async () => {
      try {
        // Import the wasm-bindgen generated JS module
        const wasm = await import('./pkg/rescope_wasm_poc.js');
        await wasm.default(); // Initialize WASM

        // Initialize the emulator state
        wasm.init(JSON.stringify({
          project_id: "playground-project"
        }));

        wasmModule = wasm;
        console.log('[SW] WASM initialized successfully');
      } catch (e) {
        console.error('[SW] Failed to initialize WASM:', e);
      }

      // Activate immediately (don't wait for old SW to terminate)
      await self.skipWaiting();
    })()
  );
});

// ── Activate: claim all clients immediately ───────────────────────────
self.addEventListener('activate', (event) => {
  console.log('[SW] Activated');
  event.waitUntil(self.clients.claim());
});

// ── Fetch: intercept API requests, pass through everything else ───────
const API_PATTERNS = [
  /^\/v1\/auth\//,
  /^\/v1\/mgmt\//,
  /^\/emulator\//,
  /^\/\.well-known\//,
  /^\/health$/,
];

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);

  // Only intercept same-origin requests matching API patterns
  if (url.origin !== self.location.origin) return;

  const shouldIntercept = API_PATTERNS.some(pattern => pattern.test(url.pathname));
  if (!shouldIntercept) return;

  if (!wasmModule) {
    event.respondWith(new Response(
      JSON.stringify({ error: 'WASM not ready' }),
      { status: 503, headers: { 'content-type': 'application/json' } }
    ));
    return;
  }

  event.respondWith(handleWasmRequest(event.request));
});

async function handleWasmRequest(request) {
  try {
    const body = request.method !== 'GET' && request.method !== 'HEAD'
      ? await request.text()
      : null;

    const url = new URL(request.url);

    // Serialize to WasmRequest
    const wasmRequest = {
      method: request.method,
      path: url.pathname,
      headers: Object.fromEntries(request.headers.entries()),
      body: body || null,
    };

    // Call WASM
    const responseJson = wasmModule.handle_request(JSON.stringify(wasmRequest));
    const wasmResponse = JSON.parse(responseJson);

    // Build a proper Response
    const responseHeaders = new Headers(wasmResponse.headers || {});
    return new Response(wasmResponse.body, {
      status: wasmResponse.status,
      headers: responseHeaders,
    });
  } catch (e) {
    console.error('[SW] WASM request error:', e);
    return new Response(
      JSON.stringify({ error: `WASM error: ${e.message}` }),
      { status: 500, headers: { 'content-type': 'application/json' } }
    );
  }
}
