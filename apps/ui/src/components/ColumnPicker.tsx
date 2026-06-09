/**
 * ColumnPicker — gear icon dropdown for toggling visible table columns.
 * Preferences are persisted to localStorage.
 */
import { useState, useRef, useEffect, useCallback } from "react";

export interface ColumnDef {
  key: string;
  label: string;
}

interface Props {
  columns: ColumnDef[];
  visibleKeys: string[];
  onChange: (keys: string[]) => void;
}

export function ColumnPicker({ columns, visibleKeys, onChange }: Props) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  const handleClickOutside = useCallback((e: MouseEvent) => {
    if (ref.current && !ref.current.contains(e.target as Node)) {
      setOpen(false);
    }
  }, []);

  useEffect(() => {
    if (open) {
      document.addEventListener("mousedown", handleClickOutside);
      return () =>
        document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [open, handleClickOutside]);

  const toggle = (key: string) => {
    if (visibleKeys.includes(key)) {
      // Don't allow hiding all columns
      if (visibleKeys.length > 1) {
        onChange(visibleKeys.filter((k) => k !== key));
      }
    } else {
      onChange([...visibleKeys, key]);
    }
  };

  return (
    <div ref={ref} style={{ position: "relative", display: "inline-block" }}>
      <button
        className="btn btn-ghost btn-icon"
        data-testid="column-picker-btn"
        onClick={() => setOpen((o) => !o)}
        title="Configure visible columns"
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.75"
          strokeLinecap="round"
          strokeLinejoin="round"
          style={{ width: 14, height: 14 }}
        >
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
      </button>

      {open && (
        <div
          data-testid="column-picker-dropdown"
          style={{
            position: "absolute",
            top: "100%",
            right: 0,
            marginTop: 4,
            background: "var(--color-card-bg)",
            border: "1px solid var(--color-card-border)",
            borderRadius: "var(--radius-lg)",
            boxShadow: "0 8px 24px rgba(0,0,0,0.5)",
            padding: "var(--space-2)",
            minWidth: 180,
            zIndex: 200,
          }}
        >
          {columns.map((col) => (
            <label
              key={col.key}
              data-testid={`col-toggle-${col.key}`}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
                padding: "4px var(--space-2)",
                borderRadius: "var(--radius-sm)",
                cursor: "pointer",
                fontSize: "var(--text-sm)",
                color: "var(--color-text-primary)",
              }}
            >
              <input
                type="checkbox"
                checked={visibleKeys.includes(col.key)}
                onChange={() => toggle(col.key)}
              />
              {col.label}
            </label>
          ))}
        </div>
      )}
    </div>
  );
}
