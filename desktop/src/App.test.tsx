import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { resetBrowserBridgeForTests, setAgentAccess } from "./bridge";
import { App } from "./App";

describe("Momonogi desktop shell", () => {
  beforeEach(() => {
    resetBrowserBridgeForTests();
  });

  it("shows the agent workbench and browser bridge status", async () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "Access matrix" })).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "Claude Code" })).toBeInTheDocument();
    expect(await screen.findByText("Development bridge")).toBeInTheDocument();
  });

  it("switches views and clears the current search", async () => {
    const user = userEvent.setup();
    render(<App />);

    const search = screen.getByRole("searchbox", { name: "Search agents" });
    await user.type(search, "codex");
    expect(search).toHaveValue("codex");

    await user.click(screen.getByRole("button", { name: "Memories" }));

    expect(screen.getByRole("heading", { name: "Indexed notes" })).toBeInTheDocument();
    expect(screen.getByRole("searchbox", { name: "Search memories" })).toHaveValue("");
  });

  it("filters memories by tag", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Memories" }));
    await user.type(screen.getByRole("searchbox", { name: "Search memories" }), "tauri");

    expect(screen.getByRole("option", { name: /Momonogi Desktop/ })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: /Agent access policy/ })).not.toBeInTheDocument();
  });

  it("opens discovered Agent configuration details", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole("heading", { name: "Claude Code" });

    await user.click(screen.getByRole("button", { name: "Open Claude Code" }));

    const inspector = screen.getByRole("complementary", { name: "Claude Code" });
    expect(within(inspector).getByText("Configuration paths")).toBeInTheDocument();
    expect(within(inspector).getByText("~/.claude/settings.json")).toBeInTheDocument();
    expect(within(inspector).getByText("Hooks active")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Close Agent details" })).toBeInTheDocument();
  });

  it("requires a writer identity before changing Agent access", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("OpenCode");

    const writerButton = screen.getByRole("button", { name: "OpenCode: Writer" });
    expect(writerButton).toBeDisabled();

    await user.selectOptions(screen.getByRole("combobox", { name: "Writer identity" }), "codex");
    expect(writerButton).toBeEnabled();
    await user.click(writerButton);

    expect(await screen.findByText("Access updated at revision 25.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "OpenCode: Writer" })).toHaveAttribute("aria-pressed", "true");
  });

  it("prevents removing the final writer", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole("heading", { name: "Claude Code" });
    await user.selectOptions(screen.getByRole("combobox", { name: "Writer identity" }), "codex");
    await user.click(screen.getByRole("button", { name: "Claude Code: Reader" }));
    await screen.findByText("Access updated at revision 25.");

    expect(screen.getByRole("button", { name: "Codex: Reader" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Codex: None" })).toBeDisabled();
  });

  it("reloads current roles after an ETag conflict", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole("heading", { name: "OpenCode" });

    await setAgentAccess({
      agentId: "opencode",
      role: "writer",
      actor: "codex",
      ifMatch: "browser-etag-24",
    });
    await user.selectOptions(screen.getByRole("combobox", { name: "Writer identity" }), "codex");
    await user.click(screen.getByRole("button", { name: "OpenClaw: Writer" }));

    expect(await screen.findByText("The store changed elsewhere. Current roles were reloaded.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "OpenCode: Writer" })).toHaveAttribute("aria-pressed", "true");
  });

  it("registers and removes a project store entry", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Settings" }));

    expect(await screen.findByRole("heading", { name: "Store registry" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "global" })).toBeInTheDocument();

    await user.type(screen.getByRole("textbox", { name: "Project store path" }), "/tmp/atlas/.momonogi");
    await user.click(screen.getByRole("button", { name: "Register" }));
    expect(await screen.findByRole("heading", { name: "atlas" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Remove atlas" }));
    await waitFor(() => expect(screen.queryByRole("heading", { name: "atlas" })).not.toBeInTheDocument());
  });
});
