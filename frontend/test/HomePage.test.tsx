import { listen } from "@tauri-apps/api/event";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { HomePage } from "../components/HomePage";
import {
  Chat,
  Settings,
  deleteChat,
  getSettings,
  loadChats,
  logDiagnosticError,
  renameChat,
  saveChat,
  streamMessage,
} from "../services/tauri";

vi.mock("../services/tauri", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../services/tauri")>();
  return {
    ...actual,
    getSettings: vi.fn(),
    streamMessage: vi.fn(),
    saveChat: vi.fn(),
    renameChat: vi.fn(),
    loadChats: vi.fn(),
    deleteChat: vi.fn(),
    logDiagnosticError: vi.fn(),
  };
});

vi.mock("../components/Sidebar", () => ({
  Sidebar: ({ chats, activeChatId, onSelectChat, onNewChat, onRename, onDelete, onSettings }: any) => (
    <aside>
      <div data-testid="active-chat-id">{activeChatId}</div>
      <button onClick={onNewChat}>New chat</button>
      <button onClick={onSettings}>Settings</button>
      {chats.map((chat: Chat) => (
        <div key={chat.id}>
          <button onClick={() => onSelectChat(chat.id)}>Select {chat.name}</button>
          <button onClick={() => onRename(chat.id, `${chat.name} renamed`)}>
            Rename {chat.name}
          </button>
          <button onClick={() => onDelete(chat)}>Delete {chat.name}</button>
        </div>
      ))}
    </aside>
  ),
}));

vi.mock("../components/ChatArea", () => ({
  ChatArea: (props: any) => (
    <section>
      <div data-testid="chat-name">{props.chatName ?? ""}</div>
      <div data-testid="message-count">{props.messages.length}</div>
      <div data-testid="messages">
        {props.messages.map((m: { role: string; content: string }) => `${m.role}:${m.content}`).join("|")}
      </div>
      <div data-testid="provider-id">{props.effectiveProvider?.id ?? "none"}</div>
      <div data-testid="selected-model">{props.selectedModelId ?? "none"}</div>
      <div data-testid="input-value">{props.input}</div>
      <div data-testid="has-providers">{String(props.hasProviders)}</div>
      <button onClick={() => props.onInputChange("Hello Magnus")}>Set input</button>
      <button onClick={() => props.onModelChange("claude-sonnet-4-6")}>Set model</button>
      <button onClick={() => props.onProviderChange("open_ai")}>Switch provider</button>
      <button onClick={props.onSend}>Send</button>
      <button onClick={props.onToggleTheme}>Toggle theme</button>
    </section>
  ),
}));

const settings: Settings = {
  default_provider_id: "anthropic",
  providers: [
    {
      id: "anthropic",
      display_name: "Anthropic",
      kind: "built_in",
      which: "anthropic",
    },
    {
      id: "open_ai",
      display_name: "OpenAI",
      kind: "built_in",
      which: "open_ai",
    },
  ],
};

const emptyChat: Chat = {
  id: "chat-1",
  name: "Empty Chat",
  messages: [],
  created_at: "08-05-26",
  provider_id: "anthropic",
};

const chatWithMessage: Chat = {
  id: "chat-2",
  name: "Existing Chat",
  messages: [{ role: "user", content: "Existing message" }],
  created_at: "08-05-26",
  provider_id: "anthropic",
};

function renderHomePage(settingsVersion = 0) {
  const props = {
    onSettings: vi.fn(),
    theme: "dark" as const,
    onToggleTheme: vi.fn(),
    settingsVersion,
    onActiveChatChange: vi.fn(),
  };

  const view = render(<HomePage {...props} />);
  return { ...view, props };
}

describe("HomePage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(getSettings).mockResolvedValue(settings);
    vi.mocked(loadChats).mockResolvedValue([emptyChat, chatWithMessage]);
    vi.mocked(saveChat).mockResolvedValue(undefined);
    vi.mocked(deleteChat).mockResolvedValue(undefined);
    vi.mocked(streamMessage).mockResolvedValue(undefined);
    vi.mocked(renameChat).mockResolvedValue("Renamed by model");
    vi.mocked(listen).mockResolvedValue(() => {});
    vi.stubGlobal("crypto", {
      ...crypto,
      randomUUID: vi.fn(() => "new-chat-id"),
    });
  });

  it("loads settings and chats, then selects the first chat", async () => {
    const { props } = renderHomePage();

    await waitFor(() => {
      expect(screen.getByTestId("chat-name")).toHaveTextContent("Empty Chat");
    });

    expect(getSettings).toHaveBeenCalled();
    expect(loadChats).toHaveBeenCalled();
    expect(screen.getByTestId("provider-id")).toHaveTextContent("anthropic");
    expect(screen.getByTestId("has-providers")).toHaveTextContent("true");
    expect(props.onActiveChatChange).toHaveBeenLastCalledWith("chat-1");
  });

  it("handles an empty persisted chat list", async () => {
    vi.mocked(loadChats).mockResolvedValue([]);
    const { props } = renderHomePage();

    await waitFor(() => {
      expect(screen.getByTestId("message-count")).toHaveTextContent("0");
    });

    expect(screen.getByTestId("active-chat-id")).toHaveTextContent("");
    expect(props.onActiveChatChange).toHaveBeenLastCalledWith(null);
  });

  it("creates a new chat with the default provider", async () => {
    vi.mocked(loadChats).mockResolvedValue([]);
    renderHomePage();

    await waitFor(() => {
      expect(screen.getByTestId("has-providers")).toHaveTextContent("true");
    });
    await userEvent.click(screen.getByRole("button", { name: "New chat" }));

    await waitFor(() => {
      expect(saveChat).toHaveBeenCalledWith(
        expect.objectContaining({
          id: "new-chat-id",
          name: "New Chat",
          provider_id: "anthropic",
          messages: [],
        })
      );
    });
    expect(screen.getByTestId("chat-name")).toHaveTextContent("New Chat");
  });

  it("renames and persists a chat", async () => {
    renderHomePage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Rename Empty Chat" })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: "Rename Empty Chat" }));

    await waitFor(() => {
      expect(saveChat).toHaveBeenCalledWith(
        expect.objectContaining({
          id: "chat-1",
          name: "Empty Chat renamed",
        })
      );
    });
    expect(screen.getByTestId("chat-name")).toHaveTextContent("Empty Chat renamed");
  });

  it("deletes the active chat and selects the next chat", async () => {
    renderHomePage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Delete Empty Chat" })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: "Delete Empty Chat" }));

    await waitFor(() => {
      expect(deleteChat).toHaveBeenCalledWith(emptyChat);
    });
    expect(screen.getByTestId("chat-name")).toHaveTextContent("Existing Chat");
    expect(screen.getByTestId("active-chat-id")).toHaveTextContent("chat-2");
  });

  it("changes provider only for empty chats and resets selected model", async () => {
    renderHomePage();

    await waitFor(() => {
      expect(screen.getByTestId("chat-name")).toHaveTextContent("Empty Chat");
    });
    await userEvent.click(screen.getByRole("button", { name: "Set model" }));
    expect(screen.getByTestId("selected-model")).toHaveTextContent("claude-sonnet-4-6");

    await userEvent.click(screen.getByRole("button", { name: "Switch provider" }));

    await waitFor(() => {
      expect(saveChat).toHaveBeenCalledWith(
        expect.objectContaining({
          id: "chat-1",
          provider_id: "open_ai",
        })
      );
    });
    expect(screen.getByTestId("provider-id")).toHaveTextContent("open_ai");
    expect(screen.getByTestId("selected-model")).toHaveTextContent("none");

    vi.mocked(saveChat).mockClear();
    await userEvent.click(screen.getByRole("button", { name: "Select Existing Chat" }));
    await userEvent.click(screen.getByRole("button", { name: "Switch provider" }));

    expect(saveChat).not.toHaveBeenCalled();
    expect(screen.getByTestId("provider-id")).toHaveTextContent("anthropic");
  });

  it("sends a message, appends streamed tokens, saves, and renames first-message chats", async () => {
    let streamHandler: ((event: { payload: string }) => void) | null = null;
    const unlisten = vi.fn();
    vi.mocked(listen).mockImplementation(async (_event, handler) => {
      streamHandler = handler as (event: { payload: string }) => void;
      return unlisten;
    });
    vi.mocked(streamMessage).mockImplementation(async () => {
      streamHandler?.({ payload: "Hello" });
      streamHandler?.({ payload: " there" });
    });
    renderHomePage();

    await waitFor(() => {
      expect(screen.getByTestId("chat-name")).toHaveTextContent("Empty Chat");
    });
    await userEvent.click(screen.getByRole("button", { name: "Set input" }));
    await userEvent.click(screen.getByRole("button", { name: "Set model" }));
    await userEvent.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() => {
      expect(streamMessage).toHaveBeenCalledWith(
        "anthropic",
        "claude-sonnet-4-6",
        [{ role: "user", content: "Hello Magnus" }]
      );
    });
    await waitFor(() => {
      expect(screen.getByTestId("messages")).toHaveTextContent("assistant:Hello there");
    });
    expect(renameChat).toHaveBeenCalledWith(
      "anthropic",
      "claude-sonnet-4-6",
      expect.objectContaining({
        id: "chat-1",
        messages: [{ role: "user", content: "Hello Magnus" }],
      })
    );
    await waitFor(() => {
      expect(screen.getByTestId("chat-name")).toHaveTextContent("Renamed by model");
    });
    expect(unlisten).toHaveBeenCalled();
  });

  it("logs diagnostics and shows an assistant error when send fails", async () => {
    vi.mocked(streamMessage).mockRejectedValue(new Error("provider failed"));
    renderHomePage();

    await waitFor(() => {
      expect(screen.getByTestId("chat-name")).toHaveTextContent("Empty Chat");
    });
    await userEvent.click(screen.getByRole("button", { name: "Set input" }));
    await userEvent.click(screen.getByRole("button", { name: "Set model" }));
    await userEvent.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() => {
      expect(logDiagnosticError).toHaveBeenCalledWith("Send message failed", {
        chat_id: "chat-1",
        provider_id: "anthropic",
        model_id: "claude-sonnet-4-6",
        error: "Error: provider failed",
      });
    });
    expect(screen.getByTestId("messages")).toHaveTextContent("assistant:Error: Error: provider failed");
  });

  it("logs diagnostics when settings or chats fail to load", async () => {
    vi.mocked(getSettings).mockRejectedValue(new Error("settings failed"));
    vi.mocked(loadChats).mockRejectedValue(new Error("chats failed"));
    renderHomePage();

    await waitFor(() => {
      expect(logDiagnosticError).toHaveBeenCalledWith("Failed to load settings", {
        error: "Error: settings failed",
      });
    });
    expect(logDiagnosticError).toHaveBeenCalledWith("Failed to load chats");
    expect(screen.getByTestId("active-chat-id")).toHaveTextContent("");
  });
});
