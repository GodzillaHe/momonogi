import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "./App";

describe("Momonogi desktop shell", () => {
  it("shows the agent workbench and browser bridge status", async () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "Access matrix" })).toBeInTheDocument();
    expect(await screen.findByText("Claude Code")).toBeInTheDocument();
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
    await screen.findByText("Claude Code");

    await user.click(screen.getByRole("button", { name: "Open Claude Code" }));

    const inspector = screen.getByRole("complementary", { name: "Claude Code" });
    expect(within(inspector).getByText("Configuration paths")).toBeInTheDocument();
    expect(within(inspector).getByText("~/.claude/settings.json")).toBeInTheDocument();
    expect(within(inspector).getByText("Hooks active")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Close Agent details" })).toBeInTheDocument();
  });
});
