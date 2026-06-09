// demo-sw.js — Service Worker for the Rescope Docs Demo
//
// Scoped to / — intercepts all /demo-wasm/ API calls from the interactive demo
// page and routes them to the WASM emulator.

// Cache-bust token — updated each time the SW script changes (Vite serves
// the SW with no-cache via updateViaCache:'none', so this value is always fresh).
const SW_BUILD_TS = 1773846210693;

let wasmReady = false;
let initPromise = null;

async function ensureWasm() {
  if (wasmReady) return true;
  if (initPromise) return initPromise;

  initPromise = (async () => {
    try {
      // --target no-modules exposes wasm_bindgen as a global on self
      // Cache-bust with SW install time so each new SW gets fresh WASM
      importScripts(`/wasm/rescope_wasm.js?t=${SW_BUILD_TS}`);
      await wasm_bindgen(`/wasm/rescope_wasm_bg.wasm?t=${SW_BUILD_TS}`);
      wasm_bindgen.init(JSON.stringify({ project_id: 'docs-demo' }));
      wasmReady = true;
      console.log('[Demo SW] WASM emulator initialized');
      return true;
    } catch (e) {
      console.error('[Demo SW] WASM init failed:', e);
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

// ── Fetch: intercept ALL requests within /demo-wasm/ scope ────────────────────
// Since the SW is scoped to /demo-wasm/, it only sees requests made to paths
// under /demo-wasm/. We strip the prefix before passing to the WASM router.
self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  if (url.origin !== self.location.origin) return;
  // Only handle paths that start with /demo-wasm/
  if (!url.pathname.startsWith('/demo-wasm/')) return;

  event.respondWith(handleWasmRequest(event.request, url));
});

async function handleWasmRequest(request, url) {
  const ready = await ensureWasm();
  if (!ready) {
    return new Response(
      JSON.stringify({ error: 'WASM failed to initialize' }),
      { status: 503, headers: { 'content-type': 'application/json' } }
    );
  }

  try {
    // Strip /demo-wasm prefix and URL-decode so WASM sees plain characters (e.g. @ not %40)
    const rawPath = url.pathname.replace(/^\/demo-wasm/, '');
    const path = decodeURIComponent(rawPath);
    const body = (request.method !== 'GET' && request.method !== 'HEAD')
      ? await request.text()
      : null;

    const wasmReq = JSON.stringify({
      method: request.method,
      path,
      query: url.search,
      headers: Object.fromEntries(request.headers.entries()),
      body,
    });

    const responseJson = wasm_bindgen.handle_request(wasmReq);
    const resp = JSON.parse(responseJson);

    if (resp.status >= 400 || path.includes('/batch') || path.includes('/emulator/')) {
      console.debug('[Demo SW]', request.method, path, '→', resp.status, resp.body?.slice(0, 200));
    }

    return new Response(resp.body, {
      status: resp.status,
      headers: new Headers(resp.headers || {}),
    });
  } catch (e) {
    console.error('[Demo SW] WASM error:', e);
    return new Response(
      JSON.stringify({ error: `WASM error: ${e.message}` }),
      { status: 500, headers: { 'content-type': 'application/json' } }
    );
  }
}
