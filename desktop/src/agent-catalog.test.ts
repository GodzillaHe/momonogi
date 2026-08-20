import { describe, expect, it } from "vitest";
import { agentCatalog, findCatalogAgent } from "./agent-catalog";

describe("built-in Agent catalog", () => {
  it("ships a broad initial catalog with local icons", () => {
    expect(agentCatalog.length).toBeGreaterThanOrEqual(30);
    expect(agentCatalog.every((agent) => agent.iconUrl.length > 0)).toBe(true);
  });

  it("keeps canonical ids unique", () => {
    const ids = agentCatalog.map((agent) => agent.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("matches ids, names, aliases, and commands locally", () => {
    expect(findCatalogAgent("claude")?.id).toBe("claude-code");
    expect(findCatalogAgent("OpenAI Codex")?.id).toBe("codex");
    expect(findCatalogAgent("RooCode")?.id).toBe("roo-code");
    expect(findCatalogAgent("gemini")?.id).toBe("gemini-cli");
    expect(findCatalogAgent("DeepSeek Reasonix")?.id).toBe("reasonix");
    expect(findCatalogAgent("hermes")?.id).toBe("hermes-agent");
    expect(findCatalogAgent("DSH")?.id).toBe("deepseek-harness");
  });

  it("does not guess unknown Agents", () => {
    expect(findCatalogAgent("private-company-agent")).toBeUndefined();
  });
});
