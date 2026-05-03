import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { Sidebar } from "./Sidebar";
import { ChatArea } from "./ChatArea";
import {
  Chat,
  Message,
  Settings,
  getSettings,
  streamMessage,
  saveChat,
  renameChat,
  loadChats,
  deleteChat,
  logDiagnosticError,
} from "../services/tauri";

interface HomePageProps {
  onSettings: () => void;
  theme: "dark" | "light";
  onToggleTheme: () => void;
  settingsVersion: number;
  onActiveChatChange: (chatId: string | null) => void;
}

function formatDate(): string {
  const now = new Date();
  const dd = String(now.getDate()).padStart(2, "0");
  const mm = String(now.getMonth() + 1).padStart(2, "0");
  const yy = String(now.getFullYear()).slice(2);
  return `${dd}-${mm}-${yy}`;
}

export function HomePage({ onSettings, theme, onToggleTheme, settingsVersion, onActiveChatChange }: HomePageProps) {
  const [chats, setChats] = useState<Chat[]>([]);
  const [activeChatId, setActiveChatId] = useState<string | null>(null);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [selectedModelId, setSelectedModelId] = useState<string | null>(null);

  const activeChat = chats.find((c) => c.id === activeChatId) ?? null;

  // Derive the effective provider for the active chat
  const effectiveProviderId = activeChat?.provider_id ||
    settings?.default_provider_id || null;

  const effectiveProvider = settings?.providers.find((p) => p.id === effectiveProviderId) ?? null;

  useEffect(() => {
    onActiveChatChange(activeChatId);
  }, [activeChatId, onActiveChatChange]);

  // Reload settings on mount and whenever settingsVersion bumps (e.g., after returning from SettingsPage)
  useEffect(() => {
    getSettings()
      .then((s) => setSettings(s))
      .catch((err) => {
        logDiagnosticError("Failed to load settings", { error: String(err) });
      });
  }, [settingsVersion]);

  useEffect(() => {
    loadChats()
      .then((loaded) => {
        if (loaded.length === 0) {
          setChats([]);
          setActiveChatId(null);
        } else {
          setChats(loaded);
          setActiveChatId(loaded[0].id);
        }
      })
      .catch(() => {
        logDiagnosticError("Failed to load chats");
        setChats([]);
        setActiveChatId(null);
      });
  }, []);

  function makeNewChat(providerId: string): Chat {
    return {
      id: crypto.randomUUID(),
      name: "New Chat",
      messages: [],
      created_at: formatDate(),
      provider_id: providerId,
    };
  }

  function handleNewChat() {
    const pid = settings?.default_provider_id ?? settings?.providers[0]?.id ?? "";
    const chat = makeNewChat(pid);
    setChats((prev) => [...prev, chat]);
    setActiveChatId(chat.id);
    saveChat(chat).catch(() => {});
  }

  function handleRename(id: string, name: string) {
    setChats((prev) => {
      const updated = prev.map((c) => c.id === id ? { ...c, name } : c);
      const chat = updated.find((c) => c.id === id);
      if (chat) saveChat(chat).catch(() => {});
      return updated;
    });
  }

  function handleDelete(chat: Chat) {
    setChats((prev) => {
      const remaining = prev.filter((c) => c.id !== chat.id);
      if (activeChatId === chat.id) {
        setActiveChatId(remaining.length > 0 ? remaining[0].id : null);
      }
      return remaining;
    });
    deleteChat(chat).catch(() => {});
  }

  function handleProviderChange(providerId: string) {
    if (!activeChat || activeChat.messages.length > 0) return;
    setChats((prev) => {
      const updated = prev.map((c) => c.id === activeChat.id ? { ...c, provider_id: providerId } : c);
      const chat = updated.find((c) => c.id === activeChat.id);
      if (chat) saveChat(chat).catch(() => {});
      return updated;
    });
    setSelectedModelId(null); // reset model when provider changes
  }

  async function handleSend() {
    if (!input.trim() || loading || !activeChat) {
      return;
    }

    const pid = activeChat.provider_id || effectiveProviderId;
    const mid = selectedModelId;
    if (!pid || !mid) {
      return;
    }

    const isFirstMessage = activeChat.messages.length === 0;

    const newMessages: Message[] = [
      ...activeChat.messages,
      { role: "user", content: input },
    ];

    setChats((prev) => prev.map((c) => c.id === activeChatId ? { ...c, messages: newMessages } : c));
    setInput("");
    setLoading(true);

    const assistantIndex = newMessages.length;
    setChats((prev) => prev.map((c) =>
      c.id === activeChatId
        ? { ...c, messages: [...newMessages, { role: "assistant", content: "", model_id: mid }] }
        : c
    ));

    let accumulated = "";

    const unlisten = await listen<string>("stream-token", (event) => {
      accumulated += event.payload;
      setChats((prev) => prev.map((c) => {
        if (c.id !== activeChatId) return c;
        const updated = [...c.messages];
        updated[assistantIndex] = { role: "assistant", content: accumulated, model_id: mid };
        return { ...c, messages: updated };
      }));
    });

    try {
      await streamMessage(pid, mid, newMessages);

      setChats((prev) => {
        const chat = prev.find((c) => c.id === activeChatId);
        if (chat) saveChat(chat).catch(() => {});
        return prev;
      });

      if (isFirstMessage) {
        renameChat(pid, mid, { ...activeChat, messages: newMessages })
          .then((name) => {
            setChats((prev) => prev.map((c) => c.id === activeChatId ? { ...c, name } : c));
          })
          .catch(() => {});
      }
    } catch (err) {
      logDiagnosticError("Send message failed", {
        chat_id: activeChat.id,
        provider_id: pid,
        model_id: mid,
        error: String(err),
      });
      setChats((prev) => prev.map((c) => {
        if (c.id !== activeChatId) return c;
        const updated = [...c.messages];
        updated[assistantIndex] = { role: "assistant", content: `Error: ${err}`, model_id: mid };
        return { ...c, messages: updated };
      }));
    } finally {
      unlisten();
      setLoading(false);
    }
  }

  const hasProviders = (settings?.providers.length ?? 0) > 0;

  return (
    <main style={{ display: "flex", height: "100vh", overflow: "hidden", background: "var(--bg)" }}>
      <Sidebar
        chats={chats}
        activeChatId={activeChatId ?? ""}
        onSelectChat={setActiveChatId}
        onNewChat={handleNewChat}
        onRename={handleRename}
        onDelete={handleDelete}
        onSettings={onSettings}
      />
      <ChatArea
        messages={activeChat?.messages ?? []}
        loading={loading}
        input={input}
        hasProviders={hasProviders}
        settings={settings}
        activeChat={activeChat}
        effectiveProvider={effectiveProvider}
        selectedModelId={selectedModelId}
        onModelChange={setSelectedModelId}
        onProviderChange={handleProviderChange}
        onInputChange={setInput}
        onSend={handleSend}
        chatName={activeChat?.name}
        theme={theme}
        onToggleTheme={onToggleTheme}
      />
    </main>
  );
}
