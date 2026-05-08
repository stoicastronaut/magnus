import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsPage } from "../components/SettingsPage";
import {
  deleteProvider,
  connectServer,
  disconnectServer,
  exportDiagnostics,
  getConnectedServers,
  getDiagnosticsSummary,
  getRecentDiagnostics,
  getSettings,
  hasApiKey,
  listModels,
  listTools,
  loadMcpServers,
  logDiagnosticError,
  revealDiagnosticsFolder,
  revealPath,
  saveMcpServers,
  setDefaultProvider,
  upsertProvider,
  writeClipboardText,
} from "../services/tauri";

vi.mock("../services/tauri", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../services/tauri")>();
  return {
    ...actual,
    getRecentDiagnostics: vi.fn(),
    getDiagnosticsSummary: vi.fn(),
    exportDiagnostics: vi.fn(),
    revealDiagnosticsFolder: vi.fn(),
    revealPath: vi.fn(),
    writeClipboardText: vi.fn(),
    getSettings: vi.fn(),
    hasApiKey: vi.fn(),
    listModels: vi.fn(),
    upsertProvider: vi.fn(),
    deleteProvider: vi.fn(),
    setDefaultProvider: vi.fn(),
    logDiagnosticError: vi.fn(),
    loadMcpServers: vi.fn(),
    saveMcpServers: vi.fn(),
    connectServer: vi.fn(),
    disconnectServer: vi.fn(),
    getConnectedServers: vi.fn(),
    listTools: vi.fn(),
  };
});

const settings = {
  default_provider_id: "anthropic",
  providers: [
    {
      id: "anthropic",
      display_name: "Anthropic",
      kind: "built_in" as const,
      which: "anthropic" as const,
    },
    {
      id: "open_ai",
      display_name: "OpenAI",
      kind: "built_in" as const,
      which: "open_ai" as const,
    },
  ],
};

const models = [
  { id: "claude-haiku-4-5-20251001", display_name: "Haiku 4.5" },
  { id: "claude-sonnet-4-6", display_name: "Sonnet 4.6" },
];

const openAiModels = [
  { id: "gpt-5", display_name: "GPT-5" },
  { id: "gpt-5-mini", display_name: "GPT-5 mini" },
];

const githubServer = {
  name: "github",
  display_name: "GitHub",
  command: "npx",
  args: ["-y", "@modelcontextprotocol/server-github"],
  token: "ghp_test",
  env_key: "GITHUB_TOKEN",
};

const diagnosticEvents = [
  {
    timestamp: "2026-05-08T09:00:00.000Z",
    level: "error" as const,
    source: "backend" as const,
    kind: "command_failed" as const,
    message: "Provider request failed",
    context: { provider_id: "anthropic", status: 500 },
  },
];

function renderSettingsPage() {
  return render(
    <SettingsPage
      onBack={vi.fn()}
      theme="dark"
      onThemeChange={vi.fn()}
      activeChatId="chat-1"
    />
  );
}

describe("SettingsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(getRecentDiagnostics).mockResolvedValue([]);
    vi.mocked(getDiagnosticsSummary).mockResolvedValue("diagnostics summary");
    vi.mocked(exportDiagnostics).mockResolvedValue({
      path: "/tmp/magnus-diagnostics.zip",
      summary: "exported",
      included: {
        fullEndpointUrl: true,
        activeChatTranscript: true,
      },
    });
    vi.mocked(revealDiagnosticsFolder).mockResolvedValue(undefined);
    vi.mocked(revealPath).mockResolvedValue(undefined);
    vi.mocked(writeClipboardText).mockResolvedValue(undefined);
    vi.mocked(getSettings).mockResolvedValue(settings);
    vi.mocked(hasApiKey).mockImplementation(async (providerId) => providerId === "anthropic");
    vi.mocked(listModels).mockImplementation(async (providerId) =>
      providerId === "open_ai" ? openAiModels : models
    );
    vi.mocked(upsertProvider).mockResolvedValue(undefined);
    vi.mocked(deleteProvider).mockResolvedValue(undefined);
    vi.mocked(setDefaultProvider).mockResolvedValue(undefined);
    vi.mocked(loadMcpServers).mockResolvedValue([]);
    vi.mocked(saveMcpServers).mockResolvedValue(undefined);
    vi.mocked(connectServer).mockResolvedValue(undefined);
    vi.mocked(disconnectServer).mockResolvedValue(undefined);
    vi.mocked(getConnectedServers).mockResolvedValue([]);
    vi.mocked(listTools).mockResolvedValue([
      { name: "search_repositories", description: "Search repositories" },
      { name: "get_issue", description: "Get issue" },
    ]);
    vi.stubGlobal("crypto", {
      ...crypto,
      randomUUID: vi.fn(() => "custom-provider-id"),
    });
  });

  it("loads providers, API key status, and models for the selected provider", async () => {
    renderSettingsPage();

    expect(screen.getByRole("heading", { name: "API Configuration" })).toBeInTheDocument();

    await waitFor(() => {
      expect(getSettings).toHaveBeenCalled();
    });

    expect(hasApiKey).toHaveBeenCalledWith("anthropic");
    expect(hasApiKey).toHaveBeenCalledWith("open_ai");
    expect(listModels).toHaveBeenCalledWith("anthropic");

    expect(screen.getAllByText("Anthropic").length).toBeGreaterThan(0);
    expect(screen.getAllByText("OpenAI").length).toBeGreaterThan(0);
    expect(screen.getByText("default")).toBeInTheDocument();
    expect(screen.getAllByText(/connected/).length).toBeGreaterThan(0);
    expect(screen.getAllByText("○ not set").length).toBeGreaterThan(0);

    await waitFor(() => {
      expect(screen.getByText("Default model")).toBeInTheDocument();
    });
    expect(screen.getByRole("option", { name: "Haiku 4.5" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Sonnet 4.6" })).toBeInTheDocument();
    expect(screen.getByText("2 models available")).toBeInTheDocument();
  });

  it("selects another provider and loads that provider's models", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /openai/i })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /openai/i }));

    await waitFor(() => {
      expect(listModels).toHaveBeenCalledWith("open_ai");
    });
    expect(screen.getByRole("heading", { name: "OpenAI" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "GPT-5" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "GPT-5 mini" })).toBeInTheDocument();
  });

  it("requires an API key before saving a provider without a stored key", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /openai/i })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /openai/i }));
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(screen.getByText("Please enter an API key.")).toBeInTheDocument();
    expect(upsertProvider).not.toHaveBeenCalled();
  });

  it("saves an API key for the selected provider and reloads settings", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByPlaceholderText("•••••• (leave blank to keep current)")).toBeInTheDocument();
    });
    await userEvent.type(
      screen.getByPlaceholderText("•••••• (leave blank to keep current)"),
      "sk-ant-new"
    );
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(upsertProvider).toHaveBeenCalledWith(settings.providers[0], "sk-ant-new");
    });
    expect(getSettings).toHaveBeenCalledTimes(2);
  });

  it("sets the selected provider as default and reloads settings", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /openai/i })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /openai/i }));
    await userEvent.click(screen.getByRole("button", { name: "Set as default" }));

    await waitFor(() => {
      expect(setDefaultProvider).toHaveBeenCalledWith("open_ai");
    });
    expect(getSettings).toHaveBeenCalledTimes(2);
  });

  it("deletes the selected provider and reloads settings", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /openai/i })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /openai/i }));
    await userEvent.click(screen.getByRole("button", { name: "Remove" }));

    await waitFor(() => {
      expect(deleteProvider).toHaveBeenCalledWith("open_ai");
    });
    expect(getSettings).toHaveBeenCalledTimes(2);
  });

  it("logs diagnostics when provider save, default update, or delete fail", async () => {
    vi.mocked(upsertProvider).mockRejectedValueOnce(new Error("save failed"));
    vi.mocked(setDefaultProvider).mockRejectedValueOnce(new Error("default failed"));
    vi.mocked(deleteProvider).mockRejectedValueOnce(new Error("delete failed"));

    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByPlaceholderText("•••••• (leave blank to keep current)")).toBeInTheDocument();
    });
    await userEvent.type(
      screen.getByPlaceholderText("•••••• (leave blank to keep current)"),
      "sk-ant-new"
    );
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(logDiagnosticError).toHaveBeenCalledWith("Provider save failed", {
        provider_id: "anthropic",
        error: "Error: save failed",
      });
    });

    await userEvent.click(screen.getByRole("button", { name: /openai/i }));
    await userEvent.click(screen.getByRole("button", { name: "Set as default" }));

    await waitFor(() => {
      expect(logDiagnosticError).toHaveBeenCalledWith("Default provider update failed", {
        provider_id: "open_ai",
        error: "Error: default failed",
      });
    });

    await userEvent.click(screen.getByRole("button", { name: "Remove" }));

    await waitFor(() => {
      expect(logDiagnosticError).toHaveBeenCalledWith("Provider delete failed", {
        provider_id: "open_ai",
        error: "Error: delete failed",
    });
  });
  });

  async function clickToggle(label: string) {
    const row = screen.getByText(label).closest("div")?.parentElement?.parentElement;
    const button = row?.querySelector("button");
    if (!button) throw new Error(`Toggle not found for ${label}`);
    await userEvent.click(button);
  }

  it("loads diagnostics, copies event details, exports, and reveals files", async () => {
    vi.mocked(getRecentDiagnostics).mockResolvedValue(diagnosticEvents);

    renderSettingsPage();

    await userEvent.click(screen.getByRole("button", { name: /diagnostics/i }));

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Diagnostics" })).toBeInTheDocument();
    });
    expect(getRecentDiagnostics).toHaveBeenCalledWith(200);
    expect(screen.getByText("Provider request failed")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Details" }));
    expect(screen.getByText(/provider_id/)).toBeInTheDocument();
    expect(screen.getByText(/anthropic/)).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Copy" }));
    await waitFor(() => {
      expect(writeClipboardText).toHaveBeenCalledWith(
        JSON.stringify(diagnosticEvents[0], null, 2)
      );
    });
    expect(screen.getByText("Error details copied.")).toBeInTheDocument();

    await clickToggle("Include full endpoint URL");
    await clickToggle("Include current chat transcript");
    await userEvent.click(screen.getByRole("button", { name: "Copy summary" }));

    await waitFor(() => {
      expect(getDiagnosticsSummary).toHaveBeenCalledWith({
        includeFullEndpointUrl: true,
        includeActiveChatTranscript: true,
        activeChatId: "chat-1",
      });
    });
    expect(writeClipboardText).toHaveBeenCalledWith("diagnostics summary");
    expect(screen.getByText("Summary copied.")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Export diagnostics" }));

    await waitFor(() => {
      expect(exportDiagnostics).toHaveBeenCalledWith({
        includeFullEndpointUrl: true,
        includeActiveChatTranscript: true,
        activeChatId: "chat-1",
      });
    });
    expect(revealPath).toHaveBeenCalledWith("/tmp/magnus-diagnostics.zip");
    expect(screen.getByText("Diagnostics exported.")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Reveal exported file" }));
    expect(revealPath).toHaveBeenCalledWith("/tmp/magnus-diagnostics.zip");

    await userEvent.click(screen.getByRole("button", { name: "Reveal diagnostics folder" }));
    expect(revealDiagnosticsFolder).toHaveBeenCalled();
  });

  it("opens an unconfigured built-in provider modal and saves it", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /google/i })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /google/i }));

    expect(screen.getByRole("heading", { name: "Add Google" })).toBeInTheDocument();
    await userEvent.type(screen.getByPlaceholderText("AIzaSy…"), "google-key");
    await userEvent.click(screen.getAllByRole("button", { name: "Save" })[0]);

    await waitFor(() => {
      expect(upsertProvider).toHaveBeenCalledWith(
        {
          id: "google",
          display_name: "Google",
          kind: "built_in",
          which: "google",
        },
        "google-key"
      );
    });
  });

  it("validates required custom provider fields", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /custom/i })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /custom/i }));
    await userEvent.click(screen.getAllByRole("button", { name: "Save" })[0]);

    expect(screen.getByText("Display name is required.")).toBeInTheDocument();
    expect(upsertProvider).not.toHaveBeenCalled();

    await userEvent.type(screen.getByPlaceholderText("My Proxy"), "Corp Gateway");
    await userEvent.click(screen.getAllByRole("button", { name: "Save" })[0]);

    expect(screen.getByText("Base URL is required.")).toBeInTheDocument();
    expect(upsertProvider).not.toHaveBeenCalled();
  });

  it("creates a custom provider with selected protocol, base URL, and API key", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /custom/i })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /custom/i }));
    await userEvent.type(screen.getByPlaceholderText("My Proxy"), "Corp Gateway");
    await userEvent.click(screen.getByRole("button", { name: "OpenAI" }));
    await userEvent.type(screen.getByPlaceholderText("https://…"), "https://proxy.example.com/v1/");
    await userEvent.type(screen.getByPlaceholderText("sk-…"), "proxy-key");
    await userEvent.click(screen.getAllByRole("button", { name: "Save" })[0]);

    await waitFor(() => {
      expect(upsertProvider).toHaveBeenCalledWith(
        {
          id: "custom-provider-id",
          display_name: "Corp Gateway",
          kind: "custom",
          protocol: "open_ai",
          base_url: "https://proxy.example.com/v1/",
        },
        "proxy-key"
      );
    });
  });

  it("loads saved MCP servers when opening the MCP section", async () => {
    vi.mocked(loadMcpServers).mockResolvedValue([githubServer]);

    renderSettingsPage();

    await userEvent.click(screen.getByRole("button", { name: /mcp connections/i }));

    await waitFor(() => {
      expect(loadMcpServers).toHaveBeenCalled();
    });
    expect(screen.getByRole("heading", { name: "MCP Connections" })).toBeInTheDocument();
    expect(screen.getByText("GitHub")).toBeInTheDocument();
    expect(screen.getByText("npx -y @modelcontextprotocol/server-github")).toBeInTheDocument();
    expect(screen.getByText("○ idle")).toBeInTheDocument();
  });

  it("adds an MCP server and persists the updated server list", async () => {
    const { container } = renderSettingsPage();

    await userEvent.click(screen.getByRole("button", { name: /mcp connections/i }));
    await userEvent.click(screen.getByRole("button", { name: "+ Add server" }));
    await userEvent.type(screen.getByPlaceholderText("GitHub"), "GitHub");
    await userEvent.type(screen.getByPlaceholderText("npx"), "npx");
    await userEvent.type(screen.getByPlaceholderText("GITHUB_TOKEN"), "GITHUB_TOKEN");
    await userEvent.type(
      screen.getByPlaceholderText("-y @modelcontextprotocol/server-github"),
      "-y @modelcontextprotocol/server-github"
    );
    await userEvent.type(container.querySelector('input[type="password"]')!, "ghp_test");
    await userEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() => {
      expect(saveMcpServers).toHaveBeenCalledWith([githubServer]);
    });
    expect(screen.queryByRole("heading", { name: "Add server" })).not.toBeInTheDocument();
  });

  it("connects an MCP server, displays tools, and disconnects it", async () => {
    vi.mocked(loadMcpServers).mockResolvedValue([githubServer]);

    renderSettingsPage();

    await userEvent.click(screen.getByRole("button", { name: /mcp connections/i }));
    await waitFor(() => {
      expect(screen.getByText("GitHub")).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: "Connect" }));

    await waitFor(() => {
      expect(connectServer).toHaveBeenCalledWith(githubServer);
    });
    expect(listTools).toHaveBeenCalledWith(githubServer);
    expect(screen.getByText("● 2 tools")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Tools" }));
    expect(screen.getByText("search_repositories")).toBeInTheDocument();
    expect(screen.getByText("get_issue")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Disconnect" }));

    await waitFor(() => {
      expect(disconnectServer).toHaveBeenCalledWith("github");
    });
    expect(screen.getByText("○ idle")).toBeInTheDocument();
  });

  it("removes an MCP server and persists the updated list", async () => {
    vi.mocked(loadMcpServers).mockResolvedValue([githubServer]);

    renderSettingsPage();

    await userEvent.click(screen.getByRole("button", { name: /mcp connections/i }));
    await waitFor(() => {
      expect(screen.getByText("GitHub")).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: "Remove" }));

    await waitFor(() => {
      expect(saveMcpServers).toHaveBeenCalledWith([]);
    });
    expect(screen.queryByText("GitHub")).not.toBeInTheDocument();
  });
});
