import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { applyAgentConfiguration, changeMemoryTag, previewAgentConfiguration, resetBrowserBridgeForTests, setAgentAccess } from "./bridge";
import { App } from "./App";

describe("Momonogi desktop shell", () => {
  beforeEach(() => {
    resetBrowserBridgeForTests();
  });

  it("supports keyboard-only navigation and skips disabled access controls", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole("heading", { name: "Agent access" });

    await user.tab();
    expect(screen.getByRole("button", { name: "Agents" })).toHaveFocus();
    await user.tab();
    expect(screen.getByRole("button", { name: "Memories" })).toHaveFocus();
    await user.keyboard("{Enter}");
    expect(await screen.findByRole("heading", { name: "Memory explorer" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Agents" }));
    const writerIdentity = await screen.findByRole("combobox", { name: "Writer identity" });
    writerIdentity.focus();
    await user.tab();
    expect(screen.getByRole("button", { name: "Open Codex" })).toHaveFocus();
    expect(screen.getByRole("button", { name: "Codex: Writer" })).toBeDisabled();
  });

  it("does not present a hard-coded tag count in the toolbar", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole("heading", { name: "Agent access" });

    await user.click(screen.getByRole("button", { name: "Tags" }));

    expect(await screen.findByRole("heading", { name: "Tag index" })).toBeInTheDocument();
    expect(screen.getByText("All stores")).toBeInTheDocument();
    expect(screen.queryByText("6 indexed")).not.toBeInTheDocument();
  });

  it("shows the agent workbench and browser bridge status", async () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "Access matrix" })).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "Claude Code" })).toBeInTheDocument();
    expect(await screen.findByText("Development bridge")).toBeInTheDocument();
  });

  it("switches between Chinese and English and persists the choice", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole("heading", { name: "Agent access" });

    await user.click(screen.getByRole("button", { name: "Switch to Chinese" }));

    expect(await screen.findByRole("heading", { name: "Agent 权限" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "记忆" })).toBeInTheDocument();
    expect(screen.getByRole("searchbox", { name: "搜索 Agent" })).toBeInTheDocument();
    expect(document.documentElement).toHaveAttribute("lang", "zh-CN");
    expect(window.localStorage.getItem("momonogi.language")).toBe("zh-CN");

    await user.click(screen.getByRole("button", { name: "切换到英文" }));
    expect(await screen.findByRole("heading", { name: "Agent access" })).toBeInTheDocument();
    expect(document.documentElement).toHaveAttribute("lang", "en");
    expect(window.localStorage.getItem("momonogi.language")).toBe("en");
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

  it("groups stores and reads complete memory details", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Memories" }));

    expect(await screen.findByRole("group", { name: "Global" })).toBeInTheDocument();
    expect(screen.getByRole("group", { name: "Projects" })).toBeInTheDocument();
    await user.click(screen.getByRole("option", { name: /Momonogi Desktop/ }));
    expect(await screen.findByText(/Momonogi Desktop manages Agent access/)).toBeInTheDocument();
  });

  it("combines metadata and archive filters", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Memories" }));
    await screen.findByRole("option", { name: /Old layout decision/ });

    await user.selectOptions(screen.getByRole("combobox", { name: "Memory type" }), "feedback");
    await user.selectOptions(screen.getByRole("combobox", { name: "Memory scope" }), "repo");
    await user.selectOptions(screen.getByRole("combobox", { name: "Archive state" }), "archived");

    expect(screen.getByRole("option", { name: /Old layout decision/ })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: /Interface preferences/ })).not.toBeInTheDocument();
  });

  it("adds normalized tags and refreshes the note revision", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Memories" }));
    await user.click(await screen.findByRole("option", { name: /Momonogi Desktop/ }));
    await screen.findByText(/Momonogi Desktop manages Agent access/);

    await user.selectOptions(screen.getByRole("combobox", { name: "Tag writer identity" }), "codex");
    await user.type(screen.getByRole("textbox", { name: "New tag" }), "Design System");
    await user.click(screen.getByRole("button", { name: "Add tag" }));

    expect(await screen.findByText("Tag added at revision 9.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Remove tag design-system" })).toBeInTheDocument();
    expect(screen.getByText("r9")).toBeInTheDocument();
  });

  it("removes tags and treats duplicate additions as no-ops", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Memories" }));
    await screen.findByText(/Agent permissions must remain explicit/);
    await user.selectOptions(screen.getByRole("combobox", { name: "Tag writer identity" }), "codex");

    await user.type(screen.getByRole("textbox", { name: "New tag" }), "AGENTS");
    await user.click(screen.getByRole("button", { name: "Add tag" }));
    expect(await screen.findByText("Tags were already current.")).toBeInTheDocument();
    expect(screen.getByText("r4")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Remove tag agents" }));
    expect(await screen.findByText("Tag removed at revision 5.")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Remove tag agents" })).not.toBeInTheDocument();
  });

  it("reloads current tags after an ETag conflict", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Memories" }));
    await screen.findByText(/Agent permissions must remain explicit/);

    await changeMemoryTag({
      storePath: "~/.local/share/momonogi/store",
      slug: "agent-access-policy.md",
      tag: "external",
      action: "add",
      actor: "codex",
      ifMatch: "mock-agent-access",
    });
    await user.selectOptions(screen.getByRole("combobox", { name: "Tag writer identity" }), "codex");
    await user.click(screen.getByRole("button", { name: "Remove tag agents" }));

    expect(await screen.findByText("This memory changed elsewhere. Current tags were reloaded.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Remove tag external" })).toBeInTheDocument();
    expect(screen.getByText("r5")).toBeInTheDocument();
  });

  it("rejects tag writes from a reader", async () => {
    await expect(changeMemoryTag({
      storePath: "~/.local/share/momonogi/store",
      slug: "agent-access-policy.md",
      tag: "blocked",
      action: "add",
      actor: "opencode",
      ifMatch: "mock-agent-access",
    })).rejects.toThrow("not a configured writer");
  });

  it("keeps archived memory tags read-only", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Memories" }));
    await user.click(await screen.findByRole("option", { name: /Old layout decision/ }));

    expect(await screen.findByText("Archived memories are read-only.")).toBeInTheDocument();
    expect(screen.queryByRole("combobox", { name: "Tag writer identity" })).not.toBeInTheDocument();
    expect(screen.queryByRole("textbox", { name: "New tag" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Remove tag/ })).not.toBeInTheDocument();
  });

  it("derives the tag index from memories and opens matching notes", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Tags" }));

    expect(await screen.findByRole("row", { name: /design mixed 2/i })).toBeInTheDocument();
    expect(screen.getByText("7 tags")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Open design" }));

    expect(screen.getByRole("searchbox", { name: "Search memories" })).toHaveValue("design");
    expect(screen.getByRole("option", { name: /Interface preferences/ })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: /Old layout decision/ })).toBeInTheDocument();
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

  it("previews and applies host configuration after an access change", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("OpenCode");
    await user.selectOptions(screen.getByRole("combobox", { name: "Writer identity" }), "codex");
    await user.click(screen.getByRole("button", { name: "OpenCode: Writer" }));

    const preview = await screen.findByRole("region", { name: "OpenCode" });
    expect(within(preview).getByRole("listitem", { name: /AGENTS\.md, rules, Update/i })).toBeInTheDocument();
    expect(within(preview).getByText("1 file will change")).toBeInTheDocument();
    await user.click(within(preview).getByRole("button", { name: "Apply changes" }));

    expect(await screen.findByText("1 host configuration file was synchronized.")).toBeInTheDocument();
    expect(within(await screen.findByRole("region", { name: "OpenCode" })).getByText("Configuration is current")).toBeInTheDocument();
    expect(within(screen.getByRole("region", { name: "OpenCode" })).getByRole("button", { name: "Apply changes" })).toBeDisabled();
  });

  it("removes managed hooks when a writer becomes a reader", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole("heading", { name: "Claude Code" });
    await user.selectOptions(screen.getByRole("combobox", { name: "Writer identity" }), "codex");
    await user.click(screen.getByRole("button", { name: "Claude Code: Reader" }));

    const preview = await screen.findByRole("region", { name: "Claude Code" });
    expect(within(preview).getByRole("listitem", { name: /settings\.json, hooks, Remove Momonogi/i })).toBeInTheDocument();
    await user.click(within(preview).getByRole("button", { name: "Apply changes" }));

    expect(await screen.findByText("2 host configuration files were synchronized.")).toBeInTheDocument();
    expect(screen.getByText("Hooks off")).toBeInTheDocument();
  });

  it("refreshes a stale host configuration preview", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("OpenCode");
    await user.selectOptions(screen.getByRole("combobox", { name: "Writer identity" }), "codex");
    await user.click(screen.getByRole("button", { name: "OpenCode: Writer" }));
    const preview = await screen.findByRole("region", { name: "OpenCode" });

    const external = await previewAgentConfiguration("opencode");
    expect(external).not.toBeNull();
    await applyAgentConfiguration("opencode", external!.digest);
    await user.click(within(preview).getByRole("button", { name: "Apply changes" }));

    expect(await screen.findByText("Host configuration changed elsewhere. The preview was refreshed.")).toBeInTheDocument();
    expect(within(await screen.findByRole("region", { name: "OpenCode" })).getByText("Configuration is current")).toBeInTheDocument();
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
