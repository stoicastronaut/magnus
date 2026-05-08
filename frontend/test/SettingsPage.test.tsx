import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsPage } from "../components/SettingsPage";
import {
  deleteProvider,
  getRecentDiagnostics,
  getSettings,
  hasApiKey,
  listModels,
  logDiagnosticError,
  setDefaultProvider,
  upsertProvider,
} from "../services/tauri";

vi.mock("../services/tauri", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../services/tauri")>();
  return {
    ...actual,
    getRecentDiagnostics: vi.fn(),
    getSettings: vi.fn(),
    hasApiKey: vi.fn(),
    listModels: vi.fn(),
    upsertProvider: vi.fn(),
    deleteProvider: vi.fn(),
    setDefaultProvider: vi.fn(),
    logDiagnosticError: vi.fn(),
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

const openAiModels = [
  { id: "gpt-5", display_name: "GPT-5" },
  { id: "gpt-5-mini", display_name: "GPT-5 mini" },
];

function renderSettingsPage() {
  return render(
    <SettingsPage
      onBack={vi.fn()}
      theme="dark"
      onThemeChange={vi.fn()}
      activeChatId="chat-1"
    />
  );
}

describe("SettingsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(getRecentDiagnostics).mockResolvedValue([]);
    vi.mocked(getSettings).mockResolvedValue(settings);
    vi.mocked(hasApiKey).mockImplementation(async (providerId) => providerId === "anthropic");
    vi.mocked(listModels).mockImplementation(async (providerId) =>
      providerId === "open_ai" ? openAiModels : models
    );
    vi.mocked(upsertProvider).mockResolvedValue(undefined);
    vi.mocked(deleteProvider).mockResolvedValue(undefined);
    vi.mocked(setDefaultProvider).mockResolvedValue(undefined);
  });

  it("loads providers, API key status, and models for the selected provider", async () => {
    renderSettingsPage();

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

  it("selects another provider and loads that provider's models", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /openai/i })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /openai/i }));

    await waitFor(() => {
      expect(listModels).toHaveBeenCalledWith("open_ai");
    });
    expect(screen.getByRole("heading", { name: "OpenAI" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "GPT-5" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "GPT-5 mini" })).toBeInTheDocument();
  });

  it("requires an API key before saving a provider without a stored key", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /openai/i })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /openai/i }));
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(screen.getByText("Please enter an API key.")).toBeInTheDocument();
    expect(upsertProvider).not.toHaveBeenCalled();
  });

  it("saves an API key for the selected provider and reloads settings", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByPlaceholderText("•••••• (leave blank to keep current)")).toBeInTheDocument();
    });
    await userEvent.type(
      screen.getByPlaceholderText("•••••• (leave blank to keep current)"),
      "sk-ant-new"
    );
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(upsertProvider).toHaveBeenCalledWith(settings.providers[0], "sk-ant-new");
    });
    expect(getSettings).toHaveBeenCalledTimes(2);
  });

  it("sets the selected provider as default and reloads settings", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /openai/i })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /openai/i }));
    await userEvent.click(screen.getByRole("button", { name: "Set as default" }));

    await waitFor(() => {
      expect(setDefaultProvider).toHaveBeenCalledWith("open_ai");
    });
    expect(getSettings).toHaveBeenCalledTimes(2);
  });

  it("deletes the selected provider and reloads settings", async () => {
    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /openai/i })).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /openai/i }));
    await userEvent.click(screen.getByRole("button", { name: "Remove" }));

    await waitFor(() => {
      expect(deleteProvider).toHaveBeenCalledWith("open_ai");
    });
    expect(getSettings).toHaveBeenCalledTimes(2);
  });

  it("logs diagnostics when provider save, default update, or delete fail", async () => {
    vi.mocked(upsertProvider).mockRejectedValueOnce(new Error("save failed"));
    vi.mocked(setDefaultProvider).mockRejectedValueOnce(new Error("default failed"));
    vi.mocked(deleteProvider).mockRejectedValueOnce(new Error("delete failed"));

    renderSettingsPage();

    await waitFor(() => {
      expect(screen.getByPlaceholderText("•••••• (leave blank to keep current)")).toBeInTheDocument();
    });
    await userEvent.type(
      screen.getByPlaceholderText("•••••• (leave blank to keep current)"),
      "sk-ant-new"
    );
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(logDiagnosticError).toHaveBeenCalledWith("Provider save failed", {
        provider_id: "anthropic",
        error: "Error: save failed",
      });
    });

    await userEvent.click(screen.getByRole("button", { name: /openai/i }));
    await userEvent.click(screen.getByRole("button", { name: "Set as default" }));

    await waitFor(() => {
      expect(logDiagnosticError).toHaveBeenCalledWith("Default provider update failed", {
        provider_id: "open_ai",
        error: "Error: default failed",
      });
    });

    await userEvent.click(screen.getByRole("button", { name: "Remove" }));

    await waitFor(() => {
      expect(logDiagnosticError).toHaveBeenCalledWith("Provider delete failed", {
        provider_id: "open_ai",
        error: "Error: delete failed",
      });
    });
  });
});
