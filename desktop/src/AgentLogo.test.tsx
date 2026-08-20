import { render } from "@testing-library/react";
import { AgentLogo } from "./AgentLogo";

describe("AgentLogo", () => {
  it("uses a bundled catalog image for a known Agent", () => {
    const { container } = render(<AgentLogo agentId="reasonix" name="Reasonix" command="reasonix" />);
    expect(container.querySelector("img")).toHaveAttribute("src");
  });

  it("falls back to a local monogram for an unknown Agent", () => {
    const { container } = render(<AgentLogo agentId="private-agent" name="Private Agent" />);
    expect(container.querySelector("img")).toBeNull();
    expect(container).toHaveTextContent("P");
  });
});
