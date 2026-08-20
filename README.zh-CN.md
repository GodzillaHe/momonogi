# Momonogi

[English](README.md) | [简体中文](README.zh-CN.md)

Momonogi 是一个面向多个 AI Agent、基于本地文件的共享记忆系统。项目包含名为
`momo` 的 Rust CLI，以及可选的 macOS 桌面应用。

- 每个记忆仓库都能单独配置 Agent 权限。默认情况下，Codex 和 Claude Code
  拥有平等写入权限，OpenCode 和 OpenClaw 只读。
- 记忆使用 Markdown 保存，便于迁移和人工检查。
- 内核文件锁负责串行写入，ETag 用于拒绝过期更新。
- `MEMORY.md` 只保留精简索引，Agent 仅在相关时读取详细记忆。
- 生命周期 Hook 会在上下文压缩前提醒写入者整理需要长期保留的内容。

## 安装 CLI

需要 Rust 1.85 或更高版本。

```sh
cargo install --path . --locked --force
momo --version
```

创建全局记忆仓库：

```sh
momo init ~/.local/share/momonogi/store \
  --store-id global \
  --writer codex \
  --writer claude-code \
  --reader opencode \
  --reader openclaw
```

已有的兼容 Markdown 记忆仓库可以原地迁移：

```sh
momo migrate ~/.local/share/momonogi/store --agent codex
```

迁移会保留记忆正文，为缺少元数据的记忆补充多写入者信息，并重新生成索引。

## 管理权限

查看当前角色和清单 ETag：

```sh
momo access list ~/.local/share/momonogi/store --json
```

当前写入者可以授予、修改或撤销任意 Agent 的角色。每次修改都需要传入当前清单
ETag，因此两个写入者无法在无提示的情况下覆盖对方的权限变更：

```sh
momo access grant ROOT opencode --role writer --by codex --if-match ETAG
momo access set ROOT openclaw --role reader --by codex --if-match ETAG
momo access revoke ROOT openclaw --by codex --if-match ETAG
```

`set` 是 `grant` 的别名。Momonogi 会拒绝无权限的操作者、过期 ETag、无效或重复的
Agent ID，以及导致仓库失去最后一个写入者的修改。未改变角色的操作不会更新 revision
和 ETag。

## 配置 Agent

```sh
momo configure \
  --host codex \
  --host claude \
  --host opencode \
  --host openclaw \
  --codex-project /path/to/project \
  --openclaw-workspace /path/to/openclaw/workspace
```

`configure` 负责维护各 Agent 常规规则文件中的 Momonogi 标记区块。Claude Hook
保存在全局 `~/.claude/settings.json`。Codex Hook 按项目配置，因此需要为每个仓库传入
`--codex-project`。使用 `--no-hooks` 可以只安装规则。Momonogi 会保留与自身无关的
规则和 Hook 处理器。

`configure` 从角色清单读取权限，不会假定固定的 Agent 角色。写入者会获得写入规则和
生命周期 Hook；只读者会获得只读规则，同时移除 Momonogi 管理的 Hook；清单中不存在的
Agent 会获得禁止访问规则。修改权限后，需要为受影响的 Agent 重新运行 `configure`。

OpenClaw 工作区规则会根据当前宿主生效，因此共享的项目级 `AGENTS.md` 不会把 Codex
从写入者降为只读者。

## CLI 命令

| 命令 | 用途 |
| --- | --- |
| `momo init ROOT ...` | 创建记忆仓库和角色清单 |
| `momo migrate ROOT --agent ID` | 接管兼容的现有记忆仓库 |
| `momo list [ROOT]` | 列出全部有效记忆元数据，默认读取全局仓库 |
| `momo list --json` | 只输出元数据，不输出记忆正文 |
| `momo get ROOT SLUG.md` | 返回当前 ETag |
| `momo get ROOT SLUG.md --content` | 读取一条记忆 |
| `momo put ROOT FILE --agent ID` | 新增记忆 |
| `momo put ... --if-match ETAG` | 更新记忆，并阻止并发覆盖 |
| `momo archive ROOT SLUG.md --agent ID --if-match ETAG` | 归档记忆 |
| `momo access list [ROOT] [--json]` | 查看角色、清单 revision 和 ETag |
| `momo access grant ROOT ID --role ROLE --by WRITER --if-match ETAG` | 授予或修改角色，`set` 是其别名 |
| `momo access revoke ROOT ID --by WRITER --if-match ETAG` | 从清单中移除 Agent |
| `momo reindex ROOT --agent ID` | 重新生成 `MEMORY.md` |
| `momo doctor ROOT` | 检查清单、记忆、索引和限制 |
| `momo configure ...` | 安装宿主规则和生命周期 Hook |
| `momo hook ...` | 生命周期 Hook 入口 |
| `momo sync status ROOT` | 查看生命周期同步状态 |
| `momo sync mark ROOT --session-id ID` | 标记会话已完成整理 |
| `momo logo` | 输出 Momonogi Logo |

运行 `momo COMMAND --help` 可以查看完整参数。

Agent 安装和信任检查见 [docs/AGENT_SETUP.md](docs/AGENT_SETUP.md)。可复用的 Agent
协议位于 [skill/SKILL.md](skill/SKILL.md)。

## 桌面应用

Momonogi Desktop 可以发现本机 Agent、管理仓库角色、在应用规则和 Hook 变更前预览
差异，并浏览全局及已登记项目中的记忆和标签。应用内置匹配版本的 `momo` sidecar，供
生命周期 Hook 调用。

当前桌面预发布版本为 `0.0.1-alpha.2`，支持 Apple Silicon Mac。打开
[GitHub Releases](https://github.com/GodzillaHe/momonogi/releases)，下载：

- `Momonogi_0.0.1-alpha.2_aarch64.dmg`
- `Momonogi_0.0.1-alpha.2_aarch64.dmg.sha256`

打开 DMG 前先验证下载文件：

```sh
cd ~/Downloads
shasum -a 256 -c Momonogi_0.0.1-alpha.2_aarch64.dmg.sha256
```

打开 DMG，将 `Momonogi.app` 移入 `/Applications`。应用使用 ad-hoc 签名保证包内
文件完整，但没有 Apple Developer ID 签名，也未经过公证。macOS 首次启动时可能要求
右键应用并选择“打开”。校验通过后，如果 Gatekeeper 仍提示应用已损坏，可以移除已安装
副本的隔离属性：

```sh
xattr -dr com.apple.quarantine /Applications/Momonogi.app
open /Applications/Momonogi.app
```

源码构建和发布流程见
[docs/DESKTOP_DEPLOYMENT.md](docs/DESKTOP_DEPLOYMENT.md)。

## 写入协议

先在正式记忆仓库外起草文件，再通过 `momo` 写入：

```markdown
---
name: Prefer concise progress updates
description: Keep intermediate updates short and specific
type: feedback
scope: global
created: 2026-08-20
updated: 2026-08-20
---

Keep progress updates concise.

Why: long updates interrupt the workflow.

How to apply: report the result, current risk, and next action in two sentences.
```

```sh
momo put ~/.local/share/momonogi/store /tmp/concise-updates.md --agent codex
```

更新记忆前先读取 ETag，并通过 `--if-match` 传入。不要直接编辑正式记忆文件或
`MEMORY.md`。

## 迁移到其他电脑

1. 克隆 Momonogi 仓库，通过 Cargo 安装 Rust CLI。
2. 单独复制或安全同步记忆仓库。记忆可能包含个人信息，不要将其提交到公共仓库。
3. 配置前运行 `momo doctor ROOT`。
4. 为目标电脑上可用的 Agent 运行 `momo configure`。

记忆仓库由 `.momonogi.json`、`MEMORY.md`、Markdown 记忆文件和可选的 `archive/`
目录组成。安装完成后，记忆仓库不依赖 Momonogi 源码目录。

## 开发

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Momonogi 使用 MIT 许可证。第三方依赖声明见
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)，维护策略见 [docs/](docs/)。
