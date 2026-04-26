import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import ProviderEditModal from "../components/ProviderEditModal";

describe("ProviderEditModal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("requires an API key when creating a new provider", async () => {
    render(
      <ProviderEditModal
        mode={{ type: "built_in", which: "anthropic" }}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(screen.getByText("API key is required.")).toBeInTheDocument();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("submits the selected built-in provider and api key", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const onSaved = vi.fn();
    const onClose = vi.fn();

    render(
      <ProviderEditModal
        mode={{ type: "built_in", which: "anthropic" }}
        onClose={onClose}
        onSaved={onSaved}
      />
    );

    await userEvent.type(screen.getByPlaceholderText("sk-ant-…"), "sk-ant-test");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(invoke).toHaveBeenCalledWith("upsert_provider", {
      provider: {
        id: "anthropic",
        display_name: "Anthropic",
        kind: "built_in",
        which: "anthropic",
      },
      apiKey: "sk-ant-test",
    });
    expect(onSaved).toHaveBeenCalledWith("anthropic", true);
    expect(onClose).toHaveBeenCalled();
  });
});
