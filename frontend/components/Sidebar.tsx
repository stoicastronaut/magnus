import { useState, useEffect, useRef } from "react";
import { PawIcon, GearIcon, PlusIcon, SearchIcon } from "./icons";
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

function CollapseIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round">
      <path d="M10 4L6 8l4 4" /><path d="M3 3v10" />
    </svg>
  );
}

function ExpandIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round">
      <path d="M6 4l4 4-4 4" /><path d="M13 3v10" />
    </svg>
  );
}

function railBtn(): React.CSSProperties {
  return {
    width: 34, height: 34, borderRadius: 9,
    background: "transparent", border: "none", cursor: "pointer",
    color: "var(--fg-2)",
    display: "flex", alignItems: "center", justifyContent: "center",
    flexShrink: 0,
  };
}

export function Sidebar({ chats, activeChatId, onSelectChat, onNewChat, onRename, onDelete, onSettings }: SidebarProps) {
  const [collapsed, setCollapsed] = useState(false);
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
    if (renameValue.trim()) onRename(id, renameValue.trim());
    setRenamingChatId(null);
  }

  const contextMenuChat = contextMenu ? chats.find((c) => c.id === contextMenu.chatId) : null;

  if (collapsed) {
    return (
      <>
        <nav style={{
          width: 56,
          flexShrink: 0,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          padding: "14px 0",
          gap: 4,
          background: "var(--bg-panel)",
          borderRight: "1px solid var(--line)",
          transition: "width 0.22s cubic-bezier(.2,.7,.3,1)",
        }}>
          {/* Brand mark */}
          <div style={{
            width: 32, height: 32, borderRadius: 9,
            background: "var(--brand)", color: "var(--on-brand)",
            display: "flex", alignItems: "center", justifyContent: "center",
            marginBottom: 6,
          }}>
            <PawIcon size={18} />
          </div>

          {/* Expand chevron */}
          <button
            onClick={() => setCollapsed(false)}
            title="Expand sidebar"
            style={{ ...railBtn(), width: 28, height: 24, marginBottom: 6, color: "var(--fg-3)" }}
          >
            <ExpandIcon />
          </button>

          <button onClick={onNewChat} title="New chat" style={railBtn()}>
            <PlusIcon size={14} />
          </button>
          <button title="Search" style={{ ...railBtn(), color: "var(--fg-3)" }}>
            <SearchIcon size={14} />
          </button>

          <div style={{ height: 1, width: 24, background: "var(--line)", margin: "8px 0" }} />

          {/* Paw-dot chat list */}
          <div style={{ display: "flex", flexDirection: "column", gap: 4, alignItems: "center", flex: 1, overflowY: "auto" }}>
            {chats.map((chat) => {
              const active = chat.id === activeChatId;
              return (
                <button
                  key={chat.id}
                  onClick={() => onSelectChat(chat.id)}
                  onContextMenu={(e) => handleRightClick(e, chat)}
                  title={chat.name}
                  style={{
                    ...railBtn(),
                    background: active ? "color-mix(in oklch, var(--brand) 18%, transparent)" : "transparent",
                    color: active ? "var(--brand)" : "var(--fg-3)",
                    position: "relative",
                  }}
                >
                  <PawBullet style={{ color: "currentColor" }} />
                </button>
              );
            })}
          </div>

          <button onClick={onSettings} title="Settings" style={railBtn()}>
            <GearIcon size={14} />
          </button>
        </nav>

        {contextMenu && contextMenuChat && (
          <ContextMenuOverlay
            ref={contextMenuRef}
            x={contextMenu.x}
            y={contextMenu.y}
            onRename={() => handleRenameClick(contextMenuChat)}
            onDelete={() => handleDeleteClick(contextMenuChat)}
          />
        )}
      </>
    );
  }

  return (
    <>
      <nav style={{
        width: 240,
        flexShrink: 0,
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
        background: "var(--bg-panel)",
        borderRight: "1px solid var(--line)",
        transition: "width 0.22s cubic-bezier(.2,.7,.3,1)",
      }}>
        {/* Header */}
        <div style={{ padding: "14px 14px 10px", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            <div style={{
              width: 28, height: 28, borderRadius: 8,
              background: "var(--brand)", color: "var(--on-brand)",
              display: "flex", alignItems: "center", justifyContent: "center",
            }}>
              <PawIcon size={15} />
            </div>
            <div style={{ display: "flex", flexDirection: "column", lineHeight: 1.1 }}>
              <span style={{ fontWeight: 600, fontSize: 14, letterSpacing: "-0.01em", color: "var(--fg)" }}>Magnus</span>
              <span style={{ fontSize: 10, color: "var(--fg-3)", fontFamily: "var(--mg-mono)" }}>purring · v0.3</span>
            </div>
          </div>
          <button
            onClick={() => setCollapsed(true)}
            title="Collapse sidebar"
            style={{
              width: 24, height: 24, borderRadius: 6,
              border: "1px solid var(--line-soft)",
              background: "transparent", color: "var(--fg-3)", cursor: "pointer",
              display: "flex", alignItems: "center", justifyContent: "center", padding: 0,
            }}
          >
            <CollapseIcon />
          </button>
        </div>

        {/* New chat */}
        <div style={{ padding: "2px 12px 8px" }}>
          <button
            onClick={onNewChat}
            style={{
              width: "100%", height: 34, borderRadius: 9,
              border: "1px solid var(--line)",
              background: "var(--bg-2)", color: "var(--fg)",
              fontFamily: "var(--mg-sans)",
              fontSize: 13, fontWeight: 500,
              display: "flex", alignItems: "center", justifyContent: "space-between",
              padding: "0 10px", cursor: "pointer",
            }}
          >
            <span style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <PlusIcon size={12} /> New chat
            </span>
          </button>
        </div>

        {/* Search (visual — ⌘K not yet wired) */}
        <div style={{ padding: "0 12px 10px" }}>
          <div style={{
            height: 30, borderRadius: 8,
            background: "var(--bg-2)", border: "1px solid var(--line-soft)",
            display: "flex", alignItems: "center", gap: 8, padding: "0 10px",
            color: "var(--fg-3)",
          }}>
            <SearchIcon size={12} />
            <span style={{ fontSize: 12, flex: 1, textAlign: "left" }}>Search</span>
          </div>
        </div>

        {/* Chat list */}
        <div style={{ flex: 1, overflowY: "auto", padding: "2px 8px" }}>
          {chats.map((chat) => {
            const active = chat.id === activeChatId;
            return (
              <div
                key={chat.id}
                onClick={() => { if (renamingChatId !== chat.id) onSelectChat(chat.id); }}
                onContextMenu={(e) => handleRightClick(e, chat)}
                style={{
                  padding: "7px 10px", borderRadius: 8,
                  background: active ? "color-mix(in oklch, var(--brand) 14%, transparent)" : "transparent",
                  display: "flex", alignItems: "center", gap: 10,
                  cursor: "pointer", marginBottom: 1,
                  color: active ? "var(--brand)" : "var(--fg)",
                  userSelect: "none",
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
                      flex: 1, padding: "2px 4px", fontSize: 13,
                      borderRadius: 4, border: "1px solid var(--brand)",
                      background: "var(--bg)", color: "var(--fg)", outline: "none",
                    }}
                  />
                ) : (
                  <span style={{
                    fontSize: 13, fontWeight: active ? 500 : 400,
                    flex: 1, minWidth: 0, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
                  }}>
                    {chat.name}
                  </span>
                )}
              </div>
            );
          })}
        </div>

        {/* Footer */}
        <div style={{ padding: "10px 12px", borderTop: "1px solid var(--line)" }}>
          <button
            onClick={onSettings}
            title="Settings"
            style={{
              background: "none", border: "none", color: "var(--fg-2)",
              display: "flex", alignItems: "center", gap: 8,
              fontSize: 13, cursor: "pointer", padding: 4,
            }}
          >
            <GearIcon size={14} /> Settings
          </button>
        </div>
      </nav>

      {contextMenu && contextMenuChat && (
        <ContextMenuOverlay
          ref={contextMenuRef}
          x={contextMenu.x}
          y={contextMenu.y}
          onRename={() => handleRenameClick(contextMenuChat)}
          onDelete={() => handleDeleteClick(contextMenuChat)}
        />
      )}
    </>
  );
}

const ContextMenuOverlay = ({ x, y, onRename, onDelete, ref }: {
  x: number; y: number;
  onRename: () => void; onDelete: () => void;
  ref: React.RefObject<HTMLDivElement | null>;
}) => (
  <div
    ref={ref}
    style={{
      position: "fixed", top: y, left: x,
      background: "var(--bg-panel)", border: "1px solid var(--line)",
      borderRadius: "var(--mg-r-sm)",
      boxShadow: "0 4px 16px rgba(0,0,0,0.24)",
      zIndex: 1000, overflow: "hidden", minWidth: 120,
    }}
  >
    <button
      onClick={onRename}
      style={{
        display: "block", width: "100%", padding: "0.5rem 1rem",
        background: "none", border: "none", textAlign: "left",
        cursor: "pointer", fontSize: 13, color: "var(--fg)",
      }}
    >
      Rename
    </button>
    <button
      onClick={onDelete}
      style={{
        display: "block", width: "100%", padding: "0.5rem 1rem",
        background: "none", border: "none", textAlign: "left",
        cursor: "pointer", fontSize: 13, color: "#e53e3e",
      }}
    >
      Delete
    </button>
  </div>
);
