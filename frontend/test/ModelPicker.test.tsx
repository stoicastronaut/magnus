import { invoke } from "@tauri-apps/api/core";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import ModelPicker from "../components/ModelPicker";
import { ProviderConfig } from "../services/tauri";

const provider: ProviderConfig = {
  id: "anthropic",
  display_name: "Anthropic",
  kind: "built_in",
  which: "anthropic",
};

const models = [
  { id: "claude-haiku-4-5-20251001", display_name: "Haiku 4.5" },
  { id: "claude-sonnet-4-6", display_name: "Sonnet 4.6" },
];

describe("ModelPicker", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue(models);
  });

  it("loads models for the selected provider", async () => {
    render(
      <ModelPicker provider={provider} value={null} onChange={vi.fn()} />
    );

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("list_models", {
        providerId: "anthropic",
      });
    });
  });

  it("auto-selects the first model when no model is selected", async () => {
    const onChange = vi.fn();

    render(
      <ModelPicker provider={provider} value={null} onChange={onChange} />
    );

    await waitFor(() => {
      expect(onChange).toHaveBeenCalledWith("claude-haiku-4-5-20251001");
    });
  });

  it("does not auto-select when a model value already exists", async () => {
    const onChange = vi.fn();

    render(
      <ModelPicker
        provider={provider}
        value="claude-sonnet-4-6"
        onChange={onChange}
      />
    );

    await waitFor(() => {
      expect(screen.getByText("Sonnet 4.6")).toBeInTheDocument();
    });
    expect(onChange).not.toHaveBeenCalled();
  });

  it("opens the model menu and lists models", async () => {
    render(
      <ModelPicker
        provider={provider}
        value="claude-sonnet-4-6"
        onChange={vi.fn()}
      />
    );

    await waitFor(() => {
      expect(screen.getByText("Sonnet 4.6")).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /anthropic/i }));

    expect(screen.getByText("claude-haiku-4-5-20251001")).toBeInTheDocument();
    expect(screen.getByText("claude-sonnet-4-6")).toBeInTheDocument();
    expect(screen.getByText("✓")).toBeInTheDocument();
  });

  it("calls onChange and closes after selecting a model", async () => {
    const onChange = vi.fn();

    render(
      <ModelPicker
        provider={provider}
        value="claude-haiku-4-5-20251001"
        onChange={onChange}
      />
    );

    await waitFor(() => {
      expect(screen.getByText("Haiku 4.5")).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /anthropic/i }));
    await userEvent.click(screen.getByRole("button", { name: /sonnet 4\.6/i }));

    expect(onChange).toHaveBeenCalledWith("claude-sonnet-4-6");
    expect(screen.queryByRole("button", { name: /sonnet 4\.6/i })).not.toBeInTheDocument();
  });

  it("handles model load failure by keeping the menu closed with no models", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("model load failed"));

    render(
      <ModelPicker provider={provider} value={null} onChange={vi.fn()} />
    );

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("list_models", {
        providerId: "anthropic",
      });
    });
    await userEvent.click(screen.getByRole("button", { name: /anthropic/i }));

    expect(screen.queryByText("claude-haiku-4-5-20251001")).not.toBeInTheDocument();
  });

  it("is disabled when no provider is selected", async () => {
    render(<ModelPicker provider={null} value={null} onChange={vi.fn()} />);

    const button = screen.getByRole("button", { name: /no provider/i });
    expect(button).toBeDisabled();
    await userEvent.click(button);
    expect(invoke).not.toHaveBeenCalled();
  });

  it("is disabled when disabled is true", async () => {
    render(
      <ModelPicker
        provider={provider}
        value={null}
        onChange={vi.fn()}
        disabled
      />
    );

    const button = screen.getByRole("button", { name: /anthropic/i });
    expect(button).toBeDisabled();
    await userEvent.click(button);
    expect(screen.queryByText("claude-haiku-4-5-20251001")).not.toBeInTheDocument();
  });
});
