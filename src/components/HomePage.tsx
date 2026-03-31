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

export function HomePage({ onSettings }: HomePageProps) {
  const [chats, setChats] = useState<Chat[]>([{ id: 1, name: "Chat #1", messages: [] }]);
  const [activeChatId, setActiveChatId] = useState(1);
  const [nextId, setNextId] = useState(2);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [settings, setSettings] = useState<Settings | null>(null);

  const activeChat = chats.find((c) => c.id === activeChatId)!;

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then(setSettings)
      .catch(() => {});
  }, []);

  function handleNewChat() {
    const newChat: Chat = { id: nextId, name: `Chat #${nextId}`, messages: [] };
    setChats((prev) => [...prev, newChat]);
    setActiveChatId(nextId);
    setNextId((n) => n + 1);
  }

  function handleRename(id: number, name: string) {
    setChats((prev) => prev.map((c) => c.id === id ? { ...c, name } : c));
  }

  async function handleSend() {
    if (!input.trim() || loading || !settings) return;

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
    <main style={{ display: "flex", height: "100vh", fontFamily: "sans-serif" }}>
      <Sidebar
        chats={chats}
        activeChatId={activeChatId}
        onSelectChat={setActiveChatId}
        onNewChat={handleNewChat}
        onRename={handleRename}
        onSettings={onSettings}
      />
      <ChatArea
        messages={activeChat.messages}
        loading={loading}
        input={input}
        hasSettings={!!settings}
        onInputChange={setInput}
        onSend={handleSend}
      />
    </main>
  );
}
