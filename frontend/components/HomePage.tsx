import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Sidebar, Chat } from "./Sidebar";
import { ChatArea } from "./ChatArea";

interface Message {
  role: "user" | "assistant";
  content: string;
}

interface Settings {
  api_key: string;
  base_url: string;
}

interface HomePageProps {
  onSettings: () => void;
}

function formatDate(): string {
  const now = new Date();
  const dd = String(now.getDate()).padStart(2, "0");
  const mm = String(now.getMonth() + 1).padStart(2, "0");
  const yy = String(now.getFullYear()).slice(2);
  return `${dd}-${mm}-${yy}`;
}

function newChat(): Chat {
  return {
    id: crypto.randomUUID(),
    name: "New Chat",
    messages: [],
    created_at: formatDate(),
  };
}

export function HomePage({ onSettings }: HomePageProps) {
  const [chats, setChats] = useState<Chat[]>([]);
  const [activeChatId, setActiveChatId] = useState<string | null>(null);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [settings, setSettings] = useState<Settings | null>(null);

  const activeChat = chats.find((c) => c.id === activeChatId) ?? null;

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then(setSettings)
      .catch(() => {});

    invoke<Chat[]>("load_chats")
      .then((loaded) => {
        if (loaded.length === 0) {
          const initial = newChat();
          setChats([initial]);
          setActiveChatId(initial.id);
          invoke("save_chat", { chat: initial }).catch(() => {});
        } else {
          setChats(loaded);
          setActiveChatId(loaded[0].id);
        }
      })
      .catch(() => {
        const initial = newChat();
        setChats([initial]);
        setActiveChatId(initial.id);
      });
  }, []);

  function handleNewChat() {
    const chat = newChat();
    setChats((prev) => [...prev, chat]);
    setActiveChatId(chat.id);
    invoke("save_chat", { chat }).catch(() => {});
  }

  function handleRename(id: string, name: string) {
    setChats((prev) => {
      const updated = prev.map((c) => c.id === id ? { ...c, name } : c);
      const chat = updated.find((c) => c.id === id);
      if (chat) invoke("save_chat", { chat }).catch(() => {});
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
    invoke("delete_chat", { chat }).catch(() => {});
  }

  async function handleSend() {
    if (!input.trim() || loading || !settings || !activeChat) return;

    const isFirstMessage = activeChat.messages.length === 0;

    const newMessages: Message[] = [
      ...activeChat.messages,
      { role: "user", content: input },
    ];

    setChats((prev) => prev.map((c) => c.id === activeChatId ? { ...c, messages: newMessages } : c));
    setInput("");
    setLoading(true);

    const assistantIndex = newMessages.length;
    setChats((prev) => prev.map((c) => c.id === activeChatId ? { ...c, messages: [...newMessages, { role: "assistant", content: "" }] } : c));

    let accumulated = "";

    const unlisten = await listen<string>("stream-token", (event) => {
      accumulated += event.payload;
      setChats((prev) => prev.map((c) => {
        if (c.id !== activeChatId) return c;
        const updated = [...c.messages];
        updated[assistantIndex] = { role: "assistant", content: accumulated };
        return { ...c, messages: updated };
      }));
    });

    try {
      await invoke("stream_message", {
        apiKey: settings.api_key,
        baseUrl: settings.base_url,
        messages: newMessages,
      });

      setChats((prev) => {
        const chat = prev.find((c) => c.id === activeChatId);
        if (chat) invoke("save_chat", { chat }).catch(() => {});
        return prev;
      });

      if (isFirstMessage) {
        invoke<string>("rename_chat", {
          apiKey: settings.api_key,
          baseUrl: settings.base_url,
          chat: { ...activeChat, messages: newMessages },
        }).then((name) => {
          setChats((prev) => prev.map((c) => c.id === activeChatId ? { ...c, name } : c));
        }).catch(() => {});
      }
    } catch (err) {
      setChats((prev) => prev.map((c) => {
        if (c.id !== activeChatId) return c;
        const updated = [...c.messages];
        updated[assistantIndex] = { role: "assistant", content: `Error: ${err}` };
        return { ...c, messages: updated };
      }));
    } finally {
      unlisten();
      setLoading(false);
    }
  }

  return (
    <main style={{ display: "flex", height: "100vh", overflow: "hidden", fontFamily: "sans-serif" }}>
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
        hasSettings={!!settings}
        onInputChange={setInput}
        onSend={handleSend}
      />
    </main>
  );
}
