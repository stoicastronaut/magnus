import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Settings {
  api_key: string;
  base_url: string;
}

interface SettingsPageProps {
  onBack: () => void;
}

type SettingsSection = "api" | "mcp";

function ApiSettings() {
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState("https://api.anthropic.com");
  const [status, setStatus] = useState("");

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then((s) => {
        setApiKey(s.api_key);
        setBaseUrl(s.base_url);
      })
      .catch(() => {});
  }, []);

  async function handleSave(e: React.FormEvent) {
    e.preventDefault();
    try {
      await invoke("save_settings", { apiKey, baseUrl });
      setStatus("Settings saved.");
    } catch (err) {
      setStatus(`Error: ${err}`);
    }
  }

  async function handleTestConnection() {
    setStatus("Testing...");
    try {
      const result = await invoke<string>("test_connection", { apiKey, baseUrl });
      setStatus(result);
    } catch (err) {
      setStatus(`Error: ${err}`);
    }
  }

  return (
    <div>
      <h2>API Configuration</h2>
      <form onSubmit={handleSave} style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
        <label>
          API Key
          <input
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder="sk-ant-..."
            style={{ display: "block", width: "100%", marginTop: 4 }}
          />
        </label>
        <label>
          Base URL
          <input
            type="text"
            value={baseUrl}
            onChange={(e) => setBaseUrl(e.target.value)}
            style={{ display: "block", width: "100%", marginTop: 4 }}
          />
        </label>
        <button type="submit">Save</button>
        <button type="button" onClick={handleTestConnection}>Test Connection</button>
        {status && <p>{status}</p>}
      </form>
    </div>
  );
}

interface McpServer {
  name: string;
  display_name: string;
  command: string;
  args: string[];
  token?: string;
  env_key?: string;
}

const emptyForm = (): Partial<McpServer> => ({
  display_name: "",
  command: "",
  args: [],
  token: "",
  env_key: "",
});

function McpConnections() {
  const [servers, setServers] = useState<McpServer[]>([]);
  const [connected, setConnected] = useState<string[]>([]);
  const [toolsByServer, setToolsByServer] = useState<Record<string, string[]>>({});
  const [form, setForm] = useState<Partial<McpServer>>(emptyForm());
  const [argsInput, setArgsInput] = useState("");
  const [status, setStatus] = useState<Record<string, string>>({});

  useEffect(() => {
    invoke<McpServer[]>("load_mcp_servers").then(setServers).catch(() => {});
    invoke<string[]>("get_connected_servers").then(setConnected).catch(() => {});
  }, []);

  const persistServers = useCallback(async (updated: McpServer[]) => {
    setServers(updated);
    await invoke("save_mcp_servers", { servers: updated });
  }, []);

  async function handleConnect(server: McpServer) {
    setStatus(s => ({ ...s, [server.name]: "Connecting..." }));
    try {
      await invoke("connect_server", { server });
      const tools = await invoke<{ name: string }[]>("list_tools", { server });
      setConnected(c => [...c, server.name]);
      setToolsByServer(t => ({ ...t, [server.name]: tools.map(t => t.name) }));
      setStatus(s => ({ ...s, [server.name]: "" }));
    } catch (err) {
      setStatus(s => ({ ...s, [server.name]: `Error: ${err}` }));
    }
  }

  async function handleDisconnect(server: McpServer) {
    await invoke("disconnect_server", { serverName: server.name }).catch(() => {});
    setConnected(c => c.filter(n => n !== server.name));
    setToolsByServer(t => { const copy = { ...t }; delete copy[server.name]; return copy; });
  }

  async function handleDelete(server: McpServer) {
    if (connected.includes(server.name)) await handleDisconnect(server);
    await persistServers(servers.filter(s => s.name !== server.name));
  }

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    if (!form.display_name || !form.command) return;
    const name = form.display_name.toLowerCase().replace(/\s+/g, "_");
    const server: McpServer = {
      name,
      display_name: form.display_name!,
      command: form.command!,
      args: argsInput.split(" ").filter(Boolean),
      token: form.token || undefined,
      env_key: form.env_key || undefined,
    };
    await persistServers([...servers, server]);
    setForm(emptyForm());
    setArgsInput("");
  }

  const inputStyle: React.CSSProperties = { display: "block", width: "100%", marginTop: 4, padding: "4px 6px" };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1.5rem" }}>
      <h2 style={{ margin: 0 }}>MCP Connections</h2>

      {servers.length === 0 && <p style={{ color: "#999" }}>No servers configured.</p>}

      {servers.map(server => {
        const isConnected = connected.includes(server.name);
        const tools = toolsByServer[server.name] ?? [];
        return (
          <div key={server.name} style={{ border: "1px solid #ddd", borderRadius: 8, padding: "0.75rem 1rem" }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <div>
                <strong>{server.display_name}</strong>
                <span style={{ marginLeft: 8, color: "#666", fontSize: 13 }}>{server.command} {server.args.join(" ")}</span>
              </div>
              <div style={{ display: "flex", gap: 8 }}>
                {isConnected
                  ? <button onClick={() => handleDisconnect(server)}>Disconnect</button>
                  : <button onClick={() => handleConnect(server)}>Connect</button>}
                <button onClick={() => handleDelete(server)} style={{ color: "red" }}>Delete</button>
              </div>
            </div>
            {status[server.name] && <p style={{ margin: "4px 0 0", fontSize: 13, color: "#666" }}>{status[server.name]}</p>}
            {isConnected && tools.length > 0 && (
              <details style={{ marginTop: 8 }}>
                <summary style={{ cursor: "pointer", fontSize: 13, color: "#555" }}>{tools.length} tools available</summary>
                <ul style={{ margin: "4px 0 0 1rem", padding: 0, fontSize: 13 }}>
                  {tools.map(t => <li key={t}>{t}</li>)}
                </ul>
              </details>
            )}
          </div>
        );
      })}

      <form onSubmit={handleAdd} style={{ border: "1px dashed #aaa", borderRadius: 8, padding: "0.75rem 1rem", display: "flex", flexDirection: "column", gap: "0.75rem" }}>
        <h3 style={{ margin: 0 }}>Add Server</h3>
        <label>Display Name<input style={inputStyle} value={form.display_name} onChange={e => setForm(f => ({ ...f, display_name: e.target.value }))} placeholder="GitHub" /></label>
        <label>Command<input style={inputStyle} value={form.command} onChange={e => setForm(f => ({ ...f, command: e.target.value }))} placeholder="npx" /></label>
        <label>Args (space-separated)<input style={inputStyle} value={argsInput} onChange={e => setArgsInput(e.target.value)} placeholder="-y @modelcontextprotocol/server-github" /></label>
        <label>Env Key<input style={inputStyle} value={form.env_key} onChange={e => setForm(f => ({ ...f, env_key: e.target.value }))} placeholder="GITHUB_PERSONAL_ACCESS_TOKEN" /></label>
        <label>Token<input type="password" style={inputStyle} value={form.token} onChange={e => setForm(f => ({ ...f, token: e.target.value }))} /></label>
        <button type="submit">Add Server</button>
      </form>
    </div>
  );
}

export function SettingsPage({ onBack }: SettingsPageProps) {
  const [section, setSection] = useState<SettingsSection>("api");

  const navStyle = (active: boolean): React.CSSProperties => ({
    display: "block",
    width: "100%",
    textAlign: "left",
    padding: "0.5rem 1rem",
    cursor: "pointer",
    background: active ? "#e0e0e0" : "transparent",
    border: "none",
    fontWeight: active ? "bold" : "normal",
  });

  return (
    <main style={{ display: "flex", height: "100vh", fontFamily: "sans-serif" }}>
      <nav style={{ width: 200, borderRight: "1px solid #ccc", padding: "1rem 0" }}>
        <button onClick={onBack} style={{ padding: "0.5rem 1rem", marginBottom: "1rem", cursor: "pointer" }}>
          ← Back
        </button>
        <button style={navStyle(section === "api")} onClick={() => setSection("api")}>
          API Configuration
        </button>
        <button style={navStyle(section === "mcp")} onClick={() => setSection("mcp")}>
          MCP Connections
        </button>
      </nav>
      <div style={{ flex: 1, padding: "2rem", maxWidth: 500 }}>
        {section === "api" && <ApiSettings />}
        {section === "mcp" && <McpConnections />}
      </div>
    </main>
  );
}
