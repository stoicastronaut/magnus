import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsPage } from "../components/SettingsPage";
import {
  getRecentDiagnostics,
  getSettings,
  hasApiKey,
  listModels,
} from "../services/tauri";

vi.mock("../services/tauri", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../services/tauri")>();
  return {
    ...actual,
    getRecentDiagnostics: vi.fn(),
    getSettings: vi.fn(),
    hasApiKey: vi.fn(),
    listModels: vi.fn(),
  };
});

const settings = {
  default_provider_id: "anthropic",
  providers: [
    {
      id: "anthropic",
      display_name: "Anthropic",
      kind: "built_in" as const,
      which: "anthropic" as const,
    },
    {
      id: "open_ai",
      display_name: "OpenAI",
      kind: "built_in" as const,
      which: "open_ai" as const,
    },
  ],
};

const models = [
  { id: "claude-haiku-4-5-20251001", display_name: "Haiku 4.5" },
  { id: "claude-sonnet-4-6", display_name: "Sonnet 4.6" },
];

describe("SettingsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(getRecentDiagnostics).mockResolvedValue([]);
    vi.mocked(getSettings).mockResolvedValue(settings);
    vi.mocked(hasApiKey).mockImplementation(async (providerId) => providerId === "anthropic");
    vi.mocked(listModels).mockResolvedValue(models);
  });

  it("loads providers, API key status, and models for the selected provider", async () => {
    render(
      <SettingsPage
        onBack={vi.fn()}
        theme="dark"
        onThemeChange={vi.fn()}
        activeChatId="chat-1"
      />
    );

    expect(screen.getByRole("heading", { name: "API Configuration" })).toBeInTheDocument();

    await waitFor(() => {
      expect(getSettings).toHaveBeenCalled();
    });

    expect(hasApiKey).toHaveBeenCalledWith("anthropic");
    expect(hasApiKey).toHaveBeenCalledWith("open_ai");
    expect(listModels).toHaveBeenCalledWith("anthropic");

    expect(screen.getAllByText("Anthropic").length).toBeGreaterThan(0);
    expect(screen.getAllByText("OpenAI").length).toBeGreaterThan(0);
    expect(screen.getByText("default")).toBeInTheDocument();
    expect(screen.getAllByText(/connected/).length).toBeGreaterThan(0);
    expect(screen.getAllByText("○ not set").length).toBeGreaterThan(0);

    await waitFor(() => {
      expect(screen.getByText("Default model")).toBeInTheDocument();
    });
    expect(screen.getByRole("option", { name: "Haiku 4.5" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Sonnet 4.6" })).toBeInTheDocument();
    expect(screen.getByText("2 models available")).toBeInTheDocument();
  });
});
