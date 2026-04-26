import { useRef, useState } from "react";
import { ProviderConfig, providerDot } from "../services/tauri";

interface Props {
  providers: ProviderConfig[];
  value: string | null;
  onChange: (providerId: string) => void;
}

export default function ProviderPicker({ providers, value, onChange }: Props) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  const current = providers.find((p) => p.id === value);

  // Close on outside click
  const handleBlur = (e: React.FocusEvent) => {
    if (!ref.current?.contains(e.relatedTarget as Node)) setOpen(false);
  };

  return (
    <div ref={ref} style={{ position: "relative" }} onBlur={handleBlur}>
      <button
        onClick={() => setOpen((o) => !o)}
        style={{
          display: "flex", alignItems: "center", gap: 7,
          padding: "5px 10px", borderRadius: 8,
          border: "1px solid var(--line)", background: "var(--bg-2)",
          color: "var(--fg)", fontFamily: "var(--mg-sans)", fontSize: 13, fontWeight: 500,
          cursor: "pointer",
        }}
      >
        {current ? (
          <>
            <span style={{ width: 8, height: 8, borderRadius: "50%", background: providerDot(current) }} />
            {current.display_name}
          </>
        ) : (
          <span style={{ color: "var(--fg-3)" }}>Select provider…</span>
        )}
        <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
          <path d={open ? "M1 6l3.5-3.5L8 6" : "M1 3l3.5 3.5L8 3"} />
        </svg>
      </button>

      {open && (
        <div style={{
          position: "absolute", top: "calc(100% + 6px)", left: 0,
          background: "var(--bg-2)", border: "1px solid var(--line)",
          borderRadius: 10, padding: "6px 0",
          boxShadow: "0 8px 24px rgba(0,0,0,0.3)",
          minWidth: 180, zIndex: 50,
        }}>
          {providers.length === 0 ? (
            <div style={{ padding: "10px 14px", fontSize: 12, color: "var(--fg-3)" }}>
              No providers configured
            </div>
          ) : providers.map((p) => (
            <button
              key={p.id}
              onClick={() => { onChange(p.id); setOpen(false); }}
              style={{
                width: "100%", padding: "8px 14px",
                background: p.id === value ? "color-mix(in oklch, var(--brand) 12%, transparent)" : "transparent",
                border: "none", cursor: "pointer",
                display: "flex", alignItems: "center", gap: 8,
                color: p.id === value ? "var(--brand)" : "var(--fg)",
                fontFamily: "var(--mg-sans)", fontSize: 13, textAlign: "left",
              }}
            >
              <span style={{ width: 8, height: 8, borderRadius: "50%", background: providerDot(p) }} />
              {p.display_name}
              {p.id === value && <span style={{ marginLeft: "auto", fontSize: 10, color: "var(--brand)" }}>✓</span>}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
