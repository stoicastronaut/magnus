import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import ProviderPicker from "../components/ProviderPicker";
import { ProviderConfig } from "../services/tauri";

const providers: ProviderConfig[] = [
  {
    id: "anthropic",
    display_name: "Anthropic",
    kind: "built_in",
    which: "anthropic",
  },
  {
    id: "open_ai",
    display_name: "OpenAI",
    kind: "built_in",
    which: "open_ai",
  },
  {
    id: "corp-openai",
    display_name: "Corp OpenAI",
    kind: "custom",
    protocol: "open_ai",
    base_url: "https://proxy.example.com/v1/",
  },
];

describe("ProviderPicker", () => {
  it("shows the selected provider", () => {
    render(
      <ProviderPicker
        providers={providers}
        value="anthropic"
        onChange={vi.fn()}
      />
    );

    expect(screen.getByRole("button", { name: /anthropic/i })).toBeInTheDocument();
  });

  it("shows a placeholder when no provider is selected", () => {
    render(
      <ProviderPicker providers={providers} value={null} onChange={vi.fn()} />
    );

    expect(screen.getByRole("button", { name: /select provider/i })).toBeInTheDocument();
  });

  it("opens the provider menu", async () => {
    render(
      <ProviderPicker
        providers={providers}
        value="anthropic"
        onChange={vi.fn()}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: /anthropic/i }));

    expect(screen.getByRole("button", { name: /^openai$/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /corp openai/i })).toBeInTheDocument();
  });

  it("shows an empty state when opened without providers", async () => {
    render(<ProviderPicker providers={[]} value={null} onChange={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: /select provider/i }));

    expect(screen.getByText("No providers configured")).toBeInTheDocument();
  });

  it("calls onChange and closes after selecting a provider", async () => {
    const onChange = vi.fn();
    render(
      <ProviderPicker
        providers={providers}
        value="anthropic"
        onChange={onChange}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: /anthropic/i }));
    await userEvent.click(screen.getByRole("button", { name: /^openai$/i }));

    expect(onChange).toHaveBeenCalledWith("open_ai");
    expect(screen.queryByRole("button", { name: /corp openai/i })).not.toBeInTheDocument();
  });

  it("closes when focus leaves the picker", async () => {
    render(
      <>
        <ProviderPicker
          providers={providers}
          value="anthropic"
          onChange={vi.fn()}
        />
        <button>Outside</button>
      </>
    );

    await userEvent.click(screen.getByRole("button", { name: /anthropic/i }));
    expect(screen.getByRole("button", { name: /corp openai/i })).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Outside" }));

    expect(screen.queryByRole("button", { name: /corp openai/i })).not.toBeInTheDocument();
  });

  it("marks the selected provider in the open menu", async () => {
    render(
      <ProviderPicker
        providers={providers}
        value="anthropic"
        onChange={vi.fn()}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: /anthropic/i }));

    expect(screen.getByText("✓")).toBeInTheDocument();
  });
});
