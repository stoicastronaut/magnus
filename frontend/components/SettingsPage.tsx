import { useState, useEffect, useCallback, useRef } from "react";
import {
  BUILT_IN_PROVIDERS,
  BuiltInId,
  ProviderConfig,
  Settings,
  ModelInfo,
  McpServer,
  getSettings,
  upsertProvider,
  deleteProvider,
  setDefaultProvider,
  listModels,
  hasApiKey,
  loadMcpServers,
  saveMcpServers,
  connectServer,
  disconnectServer,
  getConnectedServers,
  listTools,
  providerDot,
  setMcpToken,
  deleteMcpToken,
} from "../services/tauri";
import ProviderEditModal from "./ProviderEditModal";

type Section = "api" | "mcp" | "appearance" | "chat";

interface Props {
  onBack: () => void;
  theme: "dark" | "light";
  onThemeChange: (t: "dark" | "light") => void;
}

// ── Shared primitives ──────────────────────────────────────────────────────

const inputStyle: React.CSSProperties = {
  width: "100%", padding: "8px 11px",
  borderRadius: 7, border: "1px solid var(--line)",
  background: "var(--bg)", color: "var(--fg)",
  fontFamily: "var(--mg-sans)", fontSize: 13,
  boxSizing: "border-box", outline: "none",
};

const primaryBtn = (extra?: React.CSSProperties): React.CSSProperties => ({
  padding: "7px 14px", borderRadius: 7, border: "none",
  background: "var(--brand)", color: "var(--on-brand)",
  fontFamily: "var(--mg-sans)", fontSize: 12, fontWeight: 500,
  cursor: "pointer", ...extra,
});

const ghostBtn = (extra?: React.CSSProperties): React.CSSProperties => ({
  padding: "7px 12px", borderRadius: 7,
  background: "transparent", border: "1px solid var(--line)",
  color: "var(--fg-2)", fontFamily: "var(--mg-sans)", fontSize: 12, fontWeight: 500,
  cursor: "pointer", ...extra,
});

function SectionHeader({ eyebrow, title, subtitle, right }: {
  eyebrow?: string; title: string; subtitle?: string; right?: React.ReactNode;
}) {
  return (
    <header style={{
      display: "flex", alignItems: "flex-end", justifyContent: "space-between",
      gap: 16, paddingBottom: 20, marginBottom: 28,
      borderBottom: "1px solid var(--line)",
    }}>
      <div>
        {eyebrow && (
          <div style={{
            fontSize: 10, fontFamily: "var(--mg-mono)", fontWeight: 500,
            color: "var(--fg-3)", letterSpacing: "0.12em",
            textTransform: "uppercase", marginBottom: 6,
          }}>{eyebrow}</div>
        )}
        <h1 style={{
          fontSize: 24, fontWeight: 500, letterSpacing: "-0.025em",
          margin: 0, color: "var(--fg)", lineHeight: 1.15,
        }}>{title}</h1>
        {subtitle && (
          <p style={{ fontSize: 13, color: "var(--fg-2)", margin: "6px 0 0", lineHeight: 1.55 }}>
            {subtitle}
          </p>
        )}
      </div>
      {right}
    </header>
  );
}

function FieldGroup({ title, hint, children }: { title: string; hint?: string; children: React.ReactNode }) {
  return (
    <section style={{ marginBottom: 28 }}>
      <h3 style={{
        fontSize: 11, fontFamily: "var(--mg-mono)", fontWeight: 500,
        margin: "0 0 4px", color: "var(--fg-2)",
        textTransform: "uppercase", letterSpacing: "0.1em",
      }}>{title}</h3>
      {hint && <p style={{ fontSize: 12, color: "var(--fg-3)", margin: "0 0 12px", lineHeight: 1.55 }}>{hint}</p>}
      {!hint && <div style={{ height: 10 }} />}
      {children}
    </section>
  );
}

function Toggle({ value, onChange }: { value: boolean; onChange: (v: boolean) => void }) {
  return (
    <button onClick={() => onChange(!value)} style={{
      width: 36, height: 20, borderRadius: 20, border: "none",
      background: value ? "var(--brand)" : "var(--line)",
      position: "relative", cursor: "pointer", padding: 0,
      transition: "background 0.15s", flexShrink: 0,
    }}>
      <span style={{
        position: "absolute", top: 2, left: value ? 18 : 2,
        width: 16, height: 16, borderRadius: 16,
        background: value ? "var(--on-brand)" : "var(--bg-2)",
        transition: "left 0.15s",
        boxShadow: "0 1px 2px rgba(0,0,0,0.15)",
      }} />
    </button>
  );
}

function ToggleRow({ label, hint, value, onChange }: {
  label: string; hint?: string; value: boolean; onChange: (v: boolean) => void;
}) {
  return (
    <div style={{
      display: "flex", alignItems: "center", gap: 14,
      padding: "12px 0", borderBottom: "1px solid var(--line-soft)",
    }}>
      <div style={{ flex: 1 }}>
        <div style={{ fontSize: 13, fontWeight: 500, color: "var(--fg)", marginBottom: 2 }}>{label}</div>
        {hint && <div style={{ fontSize: 12, color: "var(--fg-3)", lineHeight: 1.45 }}>{hint}</div>}
      </div>
      <Toggle value={value} onChange={onChange} />
    </div>
  );
}

function SegmentedControl({ options, value, onChange }: {
  options: [string, string][]; value: string; onChange: (v: string) => void;
}) {
  return (
    <div style={{
      display: "inline-flex", padding: 3, borderRadius: 8,
      background: "var(--bg-2)", border: "1px solid var(--line)",
    }}>
      {options.map(([id, lbl]) => {
        const active = id === value;
        return (
          <button key={id} onClick={() => onChange(id)} style={{
            padding: "6px 14px", borderRadius: 6, border: "none", cursor: "pointer",
            background: active ? "var(--bg)" : "transparent",
            color: active ? "var(--fg)" : "var(--fg-3)",
            fontFamily: "var(--mg-sans)", fontSize: 12, fontWeight: active ? 500 : 400,
            boxShadow: active ? "0 1px 2px rgba(0,0,0,0.15)" : "none",
          }}>{lbl}</button>
        );
      })}
    </div>
  );
}

// ── API section ────────────────────────────────────────────────────────────

type ModalState =
  | { type: "built_in"; which: BuiltInId; existing?: ProviderConfig }
  | { type: "custom"; existing?: ProviderConfig }
  | null;

function ApiSection() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [connected, setConnected] = useState<Record<string, boolean>>({});
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [baseUrlInput, setBaseUrlInput] = useState("");
  const [defaultModel, setDefaultModel] = useState("");
  const [modal, setModal] = useState<ModalState>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const settingsRef = useRef<Settings | null>(null);
  const selectedIdRef = useRef<string | null>(null);

  const selectProvider = useCallback(async (id: string, s?: Settings) => {
    selectedIdRef.current = id;
    setSelectedId(id);
    setApiKeyInput("");
    setError(null);
    const src = s ?? settingsRef.current;
    const p = src?.providers.find((x) => x.id === id);
    if (!p) return;
    if (p.kind === "custom") setBaseUrlInput(p.base_url);
    else setBaseUrlInput("");
    const ms = await listModels(id).catch(() => [] as ModelInfo[]);
    setModels(ms);
    setDefaultModel(ms[0]?.id ?? "");
  }, []);

  const reload = useCallback(async () => {
    const s = await getSettings();
    settingsRef.current = s;
    setSettings(s);
    const checks: Record<string, boolean> = {};
    await Promise.all(s.providers.map(async (p) => {
      checks[p.id] = await hasApiKey(p.id).catch(() => false);
    }));
    // Merge: keep optimistic `true` even if hasApiKey returned false (macOS keychain access can fail intermittently for unsigned dev builds)
    setConnected((prev) => {
      const merged: Record<string, boolean> = {};
      for (const p of s.providers) {
        merged[p.id] = checks[p.id] || prev[p.id] || false;
      }
      return merged;
    });
    if (!selectedIdRef.current && s.providers.length > 0) {
      await selectProvider(s.providers[0].id, s);
    }
  }, [selectProvider]);

  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  useEffect(() => {
    selectedIdRef.current = selectedId;
  }, [selectedId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const handleSave = async () => {
    if (!selectedId || !settings) return;
    const provider = settings.providers.find((p) => p.id === selectedId);
    if (!provider) return;
    setError(null);
    const trimmedKey = apiKeyInput.trim();
    // If provider isn't configured yet, an API key is required
    if (!connected[selectedId] && !trimmedKey) {
      setError("Please enter an API key.");
      return;
    }
    setSaving(true);
    try {
      const updated: ProviderConfig = provider.kind === "custom"
        ? { ...provider, base_url: baseUrlInput }
        : provider;
      await upsertProvider(updated, trimmedKey || null);
      if (trimmedKey) {
        setConnected((c) => ({ ...c, [updated.id]: true }));
      }
      setApiKeyInput("");
      await reload();
    } catch (e) { setError(String(e)); }
    finally { setSaving(false); }
  };

  const handleDelete = async () => {
    if (!selectedId) return;
    setError(null);
    try {
      selectedIdRef.current = null;
      await deleteProvider(selectedId);
      setSelectedId(null);
      await reload();
    } catch (e) { setError(String(e)); }
  };

  const handleSetDefault = async () => {
    if (!selectedId) return;
    try { await setDefaultProvider(selectedId); await reload(); }
    catch (e) { setError(String(e)); }
  };

  const selected = settings?.providers.find((p) => p.id === selectedId) ?? null;
  const isDefault = settings?.default_provider_id === selectedId;
  const isCustom = selected?.kind === "custom";

  // All built-in IDs that are already configured
  const configuredBuiltIns = new Set(
    settings?.providers
      .filter((p) => p.kind === "built_in")
      .map((p) => p.kind === "built_in" ? p.which : null)
      .filter((x): x is BuiltInId => x !== null) ?? []
  );

  return (
    <div>
      {modal && (
        <ProviderEditModal
          mode={modal}
          onClose={() => setModal(null)}
          onSaved={async (providerId, hadKey) => {
            if (hadKey) {
              setConnected((c) => ({ ...c, [providerId]: true }));
            }
            await reload();
            // Auto-select the just-saved provider
            await selectProvider(providerId);
            setModal(null);
          }}
        />
      )}

      <SectionHeader
        eyebrow="Connections"
        title="API Configuration"
        subtitle="Magnus talks to whichever providers you've set up. Keys stay on this device."
      />

      {/* Provider grid */}
      <div style={{
        display: "grid",
        gridTemplateColumns: `repeat(${Math.min((settings?.providers.length ?? 0) + BUILT_IN_PROVIDERS.filter(b => !configuredBuiltIns.has(b.which)).length + 2, 5)}, 1fr)`,
        gap: 8, marginBottom: 24,
      }}>
        {/* Configured providers */}
        {settings?.providers.map((p) => {
          const active = p.id === selectedId;
          const isConn = connected[p.id];
          return (
            <button key={p.id} onClick={() => selectProvider(p.id)} style={{
              padding: "12px 10px", borderRadius: 10,
              border: active ? "1px solid var(--brand)" : "1px solid var(--line)",
              background: active ? "color-mix(in oklch, var(--brand) 10%, var(--bg-2))" : "var(--bg-2)",
              cursor: "pointer", textAlign: "left",
              display: "flex", flexDirection: "column", gap: 6,
              fontFamily: "var(--mg-sans)",
            }}>
              <div style={{ display: "flex", alignItems: "center", gap: 7 }}>
                <span style={{ width: 8, height: 8, borderRadius: "50%", background: providerDot(p) }} />
                <span style={{ fontSize: 12, fontWeight: 500, color: "var(--fg)" }}>{p.display_name}</span>
                {isDefault && p.id === selectedId && (
                  <span style={{ fontSize: 9, fontFamily: "var(--mg-mono)", padding: "1px 4px", borderRadius: 3, background: "color-mix(in oklch, var(--brand) 18%, transparent)", color: "var(--brand)" }}>default</span>
                )}
              </div>
              <span style={{
                fontSize: 10, fontFamily: "var(--mg-mono)",
                color: isConn ? "var(--brand)" : "var(--fg-3)",
              }}>{isConn ? "● connected" : "○ not set"}</span>
            </button>
          );
        })}

        {/* Unconfigured built-ins */}
        {BUILT_IN_PROVIDERS.filter((b) => !configuredBuiltIns.has(b.which)).map((b) => (
          <button key={b.which} onClick={() => setModal({ type: "built_in", which: b.which })} style={{
            padding: "12px 10px", borderRadius: 10,
            border: "1px dashed var(--line)",
            background: "var(--bg-2)",
            cursor: "pointer", textAlign: "left",
            display: "flex", flexDirection: "column", gap: 6,
            fontFamily: "var(--mg-sans)", opacity: 0.7,
          }}>
            <div style={{ display: "flex", alignItems: "center", gap: 7 }}>
              <span style={{ width: 8, height: 8, borderRadius: "50%", background: b.dot }} />
              <span style={{ fontSize: 12, fontWeight: 500, color: "var(--fg)" }}>{b.display_name}</span>
            </div>
            <span style={{ fontSize: 10, fontFamily: "var(--mg-mono)", color: "var(--fg-3)" }}>○ not set</span>
          </button>
        ))}

        {/* Add custom */}
        <button onClick={() => setModal({ type: "custom" })} style={{
          padding: "12px 10px", borderRadius: 10,
          border: "1px dashed var(--line)",
          background: "transparent", cursor: "pointer",
          display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center",
          gap: 4, color: "var(--fg-3)",
        }}>
          <span style={{ fontSize: 18, lineHeight: 1 }}>+</span>
          <span style={{ fontSize: 11, fontFamily: "var(--mg-sans)" }}>Custom</span>
        </button>
      </div>

      {/* Active provider panel */}
      {selected && (
        <div style={{
          background: "var(--bg-2)", border: "1px solid var(--line)",
          borderRadius: 12, padding: 20,
        }}>
          <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 16 }}>
            <span style={{ width: 10, height: 10, borderRadius: "50%", background: providerDot(selected) }} />
            <h3 style={{ fontSize: 16, fontWeight: 500, margin: 0, letterSpacing: "-0.01em" }}>{selected.display_name}</h3>
            <span style={{
              fontSize: 10, fontFamily: "var(--mg-mono)",
              padding: "2px 7px", borderRadius: 5,
              background: connected[selectedId!] ? "color-mix(in oklch, var(--brand) 18%, transparent)" : "var(--bg)",
              color: connected[selectedId!] ? "var(--brand)" : "var(--fg-3)",
            }}>{connected[selectedId!] ? "connected" : "not configured"}</span>
          </div>

          <div style={{ marginBottom: 14 }}>
            <label style={{ display: "block", fontSize: 12, fontWeight: 500, color: "var(--fg-2)", marginBottom: 6 }}>
              API key
            </label>
            <input
              type="password"
              value={apiKeyInput}
              onChange={(e) => setApiKeyInput(e.target.value)}
              placeholder={connected[selectedId!] ? "•••••• (leave blank to keep current)" : "sk-…"}
              style={inputStyle}
            />
          </div>

          {isCustom && (
            <div style={{ marginBottom: 14 }}>
              <label style={{ display: "block", fontSize: 12, fontWeight: 500, color: "var(--fg-2)", marginBottom: 6 }}>
                Base URL
              </label>
              <input
                type="text"
                value={baseUrlInput}
                onChange={(e) => setBaseUrlInput(e.target.value)}
                placeholder="https://…"
                style={inputStyle}
              />
            </div>
          )}

          {models.length > 0 && (
            <div style={{ marginBottom: 14 }}>
              <label style={{ display: "block", fontSize: 12, fontWeight: 500, color: "var(--fg-2)", marginBottom: 6 }}>
                Default model
              </label>
              <select value={defaultModel} onChange={(e) => setDefaultModel(e.target.value)} style={inputStyle}>
                {models.map((m) => <option key={m.id} value={m.id}>{m.display_name}</option>)}
              </select>
            </div>
          )}

          {error && (
            <div style={{
              fontSize: 12, color: "oklch(0.65 0.18 25)",
              padding: "8px 12px", marginBottom: 12,
              background: "color-mix(in oklch, oklch(0.65 0.18 25) 12%, transparent)", borderRadius: 7,
            }}>{error}</div>
          )}

          <div style={{
            display: "flex", justifyContent: "space-between", alignItems: "center",
            paddingTop: 16, marginTop: 8, borderTop: "1px dashed var(--line-soft)",
          }}>
            <span style={{ fontSize: 11, color: "var(--fg-3)", fontFamily: "var(--mg-mono)" }}>
              {models.length} models available
            </span>
            <div style={{ display: "flex", gap: 8 }}>
              {!isDefault && (
                <button onClick={handleSetDefault} style={ghostBtn()}>Set as default</button>
              )}
              <button onClick={handleDelete} style={ghostBtn({ color: "oklch(0.65 0.18 25)" })}>Remove</button>
              <button onClick={handleSave} disabled={saving} style={primaryBtn()}>
                {saving ? "Saving…" : "Save"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ── MCP section ────────────────────────────────────────────────────────────

interface McpServerLocal {
  id: string;
  name: string;
  display_name: string;
  command: string;
  args: string[];
  env_key?: string;
  locally_created?: boolean;
}

interface FormState {
  display_name: string;
  command: string;
  args: string[];
  env_key: string;
  token: string;
}

const emptyForm = (): FormState => ({ display_name: "", command: "", args: [], env_key: "", token: "" });

function ServerGlyph({ name }: { name: string }) {
  return (
    <div style={{
      width: 32, height: 32, borderRadius: 9,
      background: "color-mix(in oklch, var(--brand) 14%, var(--bg))",
      color: "var(--brand)",
      display: "flex", alignItems: "center", justifyContent: "center",
      fontFamily: "var(--mg-sans)", fontWeight: 600, fontSize: 14,
      flexShrink: 0,
    }}>{name.charAt(0).toUpperCase()}</div>
  );
}

const MCP_PRESETS = [
  {
    name: "filesystem",
    display_name: "Filesystem MCP",
    command: "npx",
    args: ["-y", "@modelcontextprotocol/server-filesystem", "<path>"],
    description: "Read, write, and list files with permission checks",
  },
  {
    name: "github",
    display_name: "GitHub MCP",
    command: "npx",
    args: ["-y", "@modelcontextprotocol/server-github"],
    description: "Access GitHub repositories and issues",
  },
  {
    name: "linear",
    display_name: "Linear MCP",
    command: "npx",
    args: ["-y", "@modelcontextprotocol/server-linear"],
    description: "Manage Linear issues and projects",
  },
];

function McpSection() {
  const [servers, setServers] = useState<McpServerLocal[]>([]);
  const [connected, setConnected] = useState<string[]>([]);
  const [toolsByServer, setToolsByServer] = useState<Record<string, string[]>>({});
  const [expandedServer, setExpandedServer] = useState<string | null>(null);
  const [form, setForm] = useState<FormState>(emptyForm());
  const [argsInput, setArgsInput] = useState("");
  const [status, setStatus] = useState<Record<string, string>>({});
  const [showAdd, setShowAdd] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [trustCheckbox, setTrustCheckbox] = useState(false);
  const [resolvedPath, setResolvedPath] = useState<string | null>(null);

  useEffect(() => {
    loadMcpServers().then((ss) => setServers(ss as unknown as McpServerLocal[])).catch(() => {});
    getConnectedServers().then(setConnected).catch(() => {});
  }, []);

  const persist = useCallback(async (updated: McpServerLocal[]) => {
    setServers(updated);
    await saveMcpServers(updated as unknown as McpServer[]);
  }, []);

  async function handleConnect(server: McpServerLocal) {
    setStatus((s) => ({ ...s, [server.name]: "Connecting…" }));
    try {
      await connectServer(server as unknown as McpServer);
      const tools = await listTools(server as unknown as McpServer);
      setConnected((c) => [...c, server.name]);
      setToolsByServer((t) => ({ ...t, [server.name]: tools.map((x) => x.name) }));
      setStatus((s) => ({ ...s, [server.name]: "" }));
    } catch (err) {
      setStatus((s) => ({ ...s, [server.name]: `Error: ${err}` }));
    }
  }

  async function handleDisconnect(server: McpServerLocal) {
    await disconnectServer(server.name).catch(() => {});
    setConnected((c) => c.filter((n) => n !== server.name));
    setToolsByServer((t) => { const c = { ...t }; delete c[server.name]; return c; });
  }

  async function handleDelete(server: McpServerLocal) {
    if (connected.includes(server.name)) await handleDisconnect(server);
    // Clean up token from keychain
    await deleteMcpToken(server.id).catch(() => {});
    await persist(servers.filter((s) => s.name !== server.name));
  }

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    if (!form.display_name || !form.command) return;
    const name = form.display_name.toLowerCase().replace(/\s+/g, "_");
    const server: McpServerLocal = {
      id: crypto.randomUUID(),
      name,
      display_name: form.display_name!,
      command: form.command!,
      args: argsInput.split(" ").filter(Boolean),
      env_key: form.env_key || undefined,
      locally_created: true,
    };
    // Save the server configuration (without token)
    await persist([...servers, server]);
    // Save token to keychain if provided
    if (form.token) {
      await setMcpToken(server.id, form.token).catch(() => {});
    }
    setForm(emptyForm()); setArgsInput(""); setShowAdd(false);
  }

  return (
    <div>
      <SectionHeader
        eyebrow="Extensions"
        title="MCP Connections"
        subtitle="Model Context Protocol servers add tools to every chat."
        right={<button onClick={() => setShowAdd((s) => !s)} style={ghostBtn()}>+ Add server</button>}
      />

      {servers.length === 0 && !showAdd && (
        <div style={{ color: "var(--fg-3)", fontSize: 13 }}>No servers configured.</div>
      )}

      <div style={{ display: "flex", flexDirection: "column", gap: 10, marginBottom: 24 }}>
        {servers.map((server) => {
          const isConn = connected.includes(server.name);
          const tools = toolsByServer[server.name] ?? [];
          const expanded = expandedServer === server.name;
          return (
            <div key={server.name} style={{
              background: "var(--bg-2)", border: "1px solid var(--line)",
              borderRadius: 10, overflow: "hidden",
            }}>
              <div style={{ display: "flex", alignItems: "center", gap: 12, padding: "12px 14px" }}>
                <ServerGlyph name={server.display_name} />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 2 }}>
                    <span style={{ fontSize: 13, fontWeight: 500, color: "var(--fg)" }}>{server.display_name}</span>
                    <span style={{
                      fontSize: 10, fontFamily: "var(--mg-mono)",
                      padding: "2px 6px", borderRadius: 4,
                      background: isConn ? "color-mix(in oklch, var(--brand) 18%, transparent)" : "var(--bg)",
                      color: isConn ? "var(--brand)" : "var(--fg-3)",
                    }}>{isConn ? `● ${tools.length} tools` : "○ idle"}</span>
                  </div>
                  <div style={{ fontSize: 12, color: "var(--fg-3)", fontFamily: "var(--mg-mono)" }}>
                    {server.command} {server.args.join(" ")}
                  </div>
                  {status[server.name] && <div style={{ fontSize: 11, color: "var(--fg-3)", marginTop: 2 }}>{status[server.name]}</div>}
                </div>
                {tools.length > 0 && (
                  <button onClick={() => setExpandedServer(expanded ? null : server.name)} style={ghostBtn()}>
                    {expanded ? "Hide" : "Tools"}
                  </button>
                )}
                {isConn
                  ? <button onClick={() => handleDisconnect(server)} style={ghostBtn()}>Disconnect</button>
                  : <button onClick={() => handleConnect(server)} style={primaryBtn()}>Connect</button>}
                <button onClick={() => handleDelete(server)} style={ghostBtn({ color: "oklch(0.65 0.18 25)" })}>Remove</button>
              </div>
              {expanded && tools.length > 0 && (
                <div style={{
                  padding: "12px 14px 14px 58px",
                  borderTop: "1px solid var(--line-soft)",
                  background: "color-mix(in oklch, var(--bg) 50%, var(--bg-2))",
                }}>
                  <div style={{ fontSize: 10, fontFamily: "var(--mg-mono)", color: "var(--fg-3)", marginBottom: 8, letterSpacing: "0.08em", textTransform: "uppercase" }}>
                    Tools
                  </div>
                  <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
                    {tools.map((t) => (
                      <span key={t} style={{
                        fontSize: 11, fontFamily: "var(--mg-mono)",
                        padding: "3px 8px", borderRadius: 5,
                        background: "var(--bg)", color: "var(--fg-2)",
                        border: "1px solid var(--line-soft)",
                      }}>{t}</span>
                    ))}
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>

      {showAdd && (
        <>
          {!showAdvanced ? (
            <div style={{
              border: "1px solid var(--line)", borderRadius: 12,
              padding: 20, display: "flex", flexDirection: "column", gap: 14,
            }}>
              <h3 style={{ margin: 0, fontSize: 14, fontWeight: 600, color: "var(--fg)" }}>Recommended MCP servers</h3>
              <p style={{ margin: 0, fontSize: 13, color: "var(--fg-2)" }}>Click "Add" to use one of these trusted configurations, or switch to "Advanced" for custom commands.</p>
              <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
                {MCP_PRESETS.map((preset) => (
                  <button
                    key={preset.name}
                    onClick={() => {
                      setForm({
                        display_name: preset.display_name,
                        command: preset.command,
                        args: preset.args,
                        env_key: "",
                        token: "",
                      });
                      setArgsInput(preset.args.join(" "));
                      setTrustCheckbox(false);
                      setResolvedPath(null);
                    }}
                    style={{
                      padding: 12, borderRadius: 8,
                      border: "1px solid var(--line)", background: "var(--bg-2)",
                      cursor: "pointer", textAlign: "left", color: "var(--fg)",
                      fontFamily: "var(--mg-sans)", fontSize: 13,
                    }}
                  >
                    <div style={{ fontWeight: 500, marginBottom: 4 }}>{preset.display_name}</div>
                    <div style={{ fontSize: 12, color: "var(--fg-3)" }}>{preset.description}</div>
                  </button>
                ))}
              </div>
              <div style={{
                padding: 12, borderRadius: 8, background: "color-mix(in oklch, var(--fg-3) 5%, var(--bg))",
                fontSize: 12, color: "var(--fg-2)",
              }}>
                <button
                  onClick={() => setShowAdvanced(true)}
                  style={{
                    background: "none", border: "none", color: "var(--brand)",
                    cursor: "pointer", fontSize: 12, fontWeight: 500, padding: 0,
                  }}
                >
                  Advanced: Add custom command →
                </button>
              </div>
              <button type="button" onClick={() => { setShowAdd(false); setShowAdvanced(false); }} style={ghostBtn()}>Cancel</button>
            </div>
          ) : (
            <form onSubmit={handleAdd} style={{
              border: "1px solid var(--line)", borderRadius: 12,
              padding: 20, display: "flex", flexDirection: "column", gap: 14,
            }}>
              <h3 style={{ margin: 0, fontSize: 14, fontWeight: 600, color: "var(--fg)" }}>Add custom MCP server</h3>
              <div style={{
                padding: 12, borderRadius: 8, background: "oklch(0.65 0.18 25 / 0.1)",
                borderLeft: "3px solid oklch(0.65 0.18 25)",
                fontSize: 12, color: "var(--fg-2)",
              }}>
                ⚠️ This will run an arbitrary command on your machine. Only proceed if you fully trust the source.
              </div>
              {[
                { label: "Display name", key: "display_name" as const, placeholder: "My Custom Server" },
                { label: "Command", key: "command" as const, placeholder: "npx" },
                { label: "Env key", key: "env_key" as const, placeholder: "MY_TOKEN (optional)" },
              ].map(({ label, key, placeholder }) => (
                <div key={key}>
                  <label style={{ display: "block", fontSize: 12, fontWeight: 500, color: "var(--fg-2)", marginBottom: 6 }}>{label}</label>
                  <input style={inputStyle} value={form[key] ?? ""} onChange={(e) => setForm((f) => ({ ...f, [key]: e.target.value }))} placeholder={placeholder} />
                </div>
              ))}
              <div>
                <label style={{ display: "block", fontSize: 12, fontWeight: 500, color: "var(--fg-2)", marginBottom: 6 }}>Args</label>
                <input style={inputStyle} value={argsInput} onChange={(e) => setArgsInput(e.target.value)} placeholder="-y @my-org/custom-server" />
              </div>
              <div>
                <label style={{ display: "block", fontSize: 12, fontWeight: 500, color: "var(--fg-2)", marginBottom: 6 }}>Token (stored securely in keychain)</label>
                <input type="password" style={inputStyle} value={form.token ?? ""} onChange={(e) => setForm((f) => ({ ...f, token: e.target.value }))} placeholder="Leave blank if already stored" />
              </div>
              <label style={{ display: "flex", alignItems: "flex-start", gap: 8, cursor: "pointer" }}>
                <input
                  type="checkbox"
                  checked={trustCheckbox}
                  onChange={(e) => setTrustCheckbox(e.target.checked)}
                  style={{ marginTop: 3 }}
                />
                <span style={{ fontSize: 12, color: "var(--fg-2)" }}>I trust this command to run on my machine</span>
              </label>
              {resolvedPath && (
                <div style={{
                  padding: 10, borderRadius: 6, background: "var(--bg-2)",
                  fontSize: 11, fontFamily: "var(--mg-mono)", color: "var(--fg-3)",
                }}>
                  <div style={{ marginBottom: 4, fontWeight: 500, color: "var(--fg-2)" }}>Resolved path:</div>
                  {resolvedPath}
                </div>
              )}
              <div style={{ display: "flex", gap: 8 }}>
                <button type="submit" disabled={!trustCheckbox} style={{ ...primaryBtn(), opacity: trustCheckbox ? 1 : 0.5, cursor: trustCheckbox ? "pointer" : "not-allowed" }}>Add</button>
                <button type="button" onClick={() => setShowAdvanced(false)} style={ghostBtn()}>Back</button>
                <button type="button" onClick={() => { setShowAdd(false); setShowAdvanced(false); }} style={ghostBtn()}>Cancel</button>
              </div>
            </form>
          )}
        </>
      )}
    </div>
  );
}

// ── Appearance section ─────────────────────────────────────────────────────

function AppearanceSection({ theme, onThemeChange }: { theme: "dark" | "light"; onThemeChange: (t: "dark" | "light") => void }) {
  const [accent, setAccent] = useState(() => localStorage.getItem("mg-accent") ?? "marmalade");
  const [density, setDensity] = useState(() => localStorage.getItem("mg-density") ?? "cozy");
  const [font, setFont] = useState(() => localStorage.getItem("mg-font") ?? "geist");

  const save = (key: string, val: string) => localStorage.setItem(key, val);

  return (
    <div>
      <SectionHeader eyebrow="Look & feel" title="Appearance" subtitle="How Magnus looks while you chat." />

      <FieldGroup title="Theme">
        <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 10 }}>
          {([
            { id: "light" as const, label: "Light", bg: "oklch(0.96 0.022 78)", fg: "oklch(0.30 0.02 60)", accent: "oklch(0.65 0.17 48)" },
            { id: "dark" as const, label: "Dark", bg: "oklch(0.18 0.012 55)", fg: "oklch(0.88 0.015 70)", accent: "oklch(0.72 0.16 52)" },
          ] as const).map((o) => {
            const active = theme === o.id;
            return (
              <button key={o.id} onClick={() => onThemeChange(o.id)} style={{
                padding: 12, borderRadius: 12, cursor: "pointer",
                border: active ? "1px solid var(--brand)" : "1px solid var(--line)",
                background: active ? "color-mix(in oklch, var(--brand) 10%, var(--bg-2))" : "var(--bg-2)",
                fontFamily: "var(--mg-sans)", textAlign: "left",
              }}>
                <div style={{ height: 60, borderRadius: 8, background: o.bg, marginBottom: 10, border: "1px solid var(--line-soft)", position: "relative", overflow: "hidden" }}>
                  <div style={{ position: "absolute", top: 8, left: 8, width: 20, height: 3, borderRadius: 2, background: o.fg, opacity: 0.5 }} />
                  <div style={{ position: "absolute", top: 16, left: 8, width: 40, height: 3, borderRadius: 2, background: o.fg, opacity: 0.2 }} />
                  <div style={{ position: "absolute", bottom: 8, left: 8, width: 24, height: 14, borderRadius: 4, background: o.accent }} />
                </div>
                <span style={{ fontSize: 13, fontWeight: 500, color: active ? "var(--brand)" : "var(--fg)" }}>{o.label}</span>
              </button>
            );
          })}
        </div>
      </FieldGroup>

      <FieldGroup title="Accent" hint="Marmalade is Magnus's default.">
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
          {([
            ["marmalade", "oklch(0.68 0.17 48)", "Marmalade"],
            ["amber", "oklch(0.72 0.17 70)", "Amber"],
            ["rust", "oklch(0.55 0.17 36)", "Rust"],
            ["sage", "oklch(0.68 0.10 150)", "Sage"],
            ["ink", "oklch(0.50 0.05 250)", "Ink"],
          ] as [string, string, string][]).map(([id, col, lbl]) => (
            <button key={id} onClick={() => { setAccent(id); save("mg-accent", id); }} style={{
              display: "flex", alignItems: "center", gap: 8,
              padding: "7px 12px 7px 8px", borderRadius: 999,
              border: accent === id ? "1px solid var(--brand)" : "1px solid var(--line)",
              background: accent === id ? "color-mix(in oklch, var(--brand) 10%, var(--bg-2))" : "var(--bg-2)",
              color: accent === id ? "var(--brand)" : "var(--fg-2)",
              cursor: "pointer", fontFamily: "var(--mg-sans)", fontSize: 12, fontWeight: 500,
            }}>
              <span style={{ width: 14, height: 14, borderRadius: 14, background: col }} />
              {lbl}
            </button>
          ))}
        </div>
      </FieldGroup>

      <FieldGroup title="Density">
        <SegmentedControl options={[["compact", "Compact"], ["cozy", "Cozy"], ["loaf", "Loaf"]]} value={density} onChange={(v) => { setDensity(v); save("mg-density", v); }} />
      </FieldGroup>

      <FieldGroup title="Font">
        <SegmentedControl options={[["geist", "Geist"], ["inter", "Inter"], ["system", "System"]]} value={font} onChange={(v) => { setFont(v); save("mg-font", v); }} />
      </FieldGroup>
    </div>
  );
}

// ── Chat section ───────────────────────────────────────────────────────────

function ChatBehaviorSection() {
  const getBool = (key: string, def: boolean) => localStorage.getItem(key) === null ? def : localStorage.getItem(key) === "true";
  const [streaming, setStreaming] = useState(() => getBool("mg-streaming", true));
  const [autoscroll, setAutoscroll] = useState(() => getBool("mg-autoscroll", true));
  const [purr, setPurr] = useState(() => getBool("mg-purr", true));
  const [enterSend, setEnterSend] = useState(() => getBool("mg-enter-send", true));
  const [autoRename, setAutoRename] = useState(() => getBool("mg-auto-rename", true));

  const save = (key: string, val: boolean) => { localStorage.setItem(key, String(val)); };

  return (
    <div>
      <SectionHeader eyebrow="Behavior" title="Chat" subtitle="How conversations flow and what happens while Magnus is thinking." />

      <FieldGroup title="Streaming">
        <ToggleRow label="Stream responses" hint="Show tokens as they arrive." value={streaming} onChange={(v) => { setStreaming(v); save("mg-streaming", v); }} />
        <ToggleRow label="Auto-scroll while streaming" hint="Follow the bottom of the message. Scrolling manually pauses this." value={autoscroll} onChange={(v) => { setAutoscroll(v); save("mg-autoscroll", v); }} />
        <ToggleRow label="Purring loading state" hint="Show Magnus's breathing waveform while thinking." value={purr} onChange={(v) => { setPurr(v); save("mg-purr", v); }} />
      </FieldGroup>

      <FieldGroup title="Composer">
        <ToggleRow label="Enter to send" hint="Use Shift+Enter for a new line." value={enterSend} onChange={(v) => { setEnterSend(v); save("mg-enter-send", v); }} />
        <ToggleRow label="Auto-rename new chats" hint="Magnus names each chat from the first response." value={autoRename} onChange={(v) => { setAutoRename(v); save("mg-auto-rename", v); }} />
      </FieldGroup>
    </div>
  );
}

// ── Nav item ───────────────────────────────────────────────────────────────

function NavItem({ icon, label, active, badge, onClick }: {
  icon: React.ReactNode; label: string; active: boolean; badge?: number; onClick: () => void;
}) {
  return (
    <button onClick={onClick} style={{
      display: "flex", alignItems: "center", gap: 10,
      width: "100%", padding: "8px 12px",
      borderRadius: 8, border: "none", cursor: "pointer",
      background: active ? "color-mix(in oklch, var(--brand) 14%, transparent)" : "transparent",
      color: active ? "var(--brand)" : "var(--fg-2)",
      fontFamily: "var(--mg-sans)", fontSize: 13, fontWeight: active ? 500 : 400,
      textAlign: "left",
    }}>
      <span style={{ width: 14, display: "flex", alignItems: "center", justifyContent: "center", opacity: active ? 1 : 0.7 }}>{icon}</span>
      <span style={{ flex: 1 }}>{label}</span>
      {badge != null && (
        <span style={{
          fontSize: 10, fontFamily: "var(--mg-mono)",
          padding: "2px 6px", borderRadius: 5,
          background: active ? "color-mix(in oklch, var(--brand) 22%, transparent)" : "var(--bg)",
          color: active ? "var(--brand)" : "var(--fg-3)",
        }}>{badge}</span>
      )}
    </button>
  );
}

function NavGroupLabel({ children }: { children: React.ReactNode }) {
  return (
    <div style={{
      fontSize: 10, fontFamily: "var(--mg-mono)", fontWeight: 500,
      color: "var(--fg-3)", letterSpacing: "0.1em",
      textTransform: "uppercase", padding: "6px 12px 4px",
    }}>{children}</div>
  );
}

// ── Shell ──────────────────────────────────────────────────────────────────

export function SettingsPage({ onBack, theme, onThemeChange }: Props) {
  const [section, setSection] = useState<Section>("api");

  return (
    <div style={{ display: "flex", height: "100vh", background: "var(--bg)", overflow: "hidden" }}>
      {/* Left nav */}
      <aside style={{
        width: 232, flexShrink: 0,
        display: "flex", flexDirection: "column",
        background: "var(--bg-2)", borderRight: "1px solid var(--line)",
      }}>
        <div style={{ padding: "16px 14px 10px" }}>
          <button onClick={onBack} style={{
            display: "flex", alignItems: "center", gap: 8,
            background: "transparent", border: "none", cursor: "pointer",
            color: "var(--fg-3)", fontFamily: "var(--mg-sans)", fontSize: 12,
            padding: "4px 6px", borderRadius: 6,
          }}>
            <span style={{ fontSize: 14 }}>←</span> Back to chat
          </button>
        </div>

        <div style={{ padding: "6px 14px 14px", display: "flex", alignItems: "center", gap: 10 }}>
          <div style={{
            width: 28, height: 28, borderRadius: 8,
            background: "var(--brand)", color: "var(--on-brand)",
            display: "flex", alignItems: "center", justifyContent: "center",
          }}>
            <PawIcon size={15} />
          </div>
          <div style={{ display: "flex", flexDirection: "column", lineHeight: 1.1 }}>
            <span style={{ fontSize: 14, fontWeight: 600, letterSpacing: "-0.01em" }}>Settings</span>
            <span style={{ fontSize: 10, color: "var(--fg-3)", fontFamily: "var(--mg-mono)" }}>Magnus · 0.3</span>
          </div>
        </div>

        <div style={{ padding: "0 10px", display: "flex", flexDirection: "column", gap: 2 }}>
          <NavGroupLabel>Account</NavGroupLabel>
          <NavItem icon={<KeyIcon />} label="API Configuration" active={section === "api"} onClick={() => setSection("api")} />
          <NavItem icon={<PlugIcon />} label="MCP Connections" active={section === "mcp"} onClick={() => setSection("mcp")} />
          <div style={{ height: 10 }} />
          <NavGroupLabel>Preferences</NavGroupLabel>
          <NavItem icon={<PaletteIcon />} label="Appearance" active={section === "appearance"} onClick={() => setSection("appearance")} />
          <NavItem icon={<ChatIcon />} label="Chat" active={section === "chat"} onClick={() => setSection("chat")} />
        </div>

        <div style={{ flex: 1 }} />

        <div style={{
          padding: "14px 16px", borderTop: "1px solid var(--line)",
          display: "flex", alignItems: "center", gap: 8,
          fontSize: 11, color: "var(--fg-3)", fontFamily: "var(--mg-mono)",
        }}>
          <span style={{ color: "var(--brand)" }}>●</span>
          <span>purring since 2024</span>
        </div>
      </aside>

      {/* Main */}
      <main style={{ flex: 1, minWidth: 0, overflowY: "auto", padding: "40px 48px" }}>
        <div style={{ maxWidth: 720, margin: "0 auto" }}>
          {section === "api" && <ApiSection />}
          {section === "mcp" && <McpSection />}
          {section === "appearance" && <AppearanceSection theme={theme} onThemeChange={onThemeChange} />}
          {section === "chat" && <ChatBehaviorSection />}
        </div>
      </main>
    </div>
  );
}

// ── Icons ──────────────────────────────────────────────────────────────────

function PawIcon({ size = 16 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="currentColor">
      <ellipse cx="8" cy="11" rx="3.5" ry="3" />
      <ellipse cx="3.5" cy="7.5" rx="1.8" ry="1.5" />
      <ellipse cx="12.5" cy="7.5" rx="1.8" ry="1.5" />
      <ellipse cx="5.5" cy="5" rx="1.5" ry="1.3" />
      <ellipse cx="10.5" cy="5" rx="1.5" ry="1.3" />
    </svg>
  );
}
function KeyIcon() {
  return <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><circle cx="5.5" cy="10.5" r="2.5" /><path d="M8 10l5-5" /><path d="M11 5l1.5 1.5" /></svg>;
}
function PlugIcon() {
  return <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><path d="M6 2v4" /><path d="M10 2v4" /><rect x="4" y="6" width="8" height="5" rx="1" /><path d="M8 11v3" /></svg>;
}
function PaletteIcon() {
  return <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><path d="M8 14a6 6 0 1 1 6-6c0 1.5-1 2-2 2h-1.5a1.5 1.5 0 0 0 0 3c0 .5.5 1 1 1" /><circle cx="5" cy="7" r=".7" fill="currentColor" /><circle cx="8" cy="4.5" r=".7" fill="currentColor" /><circle cx="11" cy="7" r=".7" fill="currentColor" /></svg>;
}
function ChatIcon() {
  return <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><path d="M3 4.5a1.5 1.5 0 0 1 1.5-1.5h7A1.5 1.5 0 0 1 13 4.5v5a1.5 1.5 0 0 1-1.5 1.5H7l-3 2.5v-2.5h-.5A.5.5 0 0 1 3 10.5z" /></svg>;
}
