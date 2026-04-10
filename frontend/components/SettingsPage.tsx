import { useState, useEffect } from "react";
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

function McpConnections() {
  const [status, setStatus] = useState("");

  async function handleTest() {
    setStatus("Connecting...");
    try {
      await invoke("test_mcp_connection");
      setStatus("Done — check the terminal for output.");
    } catch (err) {
      setStatus(`Error: ${err}`);
    }
  }

  return (
    <div>
      <h2>MCP Connections</h2>
      <button onClick={handleTest}>Test MCP Connection</button>
      {status && <p>{status}</p>}
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
