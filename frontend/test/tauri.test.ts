import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  Chat,
  ClientEvent,
  ExportOptions,
  McpServer,
  ProviderConfig,
  deleteChat,
  deleteProvider,
  disconnectServer,
  executeToolCall,
  exportDiagnostics,
  getConnectedServers,
  getDiagnosticsSummary,
  getRecentDiagnostics,
  getSessionId,
  getSettings,
  hasApiKey,
  installGlobalDiagnosticsHandlers,
  listModels,
  listTools,
  loadChats,
  loadMcpServers,
  logClientEvent,
  logDiagnosticError,
  providerDisplayName,
  providerDot,
  renameChat,
  revealDiagnosticsFolder,
  revealPath,
  saveChat,
  saveMcpServers,
  setDefaultProvider,
  streamMessage,
  upsertProvider,
  writeClipboardText,
  connectServer,
} from "../services/tauri";

const anthropicProvider: ProviderConfig = {
  id: "anthropic",
  display_name: "Anthropic",
  kind: "built_in",
  which: "anthropic",
};

const customProvider: ProviderConfig = {
  id: "corp-openai",
  display_name: "Corp OpenAI",
  kind: "custom",
  protocol: "open_ai",
  base_url: "https://proxy.example.com/v1/",
};

const chat: Chat = {
  id: "chat-1",
  name: "Planning",
  messages: [{ role: "user", content: "Hello" }],
  created_at: "08-05-26",
  provider_id: "anthropic",
};

const server: McpServer = {
  name: "github",
  command: "npx",
  args: ["-y", "@modelcontextprotocol/server-github"],
  env: { GITHUB_TOKEN: "secret" },
};

describe("tauri service adapter", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue(undefined);
  });

  it("returns provider display helpers without invoking Tauri", () => {
    expect(providerDisplayName(anthropicProvider)).toBe("Anthropic");
    expect(providerDisplayName(customProvider)).toBe("Corp OpenAI");
    expect(providerDot(anthropicProvider)).toBe("oklch(0.68 0.17 48)");
    expect(providerDot(customProvider)).toBe("var(--fg-3)");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("invokes provider settings commands with expected payloads", async () => {
    await getSettings();
    await upsertProvider(anthropicProvider, "sk-ant-test");
    await deleteProvider("anthropic");
    await setDefaultProvider("anthropic");
    await listModels("anthropic");
    await hasApiKey("anthropic");

    expect(invoke).toHaveBeenNthCalledWith(1, "get_settings");
    expect(invoke).toHaveBeenNthCalledWith(2, "upsert_provider", {
      provider: anthropicProvider,
      apiKey: "sk-ant-test",
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "delete_provider", {
      providerId: "anthropic",
    });
    expect(invoke).toHaveBeenNthCalledWith(4, "set_default_provider", {
      providerId: "anthropic",
    });
    expect(invoke).toHaveBeenNthCalledWith(5, "list_models", {
      providerId: "anthropic",
    });
    expect(invoke).toHaveBeenNthCalledWith(6, "has_api_key", {
      providerId: "anthropic",
    });
  });

  it("invokes diagnostics commands with expected payloads", async () => {
    const event: ClientEvent = {
      level: "error",
      kind: "client_error",
      message: "Render failed",
    };
    const options: ExportOptions = {
      includeFullEndpointUrl: false,
      includeActiveChatTranscript: true,
      activeChatId: "chat-1",
    };

    await getSessionId();
    await logClientEvent(event);
    await getRecentDiagnostics(25);
    await getDiagnosticsSummary(options);
    await exportDiagnostics(options);
    await revealDiagnosticsFolder();
    await revealPath("/tmp/magnus-diagnostics.zip");
    await writeClipboardText("summary");

    expect(invoke).toHaveBeenNthCalledWith(1, "get_session_id");
    expect(invoke).toHaveBeenNthCalledWith(2, "log_client_event", {
      event: { ...event, context: {} },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "get_recent_diagnostics", {
      limit: 25,
    });
    expect(invoke).toHaveBeenNthCalledWith(4, "get_diagnostics_summary", {
      options,
    });
    expect(invoke).toHaveBeenNthCalledWith(5, "export_diagnostics", {
      options,
    });
    expect(invoke).toHaveBeenNthCalledWith(6, "reveal_diagnostics_folder");
    expect(invoke).toHaveBeenNthCalledWith(7, "reveal_path", {
      path: "/tmp/magnus-diagnostics.zip",
    });
    expect(invoke).toHaveBeenNthCalledWith(
      8,
      "plugin:clipboard-manager|write_text",
      { text: "summary" }
    );
  });

  it("logs diagnostic errors as fire-and-forget client events", async () => {
    logDiagnosticError("Save failed", { provider_id: "anthropic" });
    await Promise.resolve();

    expect(invoke).toHaveBeenCalledWith("log_client_event", {
      event: {
        level: "error",
        kind: "client_error",
        message: "Save failed",
        context: { provider_id: "anthropic" },
      },
    });
  });

  it("logs global error and unhandled rejection events", async () => {
    installGlobalDiagnosticsHandlers();

    window.dispatchEvent(new ErrorEvent("error", { message: "Render exploded" }));

    const rejection = new Event("unhandledrejection") as PromiseRejectionEvent;
    Object.defineProperty(rejection, "reason", {
      value: new Error("Async exploded"),
    });
    window.dispatchEvent(rejection);

    await Promise.resolve();

    expect(invoke).toHaveBeenNthCalledWith(1, "log_client_event", {
      event: {
        level: "error",
        kind: "client_error",
        message: "Render exploded",
        context: {},
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "log_client_event", {
      event: {
        level: "error",
        kind: "client_error",
        message: "Error: Async exploded",
        context: {},
      },
    });
  });

  it("invokes chat commands with expected payloads", async () => {
    await streamMessage("anthropic", "claude-sonnet-4-6", chat.messages);
    await saveChat(chat);
    await renameChat("anthropic", "claude-sonnet-4-6", chat);
    await loadChats();
    await deleteChat(chat);

    expect(invoke).toHaveBeenNthCalledWith(1, "stream_message", {
      providerId: "anthropic",
      modelId: "claude-sonnet-4-6",
      messages: chat.messages,
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "save_chat", { chat });
    expect(invoke).toHaveBeenNthCalledWith(3, "rename_chat", {
      providerId: "anthropic",
      modelId: "claude-sonnet-4-6",
      chat,
    });
    expect(invoke).toHaveBeenNthCalledWith(4, "load_chats");
    expect(invoke).toHaveBeenNthCalledWith(5, "delete_chat", { chat });
  });

  it("invokes MCP commands with expected payloads", async () => {
    await loadMcpServers();
    await saveMcpServers([server]);
    await connectServer(server);
    await disconnectServer("github");
    await getConnectedServers();
    await listTools(server);
    await executeToolCall("github", "search_repositories", {
      query: "magnus",
    });

    expect(invoke).toHaveBeenNthCalledWith(1, "load_mcp_servers");
    expect(invoke).toHaveBeenNthCalledWith(2, "save_mcp_servers", {
      servers: [server],
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "connect_server", { server });
    expect(invoke).toHaveBeenNthCalledWith(4, "disconnect_server", {
      serverName: "github",
    });
    expect(invoke).toHaveBeenNthCalledWith(5, "get_connected_servers");
    expect(invoke).toHaveBeenNthCalledWith(6, "list_tools", { server });
    expect(invoke).toHaveBeenNthCalledWith(7, "execute_tool_call", {
      serverName: "github",
      toolName: "search_repositories",
      arguments: { query: "magnus" },
    });
  });
});
