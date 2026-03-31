import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Settings {
  api_key: string;
  base_url: string;
}

interface SettingsPageProps {
  onBack: () => void;
}

export function SettingsPage({ onBack }: SettingsPageProps) {
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
    <main style={{ padding: "2rem", fontFamily: "sans-serif", maxWidth: 500 }}>
      <button onClick={onBack} style={{ marginBottom: "1.5rem", cursor: "pointer" }}>
        ← Back
      </button>
      <h1>Settings</h1>
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
    </main>
  );
}
