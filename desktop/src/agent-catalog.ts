import ampIcon from "@lobehub/icons-static-svg/icons/amp-color.svg?url";
import antigravityIcon from "@lobehub/icons-static-svg/icons/antigravity-color.svg?url";
import claudeCodeIcon from "@lobehub/icons-static-svg/icons/claude-color.svg?url";
import clineIcon from "@lobehub/icons-static-svg/icons/cline.svg?url";
import codeBuddyIcon from "@lobehub/icons-static-svg/icons/codebuddy-color.svg?url";
import codeFlickerIcon from "@lobehub/icons-static-svg/icons/codeflicker-color.svg?url";
import cursorIcon from "@lobehub/icons-static-svg/icons/cursor.svg?url";
import deepSeekIcon from "@lobehub/icons-static-svg/icons/deepseek-color.svg?url";
import devinIcon from "@lobehub/icons-static-svg/icons/devin-color.svg?url";
import geminiCliIcon from "@lobehub/icons-static-svg/icons/geminicli-color.svg?url";
import githubCopilotIcon from "@lobehub/icons-static-svg/icons/githubcopilot.svg?url";
import gooseIcon from "@lobehub/icons-static-svg/icons/goose.svg?url";
import greptileIcon from "@lobehub/icons-static-svg/icons/greptile-color.svg?url";
import hermesAgentIcon from "@lobehub/icons-static-svg/icons/hermesagent.svg?url";
import junieIcon from "@lobehub/icons-static-svg/icons/junie-color.svg?url";
import kiloCodeIcon from "@lobehub/icons-static-svg/icons/kilocode.svg?url";
import kiroIcon from "@lobehub/icons-static-svg/icons/kiro-color.svg?url";
import manusIcon from "@lobehub/icons-static-svg/icons/manus.svg?url";
import openAiIcon from "@lobehub/icons-static-svg/icons/openai.svg?url";
import openClawIcon from "@lobehub/icons-static-svg/icons/openclaw-color.svg?url";
import openCodeIcon from "@lobehub/icons-static-svg/icons/opencode.svg?url";
import openHandsIcon from "@lobehub/icons-static-svg/icons/openhands-color.svg?url";
import qoderIcon from "@lobehub/icons-static-svg/icons/qoder-color.svg?url";
import qwenCodeIcon from "@lobehub/icons-static-svg/icons/qwen-color.svg?url";
import replitAgentIcon from "@lobehub/icons-static-svg/icons/replit-color.svg?url";
import rooCodeIcon from "@lobehub/icons-static-svg/icons/roocode.svg?url";
import traeIcon from "@lobehub/icons-static-svg/icons/trae-color.svg?url";
import windsurfIcon from "@lobehub/icons-static-svg/icons/windsurf.svg?url";
import zencoderIcon from "@lobehub/icons-static-svg/icons/zencoder-color.svg?url";
import reasonixIcon from "./assets/agent-logos/reasonix.svg?url";

export type AgentSurface = "cli" | "editor" | "desktop" | "service";

export interface AgentCatalogEntry {
  id: string;
  name: string;
  aliases: string[];
  commands: string[];
  homepage: string;
  iconUrl: string;
  surface: AgentSurface;
}

export const agentCatalog: AgentCatalogEntry[] = [
  { id: "codex", name: "Codex", aliases: ["OpenAI Codex", "Codex CLI"], commands: ["codex"], homepage: "https://developers.openai.com/codex/", iconUrl: openAiIcon, surface: "cli" },
  { id: "claude-code", name: "Claude Code", aliases: ["Claude", "Claude CLI"], commands: ["claude"], homepage: "https://code.claude.com/", iconUrl: claudeCodeIcon, surface: "cli" },
  { id: "opencode", name: "OpenCode", aliases: ["Open Code"], commands: ["opencode"], homepage: "https://opencode.ai/", iconUrl: openCodeIcon, surface: "cli" },
  { id: "openclaw", name: "OpenClaw", aliases: ["Open Claw"], commands: ["openclaw"], homepage: "https://openclaw.ai/", iconUrl: openClawIcon, surface: "cli" },
  { id: "reasonix", name: "Reasonix", aliases: ["DeepSeek Reasonix", "Reasonix CLI"], commands: ["reasonix"], homepage: "https://reasonix.io/", iconUrl: reasonixIcon, surface: "cli" },
  { id: "hermes-agent", name: "Hermes Agent", aliases: ["Hermes", "Nous Hermes", "NousResearch Hermes"], commands: ["hermes"], homepage: "https://hermes-agent.nousresearch.com/", iconUrl: hermesAgentIcon, surface: "cli" },
  { id: "deepseek-harness", name: "DeepSeek Harness", aliases: ["DSH", "DeepSeek DSH"], commands: ["dsh"], homepage: "https://deepseek.com/harness", iconUrl: deepSeekIcon, surface: "cli" },
  { id: "gemini-cli", name: "Gemini CLI", aliases: ["Google Gemini CLI"], commands: ["gemini"], homepage: "https://github.com/google-gemini/gemini-cli", iconUrl: geminiCliIcon, surface: "cli" },
  { id: "github-copilot", name: "GitHub Copilot", aliases: ["Copilot CLI", "GitHub Copilot CLI"], commands: ["github-copilot-cli"], homepage: "https://github.com/features/copilot/cli", iconUrl: githubCopilotIcon, surface: "cli" },
  { id: "cursor", name: "Cursor", aliases: ["Cursor Agent"], commands: ["cursor", "cursor-agent"], homepage: "https://cursor.com/", iconUrl: cursorIcon, surface: "editor" },
  { id: "windsurf", name: "Windsurf", aliases: ["Codeium Windsurf"], commands: ["windsurf"], homepage: "https://windsurf.com/", iconUrl: windsurfIcon, surface: "editor" },
  { id: "amp", name: "Amp", aliases: ["Sourcegraph Amp"], commands: ["amp"], homepage: "https://ampcode.com/", iconUrl: ampIcon, surface: "cli" },
  { id: "cline", name: "Cline", aliases: ["Cline Bot"], commands: [], homepage: "https://cline.bot/", iconUrl: clineIcon, surface: "editor" },
  { id: "roo-code", name: "Roo Code", aliases: ["Roo Cline", "RooCode"], commands: [], homepage: "https://roocode.com/", iconUrl: rooCodeIcon, surface: "editor" },
  { id: "kilo-code", name: "Kilo Code", aliases: ["KiloCode"], commands: ["kilocode"], homepage: "https://kilocode.ai/", iconUrl: kiloCodeIcon, surface: "editor" },
  { id: "kiro", name: "Kiro", aliases: ["Kiro CLI", "Kiro IDE"], commands: ["kiro", "kiro-cli"], homepage: "https://kiro.dev/", iconUrl: kiroIcon, surface: "editor" },
  { id: "goose", name: "Goose", aliases: ["Block Goose"], commands: ["goose"], homepage: "https://block.github.io/goose/", iconUrl: gooseIcon, surface: "cli" },
  { id: "qwen-code", name: "Qwen Code", aliases: ["Qwen CLI"], commands: ["qwen"], homepage: "https://github.com/QwenLM/qwen-code", iconUrl: qwenCodeIcon, surface: "cli" },
  { id: "codebuddy", name: "CodeBuddy", aliases: ["Tencent CodeBuddy"], commands: ["codebuddy"], homepage: "https://www.codebuddy.ai/", iconUrl: codeBuddyIcon, surface: "editor" },
  { id: "trae", name: "TRAE", aliases: ["Trae IDE", "Trae Agent"], commands: ["trae"], homepage: "https://www.trae.ai/", iconUrl: traeIcon, surface: "editor" },
  { id: "junie", name: "Junie", aliases: ["JetBrains Junie"], commands: [], homepage: "https://www.jetbrains.com/junie/", iconUrl: junieIcon, surface: "editor" },
  { id: "devin", name: "Devin", aliases: ["Devin AI"], commands: [], homepage: "https://devin.ai/", iconUrl: devinIcon, surface: "service" },
  { id: "qoder", name: "Qoder", aliases: ["Qoder IDE"], commands: ["qoder"], homepage: "https://qoder.com/", iconUrl: qoderIcon, surface: "editor" },
  { id: "openhands", name: "OpenHands", aliases: ["Open Hands"], commands: ["openhands"], homepage: "https://openhands.dev/", iconUrl: openHandsIcon, surface: "desktop" },
  { id: "replit-agent", name: "Replit Agent", aliases: ["Replit AI"], commands: ["replit"], homepage: "https://replit.com/ai", iconUrl: replitAgentIcon, surface: "service" },
  { id: "antigravity", name: "Antigravity", aliases: ["Google Antigravity"], commands: ["antigravity"], homepage: "https://antigravity.google/", iconUrl: antigravityIcon, surface: "editor" },
  { id: "codeflicker", name: "CodeFlicker", aliases: ["Code Flicker"], commands: ["codeflicker"], homepage: "https://www.codeflicker.ai/", iconUrl: codeFlickerIcon, surface: "editor" },
  { id: "greptile", name: "Greptile", aliases: ["Greptile Agent"], commands: [], homepage: "https://www.greptile.com/", iconUrl: greptileIcon, surface: "service" },
  { id: "manus", name: "Manus", aliases: ["Manus Agent"], commands: [], homepage: "https://manus.im/", iconUrl: manusIcon, surface: "service" },
  { id: "zencoder", name: "Zencoder", aliases: ["ZenCoder"], commands: ["zencoder"], homepage: "https://zencoder.ai/", iconUrl: zencoderIcon, surface: "editor" },
];

function normalize(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]/g, "");
}

export function findCatalogAgent(...identifiers: Array<string | undefined>): AgentCatalogEntry | undefined {
  const keys = new Set(identifiers.filter((value): value is string => Boolean(value)).map(normalize));
  return agentCatalog.find((entry) =>
    [entry.id, entry.name, ...entry.aliases, ...entry.commands].some((value) => keys.has(normalize(value))),
  );
}
