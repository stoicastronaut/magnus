import { useState, useEffect, useRef } from "react";
import { PawIcon, GearIcon, PlusIcon } from "./icons";
import { PawBullet } from "./Mascot";

interface Message {
  role: "user" | "assistant";
  content: string;
}

export interface Chat {
  id: string;
  name: string;
  messages: Message[];
  created_at: string;
}

interface ContextMenu {
  chatId: string;
  x: number;
  y: number;
}

interface SidebarProps {
  chats: Chat[];
  activeChatId: string;
  onSelectChat: (id: string) => void;
  onNewChat: () => void;
  onRename: (id: string, name: string) => void;
  onDelete: (chat: Chat) => void;
  onSettings: () => void;
}

export function Sidebar({ chats, activeChatId, onSelectChat, onNewChat, onRename, onDelete, onSettings }: SidebarProps) {
  const [renamingChatId, setRenamingChatId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const [contextMenu, setContextMenu] = useState<ContextMenu | null>(null);
  const contextMenuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (contextMenuRef.current && !contextMenuRef.current.contains(e.target as Node)) {
        setContextMenu(null);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  function handleRightClick(e: React.MouseEvent, chat: Chat) {
    e.preventDefault();
    setContextMenu({ chatId: chat.id, x: e.clientX, y: e.clientY });
  }

  function handleRenameClick(chat: Chat) {
    setContextMenu(null);
    setRenamingChatId(chat.id);
    setRenameValue(chat.name);
  }

  function handleDeleteClick(chat: Chat) {
    setContextMenu(null);
    onDelete(chat);
  }

  function commitRename(id: string) {
    if (renameValue.trim()) {
      onRename(id, renameValue.trim());
    }
    setRenamingChatId(null);
  }

  const contextMenuChat = contextMenu ? chats.find((c) => c.id === contextMenu.chatId) : null;

  return (
    <div style={{
      width: 240,
      flexShrink: 0,
      display: "flex",
      flexDirection: "column",
      overflow: "hidden",
      borderRight: "1px solid var(--line)",
      background: "var(--bg-panel)",
    }}>
      {/* Header */}
      <div style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "0.75rem 1rem",
        borderBottom: "1px solid var(--line)",
      }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <div style={{
            width: 26,
            height: 26,
            borderRadius: 7,
            background: "var(--brand)",
            color: "var(--on-brand)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}>
            <PawIcon size={14} />
          </div>
          <span style={{ fontWeight: 600, fontSize: 15, color: "var(--fg)" }}>Magnus</span>
        </div>
        <button
          onClick={onNewChat}
          title="New chat"
          style={{
            background: "none",
            border: "1px solid var(--line)",
            borderRadius: "var(--mg-r-xs)",
            cursor: "pointer",
            padding: "4px 8px",
            color: "var(--fg-2)",
            display: "flex",
            alignItems: "center",
          }}
        >
          <PlusIcon size={14} />
        </button>
      </div>

      {/* Chat list */}
      <div style={{ flex: 1, overflowY: "auto", padding: "0.5rem" }}>
        {chats.map((chat) => {
          const active = chat.id === activeChatId;
          return (
            <div
              key={chat.id}
              onClick={() => { if (renamingChatId !== chat.id) onSelectChat(chat.id); }}
              onContextMenu={(e) => handleRightClick(e, chat)}
              style={{
                padding: "0.5rem 0.75rem",
                borderRadius: "var(--mg-r-sm)",
                cursor: "pointer",
                background: active ? "color-mix(in oklch, var(--brand) 15%, var(--bg))" : "transparent",
                color: active ? "var(--brand)" : "var(--fg-2)",
                fontWeight: active ? 600 : 400,
                fontSize: 13,
                marginBottom: 2,
                userSelect: "none",
                display: "flex",
                alignItems: "center",
                gap: 8,
              }}
            >
              <PawBullet style={{ color: active ? "var(--brand)" : "var(--fg-3)", opacity: active ? 1 : 0.6, flexShrink: 0 }} />
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
                  style={{
                    flex: 1,
                    padding: "2px 4px",
                    fontSize: 13,
                    borderRadius: 4,
                    border: "1px solid var(--brand)",
                    background: "var(--bg)",
                    color: "var(--fg)",
                    outline: "none",
                  }}
                />
              ) : (
                <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", display: "block" }}>
                  {chat.name}
                </span>
              )}
            </div>
          );
        })}
      </div>

      {/* Settings at bottom */}
      <div style={{ padding: "0.75rem 1rem", borderTop: "1px solid var(--line)" }}>
        <button
          onClick={onSettings}
          title="Settings"
          style={{
            background: "none",
            border: "none",
            cursor: "pointer",
            padding: 4,
            color: "var(--fg-3)",
            display: "flex",
            alignItems: "center",
            gap: "0.5rem",
            fontSize: 13,
          }}
        >
          <GearIcon size={16} /> Settings
        </button>
      </div>

      {/* Context menu */}
      {contextMenu && contextMenuChat && (
        <div
          ref={contextMenuRef}
          style={{
            position: "fixed",
            top: contextMenu.y,
            left: contextMenu.x,
            background: "var(--bg-panel)",
            border: "1px solid var(--line)",
            borderRadius: "var(--mg-r-sm)",
            boxShadow: "0 4px 16px rgba(0,0,0,0.24)",
            zIndex: 1000,
            overflow: "hidden",
            minWidth: 120,
          }}
        >
          <button
            onClick={() => handleRenameClick(contextMenuChat)}
            style={{
              display: "block",
              width: "100%",
              padding: "0.5rem 1rem",
              background: "none",
              border: "none",
              textAlign: "left",
              cursor: "pointer",
              fontSize: 13,
              color: "var(--fg)",
            }}
          >
            Rename
          </button>
          <button
            onClick={() => handleDeleteClick(contextMenuChat)}
            style={{
              display: "block",
              width: "100%",
              padding: "0.5rem 1rem",
              background: "none",
              border: "none",
              textAlign: "left",
              cursor: "pointer",
              fontSize: 13,
              color: "#e53e3e",
            }}
          >
            Delete
          </button>
        </div>
      )}
    </div>
  );
}
