import { useEffect, useState } from 'react';

const WASM_SW = '/demo-sw.js';
const WASM_SCOPE = '/';

export type WasmStatus = 'loading' | 'ready' | 'error' | 'unavailable';

export function useWasm() {
  const [status, setStatus] = useState<WasmStatus>('loading');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    (async () => {
      if (!('serviceWorker' in navigator)) {
        setStatus('unavailable');
        return;
      }

      // Probe for WASM assets
      try {
        const probe = await fetch('/wasm/rescope_wasm.js');
        if (!probe.ok) {
          if (!cancelled) setStatus('unavailable');
          return;
        }
      } catch {
        if (!cancelled) setStatus('unavailable');
        return;
      }

      try {
        setStatus('loading');

        // Unregister stale SWs at wrong scope
        const existing = await navigator.serviceWorker.getRegistrations();
        for (const r of existing) {
          if (r.scope !== `${location.origin}${WASM_SCOPE}`) {
            await r.unregister();
          }
        }

        // Register (or get existing) SW
        const reg = await navigator.serviceWorker.register(WASM_SW, {
          scope: WASM_SCOPE,
          updateViaCache: 'none',
        });

        // Force update check — fetches the SW script and starts a new install
        // if the script changed (e.g. after npm run docs:build:wasm which stamps
        // a new SW_BUILD_TS into demo-sw.js).
        const updatedReg = await reg.update();

        // Determine which SW we need to wait for (if any)
        const pendingSW = updatedReg.installing ?? updatedReg.waiting ?? reg.installing ?? reg.waiting;

        if (pendingSW) {
          // New SW is installing — wait for it to activate and claim us.
          // SW calls skipWaiting() + clients.claim(), which fires controllerchange.
          await new Promise<void>((resolve) => {
            const timeout = setTimeout(resolve, 8000); // 8s max wait

            navigator.serviceWorker.addEventListener('controllerchange', () => {
              clearTimeout(timeout);
              resolve();
            }, { once: true });

            pendingSW.addEventListener('statechange', function handler(this: ServiceWorker) {
              if (this.state === 'activated') {
                clearTimeout(timeout);
                pendingSW.removeEventListener('statechange', handler);
                setTimeout(resolve, 100);
              }
              if (this.state === 'redundant') {
                clearTimeout(timeout);
                pendingSW.removeEventListener('statechange', handler);
                resolve(); // fall through to ready with whatever SW is active
              }
            });
          });

        } else if (!navigator.serviceWorker.controller) {
          // No pending SW, no controller yet (should be rare) — wait
          await new Promise<void>((resolve) => {
            const timeout = setTimeout(resolve, 5000);
            navigator.serviceWorker.addEventListener('controllerchange', () => {
              clearTimeout(timeout);
              resolve();
            }, { once: true });
          });
        }
        // else: existing active SW is already current — proceed immediately

        await new Promise((r) => setTimeout(r, 200));
        if (!cancelled) setStatus('ready');
      } catch (e: unknown) {
        if (!cancelled) {
          setStatus('error');
          setError(e instanceof Error ? e.message : String(e));
        }
      }
    })();

    return () => { cancelled = true; };
  }, []);

  return { status, error };
}
