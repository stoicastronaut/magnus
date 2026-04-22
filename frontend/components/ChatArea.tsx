import { useEffect, useRef } from "react";
import ReactMarkdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";
import { Purr } from "./Mascot";
import { SendIcon } from "./icons";

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
    <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", overflow: "hidden", background: "var(--bg)" }}>
      {/* Messages */}
      <div style={{ flex: 1, minHeight: 0, overflowY: "auto", overflowX: "hidden", padding: "1rem", display: "flex", flexDirection: "column", gap: "0.75rem" }}>
        {messages.length === 0 && (
          <div style={{ textAlign: "center", color: "var(--fg-3)", marginTop: "30%", fontSize: 14 }}>
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
                borderRadius: "var(--mg-r-md)",
                background: msg.role === "user" ? "var(--brand)" : "var(--bg-2)",
                color: msg.role === "user" ? "var(--on-brand)" : "var(--fg)",
                whiteSpace: msg.role === "user" ? "pre-wrap" : undefined,
                fontSize: 14,
                lineHeight: 1.6,
              }}>
                {msg.role === "user" ? msg.content : (
                  <ReactMarkdown
                    components={{
                      code({ className, children, ...props }) {
                        const match = /language-(\w+)/.exec(className || "");
                        const isBlock = !!match;
                        return isBlock ? (
                          <SyntaxHighlighter style={oneDark} language={match[1]} PreTag="div">
                            {String(children).replace(/\n$/, "")}
                          </SyntaxHighlighter>
                        ) : (
                          <code style={{
                            background: "color-mix(in oklch, var(--fg-3) 20%, var(--bg-2))",
                            borderRadius: 4,
                            padding: "0.1em 0.35em",
                            fontSize: "0.88em",
                            fontFamily: "var(--mg-mono)",
                          }} {...props}>
                            {children}
                          </code>
                        );
                      },
                      p({ children }) { return <p style={{ margin: "0.25em 0" }}>{children}</p>; },
                      ul({ children }) { return <ul style={{ margin: "0.25em 0", paddingLeft: "1.25em" }}>{children}</ul>; },
                      ol({ children }) { return <ol style={{ margin: "0.25em 0", paddingLeft: "1.25em" }}>{children}</ol>; },
                      pre({ children }) { return <pre style={{ margin: "0.5em 0", borderRadius: "var(--mg-r-sm)", overflow: "auto" }}>{children}</pre>; },
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
            <div style={{
              padding: "0.6rem 0.9rem",
              borderRadius: "var(--mg-r-md)",
              background: "var(--bg-2)",
              color: "var(--brand)",
              display: "flex",
              alignItems: "center",
            }}>
              <Purr />
            </div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      {/* Input */}
      <div style={{
        padding: "0.75rem 1rem",
        borderTop: "1px solid var(--line)",
        display: "flex",
        gap: "0.5rem",
        background: "var(--bg)",
      }}>
        <input
          value={input}
          onChange={(e) => onInputChange(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && !e.shiftKey && onSend()}
          placeholder={hasSettings ? "Message Claude..." : "Configure API key in settings first"}
          disabled={!hasSettings || loading}
          style={{
            flex: 1,
            padding: "0.5rem 0.75rem",
            borderRadius: "var(--mg-r-sm)",
            border: "1px solid var(--line)",
            background: "var(--bg-2)",
            color: "var(--fg)",
            fontSize: 14,
          }}
        />
        <button
          onClick={onSend}
          disabled={!hasSettings || loading || !input.trim()}
          style={{
            padding: "0.5rem 1rem",
            borderRadius: "var(--mg-r-sm)",
            background: "var(--brand)",
            color: "var(--on-brand)",
            border: "none",
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            gap: 6,
            fontSize: 14,
            fontWeight: 500,
            opacity: (!hasSettings || loading || !input.trim()) ? 0.5 : 1,
          }}
        >
          <SendIcon size={15} /> Send
        </button>
      </div>
    </div>
  );
}
