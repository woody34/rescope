/**
 * useColumnPreferences — custom hook for persisting column visibility to localStorage.
 */
import { useState, useCallback } from "react";

export function useColumnPreferences(
  storageKey: string,
  defaultKeys: string[],
): [string[], (keys: string[]) => void] {
  const [keys, setKeysState] = useState<string[]>(() => {
    try {
      const saved = localStorage.getItem(storageKey);
      if (saved) return JSON.parse(saved);
    } catch {
      /* ignore */
    }
    return defaultKeys;
  });

  const setKeys = useCallback(
    (newKeys: string[]) => {
      setKeysState(newKeys);
      localStorage.setItem(storageKey, JSON.stringify(newKeys));
    },
    [storageKey],
  );

  return [keys, setKeys];
}
