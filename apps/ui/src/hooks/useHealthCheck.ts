import { useEffect, useState, useRef } from "react";

const HEALTH_URL = `${window.location.origin}/health`;
const POLL_INTERVAL_MS = 1000;

/**
 * Polls the /health endpoint and returns whether the API is reachable.
 * On disconnect, returns false. On reconnect, returns true.
 */
export function useHealthCheck() {
  const [connected, setConnected] = useState(true);
  const timerRef = useRef<ReturnType<typeof setInterval>>(undefined);

  useEffect(() => {
    let cancelled = false;

    const check = async () => {
      try {
        const controller = new AbortController();
        const timeout = setTimeout(() => controller.abort(), 3000);
        const res = await fetch(HEALTH_URL, {
          method: "GET",
          cache: "no-store",
          signal: controller.signal,
        });
        clearTimeout(timeout);
        if (!cancelled) setConnected(res.ok);
      } catch {
        if (!cancelled) setConnected(false);
      }
    };

    // Initial check
    check();
    timerRef.current = setInterval(check, POLL_INTERVAL_MS);

    return () => {
      cancelled = true;
      clearInterval(timerRef.current);
    };
  }, []);

  return connected;
}
