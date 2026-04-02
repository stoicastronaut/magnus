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
};

describe("ChatArea", () => {
  it("shows empty state when there are no messages", () => {
    render(<ChatArea {...defaultProps} />);
    expect(screen.getByText("Start a conversation")).toBeInTheDocument();
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

  it("shows loading dots when loading is true", () => {
    const { container } = render(<ChatArea {...defaultProps} loading={true} />);
    const dots = container.querySelectorAll("span[style*='border-radius: 50%']");
    expect(dots).toHaveLength(3);
  });

  it("disables input and send button when hasSettings is false", () => {
    render(<ChatArea {...defaultProps} hasSettings={false} />);
    expect(screen.getByPlaceholderText("Configure API key in settings first")).toBeDisabled();
    expect(screen.getByText("Send")).toBeDisabled();
  });

  it("disables send button when input is empty", () => {
    render(<ChatArea {...defaultProps} input="" />);
    expect(screen.getByText("Send")).toBeDisabled();
  });

  it("calls onInputChange when typing", async () => {
    const onInputChange = vi.fn();
    render(<ChatArea {...defaultProps} onInputChange={onInputChange} />);
    await userEvent.type(screen.getByPlaceholderText("Message Claude..."), "a");
    expect(onInputChange).toHaveBeenCalledWith("a");
  });

  it("calls onSend when Send button is clicked", async () => {
    const onSend = vi.fn();
    render(<ChatArea {...defaultProps} input="Hello" onSend={onSend} />);
    await userEvent.click(screen.getByText("Send"));
    expect(onSend).toHaveBeenCalled();
  });

  it("calls onSend when Enter is pressed", async () => {
    const onSend = vi.fn();
    render(<ChatArea {...defaultProps} input="Hello" onSend={onSend} />);
    fireEvent.keyDown(screen.getByPlaceholderText("Message Claude..."), { key: "Enter" });
    expect(onSend).toHaveBeenCalled();
  });
});
