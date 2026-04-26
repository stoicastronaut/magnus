import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { ChatArea } from "../components/ChatArea";
import { ProviderConfig, Settings } from "../services/tauri";

const anthropicProvider: ProviderConfig = {
  id: "anthropic",
  display_name: "Anthropic",
  kind: "built_in",
  which: "anthropic",
};

const openAiProvider: ProviderConfig = {
  id: "open_ai",
  display_name: "OpenAI",
  kind: "built_in",
  which: "open_ai",
};

const googleProvider: ProviderConfig = {
  id: "google",
  display_name: "Google",
  kind: "built_in",
  which: "google",
};

const providerSettings: Settings = {
  default_provider_id: "anthropic",
  providers: [anthropicProvider, openAiProvider, googleProvider],
};

const defaultProps = {
  messages: [],
  loading: false,
  input: "",
  hasProviders: true,
  settings: { default_provider_id: null, providers: [] },
  activeChat: null,
  effectiveProvider: null,
  selectedModelId: null,
  onModelChange: vi.fn(),
  onProviderChange: vi.fn(),
  onInputChange: vi.fn(),
  onSend: vi.fn(),
  theme: "dark" as const,
  onToggleTheme: vi.fn(),
};

describe("ChatArea", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows empty state when there are no messages", () => {
    render(<ChatArea {...defaultProps} />);
    expect(screen.getByText("Review a diff")).toBeInTheDocument();
  });

  it("renders all suggestion chips in empty state", () => {
    render(<ChatArea {...defaultProps} />);
    expect(screen.getByText("Explain this error")).toBeInTheDocument();
    expect(screen.getByText("Rewrite this paragraph")).toBeInTheDocument();
    expect(screen.getByText("Plan my week")).toBeInTheDocument();
    expect(screen.getByText("Review a diff")).toBeInTheDocument();
    expect(screen.getByText("Brainstorm names")).toBeInTheDocument();
    expect(screen.getByText("Summarize a doc")).toBeInTheDocument();
  });

  it("clicking a suggestion chip calls onInputChange with chip text", async () => {
    const onInputChange = vi.fn();
    render(<ChatArea {...defaultProps} onInputChange={onInputChange} />);
    await userEvent.click(screen.getByText("Plan my week"));
    expect(onInputChange).toHaveBeenCalledWith("Plan my week");
  });

  it("hides empty state when messages exist", () => {
    const messages = [{ role: "user", content: "Hello" }];
    render(<ChatArea {...defaultProps} messages={messages} />);
    expect(screen.queryByText("What shall we chase today?")).not.toBeInTheDocument();
  });

  it("renders user and assistant messages", () => {
    const messages = [
      { role: "user", content: "Hello" },
      { role: "assistant", content: "Hi there!" },
    ];
    render(<ChatArea {...defaultProps} messages={messages} />);
    expect(screen.getByText("Hello")).toBeInTheDocument();
    expect(screen.getByText("Hi there!")).toBeInTheDocument();
  });

  it("shows chat name in header", () => {
    render(<ChatArea {...defaultProps} chatName="My project" />);
    expect(screen.getByText("My project")).toBeInTheDocument();
  });

  it("shows 'New chat' in header when chatName is not provided", () => {
    render(<ChatArea {...defaultProps} />);
    expect(screen.getByText("New chat")).toBeInTheDocument();
  });

  it("shows sun icon and light mode title when theme is dark", () => {
    render(<ChatArea {...defaultProps} theme="dark" />);
    expect(screen.getByTitle("Switch to light mode")).toBeInTheDocument();
  });

  it("shows moon icon and dark mode title when theme is light", () => {
    render(<ChatArea {...defaultProps} theme="light" />);
    expect(screen.getByTitle("Switch to dark mode")).toBeInTheDocument();
  });

  it("calls onToggleTheme when theme button is clicked", async () => {
    const onToggleTheme = vi.fn();
    render(<ChatArea {...defaultProps} onToggleTheme={onToggleTheme} />);
    await userEvent.click(screen.getByTitle("Switch to light mode"));
    expect(onToggleTheme).toHaveBeenCalled();
  });

  it("shows char count when input has text", () => {
    render(<ChatArea {...defaultProps} input="Hello" />);
    expect(screen.getByText("5 chars")).toBeInTheDocument();
  });

  it("shows idle when input is empty", () => {
    render(<ChatArea {...defaultProps} input="" />);
    expect(screen.getByText("idle")).toBeInTheDocument();
  });

  it("disables send button when loading is true", () => {
    render(<ChatArea {...defaultProps} loading={true} input="Hello" />);
    expect(screen.getByRole("button", { name: "Send" })).toBeDisabled();
  });

  it("shows no-provider state when hasProviders is false", () => {
    render(<ChatArea {...defaultProps} hasProviders={false} />);
    expect(screen.getByText("No providers configured")).toBeInTheDocument();
  });

  it("shows the provider picker for a new chat when providers exist", () => {
    render(<ChatArea {...defaultProps} settings={providerSettings} />);
    expect(screen.getByText("Anthropic")).toBeInTheDocument();
    expect(screen.queryByText("New chat")).not.toBeInTheDocument();
  });

  it("shows provider and model metadata for assistant responses", () => {
    vi.mocked(invoke).mockResolvedValue([
      { id: "gpt-5", display_name: "GPT-5" },
    ]);
    const messages = [
      { role: "assistant", content: "I can help with that.", model_id: "gpt-5" },
    ];
    render(
      <ChatArea
        {...defaultProps}
        messages={messages}
        settings={providerSettings}
        activeChat={{ id: "1", name: "Chat", messages, created_at: "01-01-25", provider_id: "open_ai" }}
        effectiveProvider={openAiProvider}
        selectedModelId="gpt-5"
      />
    );

    expect(screen.getByText(/via OpenAI · gpt-5/)).toBeInTheDocument();
  });

  it("shows a model switch divider when assistant model ids change", async () => {
    vi.mocked(invoke).mockResolvedValue([
      { id: "gpt-5", display_name: "GPT-5" },
      { id: "gemini-2.5-pro", display_name: "Gemini 2.5 Pro" },
    ]);
    const messages = [
      { role: "assistant", content: "First answer", model_id: "gpt-5" },
      { role: "assistant", content: "Second answer", model_id: "gemini-2.5-pro" },
    ];
    render(
      <ChatArea
        {...defaultProps}
        messages={messages}
        settings={providerSettings}
        activeChat={{ id: "1", name: "Chat", messages, created_at: "01-01-25", provider_id: "open_ai" }}
        effectiveProvider={openAiProvider}
        selectedModelId="gemini-2.5-pro"
      />
    );

    await waitFor(() => {
      expect(screen.getByText("Gemini 2.5 Pro")).toBeInTheDocument();
    });
    expect(
      screen.getByText((_, element) =>
        element?.tagName === "SPAN" &&
        element.textContent?.replace(/\s+/g, "") === "gpt-5→gemini-2.5-pro"
      )
    ).toBeInTheDocument();
    expect(screen.getByText(/via Google · gemini-2.5-pro/)).toBeInTheDocument();
  });

  it("disables send button when input is empty", () => {
    render(<ChatArea {...defaultProps} input="" />);
    expect(screen.getByRole("button", { name: "Send" })).toBeDisabled();
  });

  it("calls onInputChange when typing", async () => {
    const onInputChange = vi.fn();
    render(<ChatArea {...defaultProps} onInputChange={onInputChange} />);
    await userEvent.type(screen.getByPlaceholderText("Ask Magnus…"), "a");
    expect(onInputChange).toHaveBeenCalledWith("a");
  });

  it("calls onSend when Enter is pressed", async () => {
    const onSend = vi.fn();
    render(<ChatArea {...defaultProps} input="Hello" onSend={onSend} />);
    fireEvent.keyDown(screen.getByPlaceholderText("Ask Magnus…"), { key: "Enter" });
    expect(onSend).toHaveBeenCalled();
  });

  it("does not call onSend when Shift+Enter is pressed", async () => {
    const onSend = vi.fn();
    render(<ChatArea {...defaultProps} input="Hello" onSend={onSend} />);
    fireEvent.keyDown(screen.getByPlaceholderText("Ask Magnus…"), { key: "Enter", shiftKey: true });
    expect(onSend).not.toHaveBeenCalled();
  });
});
