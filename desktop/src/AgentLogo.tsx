import { useState } from "react";
import { findCatalogAgent } from "./agent-catalog";

export function AgentLogo({ agentId, name, command }: { agentId: string; name: string; command?: string }) {
  const [failed, setFailed] = useState(false);
  const catalogAgent = findCatalogAgent(agentId, name, command);

  return (
    <span className="agent-logo" aria-hidden="true">
      {catalogAgent && !failed ? (
        <img src={catalogAgent.iconUrl} alt="" width="24" height="24" onError={() => setFailed(true)} />
      ) : (
        <span>{name.slice(0, 1).toUpperCase()}</span>
      )}
    </span>
  );
}
