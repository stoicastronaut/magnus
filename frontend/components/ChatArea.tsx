import { useEffect, useRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";
import { Mascot, Purr } from "./Mascot";
import { SendIcon, SunIcon, MoonIcon } from "./icons";

const SUGGESTIONS = [
  "Explain this error",
  "Rewrite this paragraph",
  "Plan my week",
  "Review a diff",
  "Brainstorm names",
  "Summarize a doc",
];

function EmptyState({ onSuggest }: { onSuggest: (text: string) => void }) {
  return (
    <div style={{
      flex: 1,
      display: "flex",
      flexDirection: "column",
      alignItems: "center",
      justifyContent: "center",
      gap: 32,
      padding: 40,
      position: "relative",
    }}>
      {/* Radial sunbeam glow */}
      <div style={{
        position: "absolute",
        top: "15%",
        left: "50%",
        transform: "translateX(-50%)",
        width: 680,
        height: 680,
        borderRadius: "50%",
        pointerEvents: "none",
        background: "radial-gradient(circle, color-mix(in oklch, var(--brand) 10%, transparent), transparent 55%)",
      }} />

      {/* Mascot + purr */}
      <div style={{ position: "relative", display: "flex", flexDirection: "column", alignItems: "center" }}>
        <Mascot size={160} variant="breathing" />
        <div style={{ marginTop: 14, color: "var(--brand)", opacity: 0.8, display: "flex", alignItems: "center", gap: 10 }}>
          <Purr />
          <span style={{ fontSize: 11, color: "var(--fg-3)", fontFamily: "var(--mg-mono)" }}>
            idle · 72 bpm
          </span>
        </div>
      </div>

      {/* Headline */}
      <div style={{ textAlign: "center", position: "relative", maxWidth: 520 }}>
        <h1 style={{ fontSize: 40, fontWeight: 500, letterSpacing: "-0.035em", margin: 0, color: "var(--fg)" }}>
          What shall we chase today?
        </h1>
        <p style={{ fontSize: 14, color: "var(--fg-2)", marginTop: 10, lineHeight: 1.6, margin: "10px 0 0" }}>
          Pick a prompt or type below to start.
        </p>
      </div>

      {/* Suggestion chips */}
      <div style={{ display: "flex", flexWrap: "wrap", gap: 8, justifyContent: "center", maxWidth: 620, position: "relative" }}>
        {SUGGESTIONS.map((t) => (
          <button
            key={t}
            onClick={() => onSuggest(t)}
            style={{
              padding: "8px 14px",
              borderRadius: 999,
              background: "var(--bg-2)",
              border: "1px solid var(--line)",
              color: "var(--fg-2)",
              fontSize: 12,
              cursor: "pointer",
              fontFamily: "var(--mg-sans)",
            }}
          >
            {t}
          </button>
        ))}
      </div>
    </div>
  );
}

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
  chatName?: string;
  theme: "dark" | "light";
  onToggleTheme: () => void;
}

export function ChatArea({ messages, loading, input, hasSettings, onInputChange, onSend, chatName, theme, onToggleTheme }: ChatAreaProps) {
  const bottomRef = useRef<HTMLDivElement>(null);
  const [focused, setFocused] = useState(false);
  const canSend = hasSettings && !loading && !!input.trim();

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  return (
    <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", overflow: "hidden", background: "var(--bg)", position: "relative" }}>
      {/* Header */}
      <header style={{
        height: 48,
        flexShrink: 0,
        padding: "0 20px",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        borderBottom: "1px solid var(--line)",
      }}>
        <span style={{ fontSize: 13, fontWeight: 500, color: "var(--fg)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {chatName ?? "New chat"}
        </span>
        <button
          onClick={onToggleTheme}
          title={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
          style={{
            width: 30,
            height: 30,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "transparent",
            border: "1px solid var(--line)",
            borderRadius: 8,
            color: "var(--fg-3)",
            cursor: "pointer",
            flexShrink: 0,
          }}
        >
          {theme === "dark" ? <SunIcon size={14} /> : <MoonIcon size={14} />}
        </button>
      </header>

      {/* Messages */}
      <div style={{ flex: 1, minHeight: 0, overflowY: "auto", overflowX: "hidden" }}>
        {messages.length === 0
          ? <EmptyState onSuggest={onInputChange} />
          : (
            <div style={{ maxWidth: 680, margin: "0 auto", padding: "32px 24px 180px" }}>
              <div style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
                {messages.map((msg, i) => {
                  if (msg.role === "assistant" && msg.content === "") return null;
                  return (
                    <div key={i} style={{ display: "flex", justifyContent: msg.role === "user" ? "flex-end" : "flex-start" }}>
                      <div style={{
                        maxWidth: "80%",
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
            </div>
          )
        }
      </div>

      {/* Floating composer */}
      <div style={{
        position: "absolute",
        bottom: 20,
        left: "50%",
        transform: "translateX(-50%)",
        width: "min(680px, calc(100% - 40px))",
        zIndex: 10,
      }}>
        <div style={{
          background: "color-mix(in oklch, var(--bg-2) 96%, transparent)",
          backdropFilter: "blur(12px)",
          border: `1px solid ${focused ? "color-mix(in oklch, var(--brand) 55%, var(--line))" : "var(--line)"}`,
          borderRadius: 16,
          boxShadow: focused
            ? "0 0 0 3px color-mix(in oklch, var(--brand) 14%, transparent), 0 18px 40px -12px rgba(0,0,0,0.5)"
            : "0 18px 40px -12px rgba(0,0,0,0.45)",
          transition: "border-color 0.18s, box-shadow 0.18s",
          padding: "12px 12px 10px 16px",
        }}>
          <textarea
            value={input}
            onFocus={() => setFocused(true)}
            onBlur={() => setFocused(false)}
            onChange={(e) => onInputChange(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && !e.shiftKey && (e.preventDefault(), onSend())}
            placeholder={hasSettings ? "Ask Magnus…" : "Configure API key in settings first"}
            disabled={!hasSettings || loading}
            rows={1}
            style={{
              width: "100%",
              resize: "none",
              border: "none",
              outline: "none",
              background: "transparent",
              color: "var(--fg)",
              fontFamily: "var(--mg-sans)",
              fontSize: 14,
              lineHeight: 1.5,
              padding: "4px 0 6px",
              boxSizing: "border-box",
            }}
          />
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <div style={{ flex: 1 }} />
            <span style={{ fontSize: 11, color: "var(--fg-3)", fontFamily: "var(--mg-mono)" }}>
              {input.length ? `${input.length} chars` : "idle"}
            </span>
            <button
              onClick={onSend}
              disabled={!canSend}
              style={{
                width: 32,
                height: 32,
                borderRadius: 9,
                border: "none",
                background: canSend ? "var(--brand)" : "var(--bg)",
                color: canSend ? "var(--on-brand)" : "var(--fg-3)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                cursor: canSend ? "pointer" : "default",
                transition: "background 0.15s, color 0.15s",
                flexShrink: 0,
              }}
            >
              <SendIcon size={14} />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
