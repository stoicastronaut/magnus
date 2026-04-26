import { useEffect, useRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";
import { Mascot, Purr } from "./Mascot";
import { SendIcon, SunIcon, MoonIcon } from "./icons";
import ModelPicker from "./ModelPicker";
import ProviderPicker from "./ProviderPicker";
import { Chat, Message, ProviderConfig, Settings, providerDot } from "../services/tauri";

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
      flex: 1, display: "flex", flexDirection: "column",
      alignItems: "center", justifyContent: "center",
      gap: 32, padding: 40, position: "relative",
    }}>
      <div style={{
        position: "absolute", top: "15%", left: "50%", transform: "translateX(-50%)",
        width: 680, height: 680, borderRadius: "50%", pointerEvents: "none",
        background: "radial-gradient(circle, color-mix(in oklch, var(--brand) 10%, transparent), transparent 55%)",
      }} />
      <div style={{ position: "relative", display: "flex", flexDirection: "column", alignItems: "center" }}>
        <Mascot size={160} variant="breathing" />
        <div style={{ marginTop: 14, color: "var(--brand)", opacity: 0.8, display: "flex", alignItems: "center", gap: 10 }}>
          <Purr />
          <span style={{ fontSize: 11, color: "var(--fg-3)", fontFamily: "var(--mg-mono)" }}>idle · 72 bpm</span>
        </div>
      </div>
      <div style={{ textAlign: "center", position: "relative", maxWidth: 520 }}>
        <h1 style={{ fontSize: 40, fontWeight: 500, letterSpacing: "-0.035em", margin: 0, color: "var(--fg)" }}>
          What shall we chase today?
        </h1>
        <p style={{ fontSize: 14, color: "var(--fg-2)", margin: "10px 0 0", lineHeight: 1.6 }}>
          Pick a prompt or type below to start.
        </p>
      </div>
      <div style={{ display: "flex", flexWrap: "wrap", gap: 8, justifyContent: "center", maxWidth: 620, position: "relative" }}>
        {SUGGESTIONS.map((t) => (
          <button key={t} onClick={() => onSuggest(t)} style={{
            padding: "8px 14px", borderRadius: 999,
            background: "var(--bg-2)", border: "1px solid var(--line)",
            color: "var(--fg-2)", fontSize: 12, cursor: "pointer", fontFamily: "var(--mg-sans)",
          }}>{t}</button>
        ))}
      </div>
    </div>
  );
}

function NoProviderState({ onSettings }: { onSettings?: () => void }) {
  return (
    <div style={{
      flex: 1, display: "flex", flexDirection: "column",
      alignItems: "center", justifyContent: "center", gap: 20, padding: 40,
    }}>
      <Mascot size={120} variant="sleepy" />
      <div style={{ textAlign: "center", maxWidth: 360 }}>
        <h2 style={{ fontSize: 22, fontWeight: 500, margin: "0 0 8px", color: "var(--fg)" }}>
          No providers configured
        </h2>
        <p style={{ fontSize: 14, color: "var(--fg-3)", margin: 0, lineHeight: 1.6 }}>
          Add an API key in Settings to start chatting with Magnus.
        </p>
      </div>
      {onSettings && (
        <button onClick={onSettings} style={{
          padding: "9px 20px", borderRadius: 8, border: "none",
          background: "var(--brand)", color: "var(--on-brand)",
          fontFamily: "var(--mg-sans)", fontSize: 13, fontWeight: 500, cursor: "pointer",
        }}>Open Settings</button>
      )}
    </div>
  );
}

// Model-switch divider shown between messages from different models
function ModelSwitchDivider({ from, to, provider }: { from: string; to: string; provider?: ProviderConfig }) {
  const dot = provider ? providerDot(provider) : "var(--fg-3)";
  return (
    <div style={{
      display: "flex", alignItems: "center", gap: 10,
      margin: "8px 0", color: "var(--fg-3)",
    }}>
      <div style={{ flex: 1, height: 1, borderTop: "1px dashed var(--line)" }} />
      <span style={{
        fontSize: 10, fontFamily: "var(--mg-mono)", letterSpacing: "0.06em",
        display: "flex", alignItems: "center", gap: 6, whiteSpace: "nowrap",
      }}>
        <span style={{ width: 6, height: 6, borderRadius: "50%", background: dot }} />
        {from}
        <span style={{ opacity: 0.5 }}>→</span>
        {to}
      </span>
      <div style={{ flex: 1, height: 1, borderTop: "1px dashed var(--line)" }} />
    </div>
  );
}

interface ChatAreaProps {
  messages: Message[];
  loading: boolean;
  input: string;
  hasProviders: boolean;
  settings: Settings | null;
  activeChat: Chat | null;
  effectiveProvider: ProviderConfig | null;
  selectedModelId: string | null;
  onModelChange: (modelId: string) => void;
  onProviderChange: (providerId: string) => void;
  onInputChange: (value: string) => void;
  onSend: () => void;
  chatName?: string;
  theme: "dark" | "light";
  onToggleTheme: () => void;
  onSettings?: () => void;
}

export function ChatArea({
  messages, loading, input, hasProviders, settings, activeChat,
  effectiveProvider, selectedModelId, onModelChange, onProviderChange,
  onInputChange, onSend, chatName, theme, onToggleTheme, onSettings,
}: ChatAreaProps) {
  const bottomRef = useRef<HTMLDivElement>(null);
  const [focused, setFocused] = useState(false);
  const hasMessages = messages.length > 0;
  const providerLocked = hasMessages;
  const canSend = hasProviders && !loading && !!input.trim() && !!selectedModelId;

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Helper: find provider for a model_id tag
  const providerForModel = (modelId?: string): ProviderConfig | undefined => {
    if (!modelId || !settings) return undefined;
    const guessedBuiltIn =
      modelId.startsWith("claude") ? "anthropic" :
      modelId.startsWith("gpt") || modelId.startsWith("o") ? "open_ai" :
      modelId.startsWith("gemini") ? "google" : null;
    if (guessedBuiltIn) {
      const found = settings.providers.find((p) => {
        return p.kind === "built_in" && p.which === guessedBuiltIn;
      });
      if (found) return found;
    }
    return settings.providers.find((p) => p.id === activeChat?.provider_id);
  };

  return (
    <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", overflow: "hidden", background: "var(--bg)", position: "relative" }}>
      {/* Header */}
      <header style={{
        height: 48, flexShrink: 0, padding: "0 20px",
        display: "flex", alignItems: "center", justifyContent: "space-between",
        borderBottom: "1px solid var(--line)", gap: 12,
      }}>
        {/* Left: title or provider picker */}
        <div style={{ flex: 1, minWidth: 0, display: "flex", alignItems: "center", gap: 10 }}>
          {!providerLocked && hasProviders && settings && settings.providers.length > 0 ? (
            <ProviderPicker
              providers={settings.providers}
              value={activeChat?.provider_id ?? settings.default_provider_id ?? null}
              onChange={onProviderChange}
            />
          ) : (
            <span style={{ fontSize: 13, fontWeight: 500, color: "var(--fg)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {chatName ?? "New chat"}
            </span>
          )}

          {/* Passive model chip when chat has messages */}
          {providerLocked && effectiveProvider && selectedModelId && (
            <span
              title="Change model in the composer below"
              style={{
                display: "flex", alignItems: "center", gap: 5,
                fontSize: 11, fontFamily: "var(--mg-mono)",
                padding: "3px 8px", borderRadius: 6,
                background: "var(--bg-2)", border: "1px solid var(--line)",
                color: "var(--fg-3)", cursor: "default", flexShrink: 0,
              }}
            >
              <span style={{ width: 6, height: 6, borderRadius: "50%", background: providerDot(effectiveProvider) }} />
              {effectiveProvider.display_name}
            </span>
          )}
        </div>

        {/* Right: theme toggle */}
        <button
          onClick={onToggleTheme}
          title={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
          style={{
            width: 30, height: 30, display: "flex", alignItems: "center", justifyContent: "center",
            background: "transparent", border: "1px solid var(--line)",
            borderRadius: 8, color: "var(--fg-3)", cursor: "pointer", flexShrink: 0,
          }}
        >
          {theme === "dark" ? <SunIcon size={14} /> : <MoonIcon size={14} />}
        </button>
      </header>

      {/* Messages */}
      <div style={{ flex: 1, minHeight: 0, overflowY: "auto", overflowX: "hidden" }}>
        {!hasProviders
          ? <NoProviderState onSettings={onSettings} />
          : messages.length === 0
          ? <EmptyState onSuggest={onInputChange} />
          : (
            <div style={{ maxWidth: 680, margin: "0 auto", padding: "32px 24px 180px" }}>
              <div style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
                {messages.map((msg, i) => {
                  if (msg.role === "assistant" && msg.content === "") return null;

                  // Check if we need a model-switch divider before this message
                  const prevAssistants = messages.slice(0, i).filter((m) => m.role === "assistant");
                  const prevAssistant = prevAssistants[prevAssistants.length - 1];
                  const showDivider =
                    msg.role === "assistant" &&
                    prevAssistant?.model_id &&
                    msg.model_id &&
                    prevAssistant.model_id !== msg.model_id;

                  const msgProvider = providerForModel(msg.model_id);

                  return (
                    <div key={i}>
                      {showDivider && (
                        <ModelSwitchDivider
                          from={prevAssistant!.model_id!}
                          to={msg.model_id!}
                          provider={msgProvider}
                        />
                      )}
                      <div style={{ display: "flex", justifyContent: msg.role === "user" ? "flex-end" : "flex-start" }}>
                        <div style={{
                          maxWidth: "80%",
                          padding: "0.6rem 0.9rem",
                          borderRadius: "var(--mg-r-md)",
                          background: msg.role === "user" ? "var(--brand)" : "var(--bg-2)",
                          color: msg.role === "user" ? "var(--on-brand)" : "var(--fg)",
                          whiteSpace: msg.role === "user" ? "pre-wrap" : undefined,
                          fontSize: 14, lineHeight: 1.6,
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
                                      borderRadius: 4, padding: "0.1em 0.35em",
                                      fontSize: "0.88em", fontFamily: "var(--mg-mono)",
                                    }} {...props}>{children}</code>
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
                          {/* Per-message model tag */}
                          {msg.role === "assistant" && msg.model_id && (
                            <div style={{
                              marginTop: 6, fontSize: 10, fontFamily: "var(--mg-mono)",
                              color: "var(--fg-3)", display: "flex", alignItems: "center", gap: 5,
                            }}>
                              {msgProvider && (
                                <span style={{ width: 5, height: 5, borderRadius: "50%", background: providerDot(msgProvider) }} />
                              )}
                              via {msgProvider?.display_name ?? "unknown"} · {msg.model_id}
                            </div>
                          )}
                        </div>
                      </div>
                    </div>
                  );
                })}
                {loading && (
                  <div style={{ display: "flex", justifyContent: "flex-start" }}>
                    <div style={{
                      padding: "0.6rem 0.9rem", borderRadius: "var(--mg-r-md)",
                      background: "var(--bg-2)", color: "var(--brand)",
                      display: "flex", alignItems: "center",
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
        position: "absolute", bottom: 20, left: "50%", transform: "translateX(-50%)",
        width: "min(680px, calc(100% - 40px))", zIndex: 10,
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
            placeholder={hasProviders ? "Ask Magnus…" : "Configure a provider in settings first"}
            disabled={!hasProviders || loading}
            rows={1}
            style={{
              width: "100%", resize: "none", border: "none", outline: "none",
              background: "transparent", color: "var(--fg)",
              fontFamily: "var(--mg-sans)", fontSize: 14, lineHeight: 1.5,
              padding: "4px 0 6px", boxSizing: "border-box",
            }}
          />
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            {/* Model picker */}
            <ModelPicker
              provider={effectiveProvider}
              value={selectedModelId}
              onChange={onModelChange}
              disabled={!hasProviders || loading}
            />
            <div style={{ flex: 1 }} />
            <span style={{ fontSize: 11, color: "var(--fg-3)", fontFamily: "var(--mg-mono)" }}>
              {input.length ? `${input.length} chars` : "idle"}
            </span>
            <button
              aria-label="Send"
              onClick={onSend}
              disabled={!canSend}
              style={{
                width: 32, height: 32, borderRadius: 9, border: "none",
                background: canSend ? "var(--brand)" : "var(--bg)",
                color: canSend ? "var(--on-brand)" : "var(--fg-3)",
                display: "flex", alignItems: "center", justifyContent: "center",
                cursor: canSend ? "pointer" : "default",
                transition: "background 0.15s, color 0.15s", flexShrink: 0,
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
