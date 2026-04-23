import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { Sidebar, Chat } from "../components/Sidebar";

const chats: Chat[] = [
  { id: "1", name: "First Chat", messages: [], created_at: "01-01-25" },
  { id: "2", name: "Second Chat", messages: [], created_at: "02-01-25" },
];

const defaultProps = {
  chats,
  activeChatId: "1",
  onSelectChat: vi.fn(),
  onNewChat: vi.fn(),
  onRename: vi.fn(),
  onDelete: vi.fn(),
  onSettings: vi.fn(),
};

describe("Sidebar", () => {
  it("renders all chats", () => {
    render(<Sidebar {...defaultProps} />);
    expect(screen.getByText("First Chat")).toBeInTheDocument();
    expect(screen.getByText("Second Chat")).toBeInTheDocument();
  });

  it("calls onSelectChat when a chat is clicked", async () => {
    const onSelectChat = vi.fn();
    render(<Sidebar {...defaultProps} onSelectChat={onSelectChat} />);
    await userEvent.click(screen.getByText("Second Chat"));
    expect(onSelectChat).toHaveBeenCalledWith("2");
  });

  it("calls onNewChat when + button is clicked", async () => {
    const onNewChat = vi.fn();
    render(<Sidebar {...defaultProps} onNewChat={onNewChat} />);
    await userEvent.click(screen.getByRole("button", { name: /new chat/i }));
    expect(onNewChat).toHaveBeenCalled();
  });

  it("shows context menu on right-click", () => {
    render(<Sidebar {...defaultProps} />);
    fireEvent.contextMenu(screen.getByText("First Chat"));
    expect(screen.getByText("Rename")).toBeInTheDocument();
    expect(screen.getByText("Delete")).toBeInTheDocument();
  });

  it("calls onDelete when Delete is clicked in context menu", async () => {
    const onDelete = vi.fn();
    render(<Sidebar {...defaultProps} onDelete={onDelete} />);
    fireEvent.contextMenu(screen.getByText("First Chat"));
    await userEvent.click(screen.getByText("Delete"));
    expect(onDelete).toHaveBeenCalledWith(chats[0]);
  });

  it("shows rename input when Rename is clicked in context menu", async () => {
    render(<Sidebar {...defaultProps} />);
    fireEvent.contextMenu(screen.getByText("First Chat"));
    await userEvent.click(screen.getByText("Rename"));
    expect(screen.getByDisplayValue("First Chat")).toBeInTheDocument();
  });

  it("calls onRename when rename is committed with Enter", async () => {
    const onRename = vi.fn();
    render(<Sidebar {...defaultProps} onRename={onRename} />);
    fireEvent.contextMenu(screen.getByText("First Chat"));
    await userEvent.click(screen.getByText("Rename"));
    const input = screen.getByDisplayValue("First Chat");
    await userEvent.clear(input);
    await userEvent.type(input, "Renamed Chat{Enter}");
    expect(onRename).toHaveBeenCalledWith("1", "Renamed Chat");
  });

  it("calls onSettings when Settings button is clicked", async () => {
    const onSettings = vi.fn();
    render(<Sidebar {...defaultProps} onSettings={onSettings} />);
    await userEvent.click(screen.getByTitle("Settings"));
    expect(onSettings).toHaveBeenCalled();
  });

  it("collapses sidebar when collapse button is clicked", async () => {
    render(<Sidebar {...defaultProps} />);
    await userEvent.click(screen.getByTitle("Collapse sidebar"));
    expect(screen.queryByText("First Chat")).not.toBeInTheDocument();
    expect(screen.getByTitle("Expand sidebar")).toBeInTheDocument();
  });

  it("expands sidebar when expand button is clicked after collapse", async () => {
    render(<Sidebar {...defaultProps} />);
    await userEvent.click(screen.getByTitle("Collapse sidebar"));
    await userEvent.click(screen.getByTitle("Expand sidebar"));
    expect(screen.getByText("First Chat")).toBeInTheDocument();
  });

  it("shows chat titles as button titles in collapsed mode", async () => {
    render(<Sidebar {...defaultProps} />);
    await userEvent.click(screen.getByTitle("Collapse sidebar"));
    expect(screen.getByTitle("First Chat")).toBeInTheDocument();
    expect(screen.getByTitle("Second Chat")).toBeInTheDocument();
  });

  it("calls onNewChat from collapsed rail button", async () => {
    const onNewChat = vi.fn();
    render(<Sidebar {...defaultProps} onNewChat={onNewChat} />);
    await userEvent.click(screen.getByTitle("Collapse sidebar"));
    await userEvent.click(screen.getByTitle("New chat"));
    expect(onNewChat).toHaveBeenCalled();
  });

  it("calls onSelectChat from collapsed rail when a chat button is clicked", async () => {
    const onSelectChat = vi.fn();
    render(<Sidebar {...defaultProps} onSelectChat={onSelectChat} />);
    await userEvent.click(screen.getByTitle("Collapse sidebar"));
    await userEvent.click(screen.getByTitle("Second Chat"));
    expect(onSelectChat).toHaveBeenCalledWith("2");
  });
});
