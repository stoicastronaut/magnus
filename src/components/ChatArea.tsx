import { useEffect, useRef } from "react";

interface Message {
  role: "user" | "assistant";
  content: string;
}

interface ChatAreaProps {
  messages: Message[];
  loading: boolean;
  input: string;
  hasSettings: boolean;
  onInputChange: (value: string) => void;
  onSend: () => void;
}

export function ChatArea({ messages, loading, input, hasSettings, onInputChange, onSend }: ChatAreaProps) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
      {/* Messages */}
      <div style={{ flex: 1, overflowY: "auto", padding: "1rem", display: "flex", flexDirection: "column", gap: "0.75rem" }}>
        {messages.length === 0 && (
          <div style={{ textAlign: "center", color: "#999", marginTop: "30%" }}>
            Start a conversation
          </div>
        )}
        {messages.map((msg, i) => {
          if (msg.role === "assistant" && msg.content === "") return null;
          return (
            <div key={i} style={{ display: "flex", justifyContent: msg.role === "user" ? "flex-end" : "flex-start" }}>
              <div style={{
                maxWidth: "70%",
                padding: "0.6rem 0.9rem",
                borderRadius: 12,
                background: msg.role === "user" ? "#0070f3" : "#f0f0f0",
                color: msg.role === "user" ? "white" : "black",
                whiteSpace: "pre-wrap",
              }}>
                {msg.content}
              </div>
            </div>
          );
        })}
        {loading && (
          <div style={{ display: "flex", justifyContent: "flex-start" }}>
            <div style={{ padding: "0.6rem 0.9rem", borderRadius: 12, background: "#f0f0f0", color: "#999" }}>
              ...
            </div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      {/* Input */}
      <div style={{ padding: "0.75rem 1rem", borderTop: "1px solid #eee", display: "flex", gap: "0.5rem" }}>
        <input
          value={input}
          onChange={(e) => onInputChange(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && !e.shiftKey && onSend()}
          placeholder={hasSettings ? "Message Claude..." : "Configure API key in settings first"}
          disabled={!hasSettings || loading}
          style={{ flex: 1, padding: "0.5rem 0.75rem", borderRadius: 8, border: "1px solid #ddd", fontSize: 14 }}
        />
        <button
          onClick={onSend}
          disabled={!hasSettings || loading || !input.trim()}
          style={{ padding: "0.5rem 1rem", borderRadius: 8, background: "#0070f3", color: "white", border: "none", cursor: "pointer" }}
        >
          Send
        </button>
      </div>
    </div>
  );
}
