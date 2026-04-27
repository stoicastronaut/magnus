import { useState } from "react";
import {
  BUILT_IN_PROVIDERS,
  BuiltInId,
  Protocol,
  ProviderConfig,
  upsertProvider,
} from "../services/tauri";

type Mode =
  | { type: "built_in"; which: BuiltInId; existing?: ProviderConfig }
  | { type: "custom"; existing?: ProviderConfig };

interface Props {
  mode: Mode;
  onClose: () => void;
  onSaved: (providerId: string, hadKey: boolean) => void;
}

export default function ProviderEditModal({ mode, onClose, onSaved }: Props) {
  const isCustom = mode.type === "custom";
  const builtInMeta =
    mode.type === "built_in"
      ? BUILT_IN_PROVIDERS.find((p) => p.which === mode.which)
      : null;

  const [displayName, setDisplayName] = useState(
    mode.existing?.display_name ?? builtInMeta?.display_name ?? ""
  );
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState(() => {
    if (mode.existing?.kind === "custom") return mode.existing.base_url;
    return "";
  });
  const [protocol, setProtocol] = useState<Protocol>(() => {
    if (mode.existing?.kind === "custom") return mode.existing.protocol;
    return "anthropic";
  });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hostWarning, setHostWarning] = useState<string | null>(null);

  // Mirror validation for base URL on the frontend
  const validateBaseUrl = (url: string): string | null => {
    if (!url.trim()) return null; // Empty is caught in handleSave

    try {
      const parsed = new URL(url);

      // Check scheme
      if (parsed.protocol !== "https:" && parsed.protocol !== "http:") {
        return `Unsupported scheme: ${parsed.protocol}`;
      }

      // Check HTTP restriction to localhost only
      if (parsed.protocol === "http:") {
        const host = parsed.hostname;
        if (host !== "localhost" && host !== "127.0.0.1" && host !== "[::1]") {
          return "http:// is only allowed for localhost, 127.0.0.1, or [::1]";
        }
      }

      // Check for embedded credentials
      if (parsed.username || parsed.password) {
        return "URL must not embed credentials";
      }

      // Check for fragment
      if (parsed.hash) {
        return "URL must not have a fragment";
      }

      return null;
    } catch {
      return "Invalid URL format";
    }
  };

  // Check for non-standard hosts and show warning
  const checkHostWarning = (url: string) => {
    try {
      const parsed = new URL(url);
      const host = parsed.hostname || "";
      const standardHosts = [
        "api.anthropic.com",
        "api.openai.com",
        "generativelanguage.googleapis.com",
      ];
      if (!standardHosts.includes(host)) {
        setHostWarning(`Your API key will be sent to ${host}. Continue only if you trust this endpoint.`);
      } else {
        setHostWarning(null);
      }
    } catch {
      setHostWarning(null);
    }
  };

  const handleSave = async () => {
    setError(null);
    if (isCustom && !displayName.trim()) { setError("Display name is required."); return; }
    if (isCustom && !baseUrl.trim()) { setError("Base URL is required."); return; }

    // Validate base URL format on frontend
    if (isCustom) {
      const urlError = validateBaseUrl(baseUrl);
      if (urlError) { setError(urlError); return; }
    }

    // For brand-new providers, require an API key
    if (!mode.existing && !apiKey.trim()) { setError("API key is required."); return; }

    const config: ProviderConfig = isCustom
      ? {
          id: mode.existing?.id ?? crypto.randomUUID(),
          display_name: displayName.trim(),
          kind: "custom", protocol, base_url: baseUrl.trim(),
        }
      : {
          id: mode.which!,
          display_name: builtInMeta!.display_name,
          kind: "built_in", which: mode.which!,
        };

    setSaving(true);
    try {
      const trimmedKey = apiKey.trim();
      await upsertProvider(config, trimmedKey || null);
      onSaved(config.id, !!trimmedKey);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div
      onClick={onClose}
      style={{
        position: "fixed", inset: 0, zIndex: 100,
        background: "rgba(0,0,0,0.5)", backdropFilter: "blur(4px)",
        display: "flex", alignItems: "center", justifyContent: "center",
      }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          width: 440, background: "var(--bg-2)",
          border: "1px solid var(--line)", borderRadius: 14,
          padding: 28, display: "flex", flexDirection: "column", gap: 20,
          boxShadow: "0 20px 60px rgba(0,0,0,0.4)",
        }}
      >
        {/* Header */}
        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
          {builtInMeta && (
            <span style={{ width: 10, height: 10, borderRadius: "50%", background: builtInMeta.dot }} />
          )}
          <h2 style={{ margin: 0, fontSize: 17, fontWeight: 600, letterSpacing: "-0.01em" }}>
            {mode.existing ? "Edit" : "Add"} {isCustom ? "custom provider" : builtInMeta?.display_name}
          </h2>
          <button
            onClick={onClose}
            style={{
              marginLeft: "auto", background: "none", border: "none",
              color: "var(--fg-3)", cursor: "pointer", fontSize: 18, lineHeight: 1,
            }}
          >×</button>
        </div>

        {/* Custom: display name */}
        {isCustom && (
          <FormRow label="Display name">
            <input
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder="My Proxy"
              style={inputStyle}
            />
          </FormRow>
        )}

        {/* Custom: protocol */}
        {isCustom && (
          <FormRow label="Protocol">
            <div style={{ display: "flex", gap: 8 }}>
              {(["anthropic", "open_ai", "google"] as Protocol[]).map((p) => (
                <button
                  key={p}
                  onClick={() => setProtocol(p)}
                  style={{
                    padding: "6px 12px", borderRadius: 7,
                    border: protocol === p ? "1px solid var(--brand)" : "1px solid var(--line)",
                    background: protocol === p ? "color-mix(in oklch, var(--brand) 12%, var(--bg-2))" : "var(--bg-2)",
                    color: protocol === p ? "var(--brand)" : "var(--fg-2)",
                    fontFamily: "var(--mg-sans)", fontSize: 12, fontWeight: 500, cursor: "pointer",
                  }}
                >
                  {p === "open_ai" ? "OpenAI" : p.charAt(0).toUpperCase() + p.slice(1)}
                </button>
              ))}
            </div>
          </FormRow>
        )}

        {/* Custom: base URL */}
        {isCustom && (
          <FormRow label="Base URL">
            <input
              value={baseUrl}
              onChange={(e) => {
                setBaseUrl(e.target.value);
                checkHostWarning(e.target.value);
              }}
              placeholder="https://…"
              style={inputStyle}
            />
          </FormRow>
        )}

        {/* API key */}
        <FormRow label={isCustom ? "API key" : "API key"}>
          <input
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder={mode.existing ? "•••••• (leave blank to keep current)" : builtInMeta?.placeholder ?? "sk-…"}
            style={inputStyle}
          />
        </FormRow>

        {error && (
          <div style={{ fontSize: 12, color: "oklch(0.65 0.18 25)", padding: "8px 12px", background: "color-mix(in oklch, oklch(0.65 0.18 25) 12%, transparent)", borderRadius: 7 }}>
            {error}
          </div>
        )}

        {hostWarning && (
          <div style={{ fontSize: 12, color: "oklch(0.72 0.15 60)", padding: "8px 12px", background: "color-mix(in oklch, oklch(0.72 0.15 60) 12%, transparent)", borderRadius: 7 }}>
            {hostWarning}
          </div>
        )}

        {/* Actions */}
        <div style={{ display: "flex", gap: 8, justifyContent: "flex-end", paddingTop: 4 }}>
          <button onClick={onClose} style={ghostBtn}>Cancel</button>
          <button onClick={handleSave} disabled={saving} style={primaryBtn}>
            {saving ? "Saving…" : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}

function FormRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <label style={{ display: "block", fontSize: 12, fontWeight: 500, color: "var(--fg-2)", marginBottom: 6 }}>
        {label}
      </label>
      {children}
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  width: "100%", padding: "8px 11px",
  borderRadius: 7, border: "1px solid var(--line)",
  background: "var(--bg)", color: "var(--fg)",
  fontFamily: "var(--mg-sans)", fontSize: 13,
  boxSizing: "border-box", outline: "none",
};

const primaryBtn: React.CSSProperties = {
  padding: "8px 16px", borderRadius: 7, border: "none",
  background: "var(--brand)", color: "var(--on-brand)",
  fontFamily: "var(--mg-sans)", fontSize: 13, fontWeight: 500, cursor: "pointer",
};

const ghostBtn: React.CSSProperties = {
  padding: "8px 14px", borderRadius: 7,
  background: "transparent", border: "1px solid var(--line)",
  color: "var(--fg-2)", fontFamily: "var(--mg-sans)", fontSize: 13, cursor: "pointer",
};
