import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { ChatArea } from "../components/ChatArea";

const defaultProps = {
  messages: [],
  loading: false,
  input: "",
  hasSettings: true,
  onInputChange: vi.fn(),
  onSend: vi.fn(),
  theme: "dark" as const,
  onToggleTheme: vi.fn(),
};

describe("ChatArea", () => {
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
    const messages = [{ role: "user" as const, content: "Hello" }];
    render(<ChatArea {...defaultProps} messages={messages} />);
    expect(screen.queryByText("What shall we chase today?")).not.toBeInTheDocument();
  });

  it("renders user and assistant messages", () => {
    const messages = [
      { role: "user" as const, content: "Hello" },
      { role: "assistant" as const, content: "Hi there!" },
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

  it("disables input and send button when hasSettings is false", () => {
    render(<ChatArea {...defaultProps} hasSettings={false} />);
    expect(screen.getByPlaceholderText("Configure API key in settings first")).toBeDisabled();
    expect(screen.getByRole("button", { name: "Send" })).toBeDisabled();
  });

  it("disables send button when input is empty", () => {
    render(<ChatArea {...defaultProps} input="" />);
    expect(screen.getByRole("button", { name: "Send" })).toBeDisabled();
  });

  it("enables send button when input has text and settings configured", () => {
    render(<ChatArea {...defaultProps} input="Hello" />);
    expect(screen.getByRole("button", { name: "Send" })).not.toBeDisabled();
  });

  it("calls onInputChange when typing", async () => {
    const onInputChange = vi.fn();
    render(<ChatArea {...defaultProps} onInputChange={onInputChange} />);
    await userEvent.type(screen.getByPlaceholderText("Ask Magnus…"), "a");
    expect(onInputChange).toHaveBeenCalledWith("a");
  });

  it("calls onSend when Send button is clicked", async () => {
    const onSend = vi.fn();
    render(<ChatArea {...defaultProps} input="Hello" onSend={onSend} />);
    await userEvent.click(screen.getByRole("button", { name: "Send" }));
    expect(onSend).toHaveBeenCalled();
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
