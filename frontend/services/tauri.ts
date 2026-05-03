import { invoke } from "@tauri-apps/api/core";

// ── Types ──────────────────────────────────────────────────────────────────

export type BuiltInId = "anthropic" | "open_ai" | "google";
export type Protocol = "anthropic" | "open_ai" | "google";

export type ProviderConfig = { id: string; display_name: string } & (
  | { kind: "built_in"; which: BuiltInId }
  | { kind: "custom"; protocol: Protocol; base_url: string }
);

export interface ModelInfo {
  id: string;
  display_name: string;
}

export interface Settings {
  default_provider_id: string | null;
  providers: ProviderConfig[];
}

export interface Message {
  role: string;
  content: string;
  model_id?: string;
}

export interface Chat {
  id: string;
  name: string;
  messages: Message[];
  created_at: string;
  provider_id: string;
}

export interface McpServer {
  name: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
}

export type DiagnosticLevel = "info" | "warn" | "error" | "fatal";
export type DiagnosticKind =
  | "app_lifecycle"
  | "command_failed"
  | "client_error"
  | "panic"
  | "dropped_events";

export interface ClientEvent {
  level: DiagnosticLevel;
  kind: DiagnosticKind;
  message: string;
  context?: Record<string, unknown>;
}

export interface DiagnosticEvent {
  timestamp: string;
  level: DiagnosticLevel;
  source: "backend" | "frontend";
  kind: DiagnosticKind;
  message: string;
  context: Record<string, unknown>;
}

export interface ExportOptions {
  includeFullEndpointUrl: boolean;
  includeActiveChatTranscript: boolean;
  activeChatId: string | null;
}

export interface ExportResult {
  path: string;
  summary: string;
  included: {
    fullEndpointUrl: boolean;
    activeChatTranscript: boolean;
  };
}

// ── Provider helpers ───────────────────────────────────────────────────────

export const BUILT_IN_PROVIDERS: Array<{
  which: BuiltInId;
  display_name: string;
  dot: string;
  placeholder: string;
}> = [
  {
    which: "anthropic",
    display_name: "Anthropic",
    dot: "oklch(0.68 0.17 48)",
    placeholder: "sk-ant-…",
  },
  {
    which: "open_ai",
    display_name: "OpenAI",
    dot: "oklch(0.72 0.17 155)",
    placeholder: "sk-…",
  },
  {
    which: "google",
    display_name: "Google",
    dot: "oklch(0.66 0.17 250)",
    placeholder: "AIzaSy…",
  },
];

export function providerDot(provider: ProviderConfig): string {
  if (provider.kind === "built_in") {
    return BUILT_IN_PROVIDERS.find((p) => p.which === provider.which)?.dot ?? "var(--fg-3)";
  }
  return "var(--fg-3)";
}

export function providerDisplayName(provider: ProviderConfig): string {
  return provider.display_name;
}

// ── Settings ───────────────────────────────────────────────────────────────

export const getSettings = (): Promise<Settings> => invoke("get_settings");

export const upsertProvider = (
  provider: ProviderConfig,
  apiKey: string | null
): Promise<void> => invoke("upsert_provider", { provider, apiKey });

export const deleteProvider = (providerId: string): Promise<void> =>
  invoke("delete_provider", { providerId });

export const setDefaultProvider = (providerId: string): Promise<void> =>
  invoke("set_default_provider", { providerId });

export const listModels = (providerId: string): Promise<ModelInfo[]> =>
  invoke("list_models", { providerId });

export const hasApiKey = (providerId: string): Promise<boolean> =>
  invoke("has_api_key", { providerId });

// ── Diagnostics ───────────────────────────────────────────────────────────

export const getSessionId = (): Promise<string> => invoke("get_session_id");

export const logClientEvent = (event: ClientEvent): Promise<void> =>
  invoke("log_client_event", { event: { ...event, context: event.context ?? {} } });

export const getRecentDiagnostics = (limit: number): Promise<DiagnosticEvent[]> =>
  invoke("get_recent_diagnostics", { limit });

export const getDiagnosticsSummary = (options: ExportOptions): Promise<string> =>
  invoke("get_diagnostics_summary", { options });

export const exportDiagnostics = (options: ExportOptions): Promise<ExportResult> =>
  invoke("export_diagnostics", { options });

export const revealDiagnosticsFolder = (): Promise<void> =>
  invoke("reveal_diagnostics_folder");

export const revealPath = (path: string): Promise<void> =>
  invoke("reveal_path", { path });

export const writeClipboardText = (text: string): Promise<void> =>
  invoke("plugin:clipboard-manager|write_text", { text });

export function installGlobalDiagnosticsHandlers() {
  window.addEventListener("error", (event) => {
    void logClientEvent({
      level: "error",
      kind: "client_error",
      message: event.message || "Unhandled frontend error",
      context: {},
    }).catch(() => {});
  });

  window.addEventListener("unhandledrejection", (event) => {
    void logClientEvent({
      level: "error",
      kind: "client_error",
      message: String(event.reason ?? "Unhandled promise rejection"),
      context: {},
    }).catch(() => {});
  });
}

export function logDiagnosticError(message: string, context: Record<string, unknown> = {}) {
  void logClientEvent({
    level: "error",
    kind: "client_error",
    message,
    context,
  }).catch(() => {});
}

// ── Chat ──────────────────────────────────────────────────────────────────

export const streamMessage = (
  providerId: string,
  modelId: string,
  messages: Message[]
): Promise<void> => invoke("stream_message", { providerId, modelId, messages });

export const saveChat = (chat: Chat): Promise<void> =>
  invoke("save_chat", { chat });

export const renameChat = (
  providerId: string,
  modelId: string,
  chat: Chat
): Promise<string> => invoke("rename_chat", { providerId, modelId, chat });

export const loadChats = (): Promise<Chat[]> => invoke("load_chats");

export const deleteChat = (chat: Chat): Promise<void> =>
  invoke("delete_chat", { chat });

// ── MCP ───────────────────────────────────────────────────────────────────

export const loadMcpServers = (): Promise<McpServer[]> =>
  invoke("load_mcp_servers");

export const saveMcpServers = (servers: McpServer[]): Promise<void> =>
  invoke("save_mcp_servers", { servers });

export const connectServer = (server: McpServer): Promise<void> =>
  invoke("connect_server", { server });

export const disconnectServer = (serverName: string): Promise<void> =>
  invoke("disconnect_server", { serverName });

export const getConnectedServers = (): Promise<string[]> =>
  invoke("get_connected_servers");

export const listTools = (
  server: McpServer
): Promise<Array<{ name: string; description: string }>> =>
  invoke("list_tools", { server });

export const executeToolCall = (
  serverName: string,
  toolName: string,
  arguments_: Record<string, unknown>
): Promise<string> =>
  invoke("execute_tool_call", {
    serverName,
    toolName,
    arguments: arguments_,
  });
