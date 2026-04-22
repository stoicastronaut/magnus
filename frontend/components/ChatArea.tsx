import { useEffect, useRef } from "react";
import ReactMarkdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneLight } from "react-syntax-highlighter/dist/esm/styles/prism";

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
    <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", overflow: "hidden" }}>
      {/* Messages */}
      <div style={{ flex: 1, minHeight: 0, overflowY: "auto", overflowX: "hidden", padding: "1rem", display: "flex", flexDirection: "column", gap: "0.75rem" }}>
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
                whiteSpace: msg.role === "user" ? "pre-wrap" : undefined,
              }}>
                {msg.role === "user" ? msg.content : (
                  <ReactMarkdown
                    components={{
                      code({ className, children, ...props }) {
                        const match = /language-(\w+)/.exec(className || "");
                        const isBlock = !!match;
                        return isBlock ? (
                          <SyntaxHighlighter style={oneLight} language={match[1]} PreTag="div">
                            {String(children).replace(/\n$/, "")}
                          </SyntaxHighlighter>
                        ) : (
                          <code style={{ background: "#e0e0e0", borderRadius: 4, padding: "0.1em 0.3em", fontSize: "0.9em" }} {...props}>
                            {children}
                          </code>
                        );
                      },
                      p({ children }) { return <p style={{ margin: "0.25em 0" }}>{children}</p>; },
                      ul({ children }) { return <ul style={{ margin: "0.25em 0", paddingLeft: "1.25em" }}>{children}</ul>; },
                      ol({ children }) { return <ol style={{ margin: "0.25em 0", paddingLeft: "1.25em" }}>{children}</ol>; },
                      pre({ children }) { return <pre style={{ margin: "0.5em 0", borderRadius: 8, overflow: "auto" }}>{children}</pre>; },
                    }}
                  >
                    {msg.content}
                  </ReactMarkdown>
                )}
              </div>
            </div>
          );
        })}
        {loading && (
          <div style={{ display: "flex", justifyContent: "flex-start" }}>
            <div style={{ padding: "0.6rem 0.9rem", borderRadius: 12, background: "#f0f0f0", display: "flex", gap: 4, alignItems: "center" }}>
              {[0, 1, 2].map((i) => (
                <span
                  key={i}
                  style={{
                    width: 6,
                    height: 6,
                    borderRadius: "50%",
                    background: "#999",
                    display: "inline-block",
                    animation: "bounce 1.2s infinite",
                    animationDelay: `${i * 0.2}s`,
                  }}
                />
              ))}
              <style>{`
                @keyframes bounce {
                  0%, 60%, 100% { transform: translateY(0); }
                  30% { transform: translateY(-6px); }
                }
              `}</style>
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
