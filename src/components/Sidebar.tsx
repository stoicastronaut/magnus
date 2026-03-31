import { useState } from "react";

interface Message {
  role: "user" | "assistant";
  content: string;
}

export interface Chat {
  id: number;
  name: string;
  messages: Message[];
}

interface SidebarProps {
  chats: Chat[];
  activeChatId: number;
  onSelectChat: (id: number) => void;
  onNewChat: () => void;
  onRename: (id: number, name: string) => void;
  onSettings: () => void;
}

const GearIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
  </svg>
);

export function Sidebar({ chats, activeChatId, onSelectChat, onNewChat, onRename, onSettings }: SidebarProps) {
  const [renamingChatId, setRenamingChatId] = useState<number | null>(null);
  const [renameValue, setRenameValue] = useState("");

  function handleRightClick(e: React.MouseEvent, chat: Chat) {
    e.preventDefault();
    setRenamingChatId(chat.id);
    setRenameValue(chat.name);
  }

  function commitRename(id: number) {
    if (renameValue.trim()) {
      onRename(id, renameValue.trim());
    }
    setRenamingChatId(null);
  }

  return (
    <div style={{ width: 240, display: "flex", flexDirection: "column", borderRight: "1px solid #eee", background: "#fafafa" }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "0.75rem 1rem", borderBottom: "1px solid #eee" }}>
        <span style={{ fontWeight: 700, fontSize: 15 }}>Magnus</span>
        <button
          onClick={onNewChat}
          title="New chat"
          style={{ background: "none", border: "1px solid #ddd", borderRadius: 6, cursor: "pointer", padding: "2px 8px", fontSize: 16, lineHeight: 1 }}
        >
          +
        </button>
      </div>

      {/* Chat list */}
      <div style={{ flex: 1, overflowY: "auto", padding: "0.5rem" }}>
        {chats.map((chat) => (
          <div
            key={chat.id}
            onClick={() => { if (renamingChatId !== chat.id) onSelectChat(chat.id); }}
            onContextMenu={(e) => handleRightClick(e, chat)}
            style={{
              padding: "0.5rem 0.75rem",
              borderRadius: 8,
              cursor: "pointer",
              background: chat.id === activeChatId ? "#e8f0fe" : "transparent",
              color: chat.id === activeChatId ? "#0070f3" : "#333",
              fontWeight: chat.id === activeChatId ? 600 : 400,
              fontSize: 14,
              marginBottom: 2,
              userSelect: "none",
            }}
          >
            {renamingChatId === chat.id ? (
              <input
                autoFocus
                value={renameValue}
                onChange={(e) => setRenameValue(e.target.value)}
                onBlur={() => commitRename(chat.id)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") commitRename(chat.id);
                  if (e.key === "Escape") setRenamingChatId(null);
                }}
                onClick={(e) => e.stopPropagation()}
                style={{ width: "100%", padding: "2px 4px", fontSize: 14, borderRadius: 4, border: "1px solid #0070f3", outline: "none" }}
              />
            ) : (
              <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", display: "block" }}>
                {chat.name}
              </span>
            )}
          </div>
        ))}
      </div>

      {/* Settings at bottom */}
      <div style={{ padding: "0.75rem 1rem", borderTop: "1px solid #eee" }}>
        <button
          onClick={onSettings}
          title="Settings"
          style={{ background: "none", border: "none", cursor: "pointer", padding: 4, color: "#666", display: "flex", alignItems: "center", gap: "0.5rem", fontSize: 13 }}
        >
          <GearIcon /> Settings
        </button>
      </div>
    </div>
  );
}
