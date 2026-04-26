import { useEffect, useRef, useState } from "react";
import { listModels, ModelInfo, ProviderConfig, providerDot } from "../services/tauri";

interface Props {
  provider: ProviderConfig | null;
  value: string | null;
  onChange: (modelId: string) => void;
  disabled?: boolean;
}

export default function ModelPicker({ provider, value, onChange, disabled }: Props) {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!provider) { setModels([]); return; }
    listModels(provider.id).then((ms) => {
      setModels(ms);
      if (ms.length > 0 && !value) onChange(ms[0].id);
    });
  }, [provider?.id]);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const current = models.find((m) => m.id === value) ?? models[0];
  const dot = provider ? providerDot(provider) : "var(--fg-3)";

  return (
    <div ref={ref} style={{ position: "relative" }}>
      <button
        onClick={() => !disabled && setOpen((o) => !o)}
        disabled={disabled || !provider}
        style={{
          display: "flex", alignItems: "center", gap: 6,
          padding: "5px 10px", borderRadius: 20,
          border: "1px solid var(--line)",
          background: open ? "var(--bg-2)" : "transparent",
          color: disabled ? "var(--fg-3)" : "var(--fg)",
          fontFamily: "var(--mg-sans)", fontSize: 12, fontWeight: 500,
          cursor: disabled ? "not-allowed" : "pointer",
          opacity: disabled ? 0.5 : 1,
        }}
      >
        <span style={{ width: 7, height: 7, borderRadius: "50%", background: dot }} />
        <span>{provider?.display_name ?? "No provider"}</span>
        {current && <><span style={{ color: "var(--fg-3)" }}>·</span><span>{current.display_name}</span></>}
        <svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
          <path d={open ? "M1 5l3-3 3 3" : "M1 3l3 3 3-3"} />
        </svg>
      </button>

      {open && models.length > 0 && (
        <div style={{
          position: "absolute", bottom: "calc(100% + 6px)", left: 0,
          background: "var(--bg-2)", border: "1px solid var(--line)",
          borderRadius: 10, padding: "6px 0",
          boxShadow: "0 8px 24px rgba(0,0,0,0.3)",
          minWidth: 200, zIndex: 50,
        }}>
          {models.map((m) => (
            <button
              key={m.id}
              onClick={() => { onChange(m.id); setOpen(false); }}
              style={{
                width: "100%", padding: "8px 14px",
                background: m.id === value ? "color-mix(in oklch, var(--brand) 12%, transparent)" : "transparent",
                border: "none", cursor: "pointer",
                display: "flex", alignItems: "center", gap: 8,
                color: m.id === value ? "var(--brand)" : "var(--fg)",
                fontFamily: "var(--mg-sans)", fontSize: 13, textAlign: "left",
              }}
            >
              {m.id === value && <span style={{ color: "var(--brand)", fontSize: 10 }}>✓</span>}
              <span style={{ flex: 1 }}>{m.display_name}</span>
              <span style={{ fontSize: 10, fontFamily: "var(--mg-mono)", color: "var(--fg-3)" }}>{m.id}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
