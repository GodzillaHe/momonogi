# Momonogi

Momonogi 是一个供多个 AI Agent 共用的本地文件式记忆系统，发布为单个 Rust
二进制，CLI 名为 `momo`。

- 每个记忆库都可以自由配置 Agent 角色。默认情况下，Codex 与 Claude Code 是
  平权写入者，OpenCode 与 OpenClaw 是只读使用者。
- Markdown 记忆可直接查看、迁移和备份。
- 内核文件锁串行化写入，ETag 阻止旧版本覆盖新修改。
- `MEMORY.md` 只保存短索引，Agent 按需打开相关详情。
- 生命周期 Hook 在压缩上下文前提醒写入者处理需要长期保留的信息。

## 安装

需要 Rust 1.85 或更高版本。

```sh
cargo install --path . --locked --force
momo --version
```

新建全局记忆库：

```sh
momo init ~/.local/share/momonogi/store \
  --store-id global \
  --writer codex \
  --writer claude-code \
  --reader opencode \
  --reader openclaw
```

兼容的旧 Markdown 记忆库可以原地接入：

```sh
momo migrate ~/.local/share/momonogi/store --agent codex
```

迁移不会修改记忆正文，只会补齐缺少的多写入者元数据并重新生成索引。

## 管理权限

先查看当前角色和权限清单 ETag：

```sh
momo access list ~/.local/share/momonogi/store --json
```

当前任意 writer 都可以授予、更改或撤销 Agent 角色。修改必须携带最新的清单
ETag，避免两个 writer 静默覆盖彼此的权限调整：

```sh
momo access grant ROOT opencode --role writer --by codex --if-match ETAG
momo access set ROOT openclaw --role reader --by codex --if-match ETAG
momo access revoke ROOT openclaw --by codex --if-match ETAG
```

`set` 是 `grant` 的别名。Momonogi 会拒绝非 writer 操作、过期 ETag、非法或重复的
Agent ID，以及导致记忆库没有 writer 的修改。重复设置相同角色不会改变 revision
或 ETag。

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

`configure` 只管理宿主规则文件中的 Momonogi 标记块。Claude Hook 全局写入
`~/.claude/settings.json`；Codex Hook 是项目级配置，因此每个需要 Hook 的仓库都要传
`--codex-project`。`--no-hooks` 可以只安装规则。其他规则和 Hook 处理器会被保留。

`configure` 会读取权限清单，不再假设宿主角色固定。writer 获得写入规则和受管理的
生命周期 Hook；reader 获得只读规则，同时移除 Momonogi 自己的 Hook；不在清单中的
Agent 获得 no-access 规则。权限修改后需要为受影响的宿主重新运行 `configure`。

OpenClaw 工作区规则会按当前宿主生效，避免共享的项目级 `AGENTS.md` 把 Codex 从
写入者错误降级为只读者。

## 命令

| 命令 | 用途 |
| --- | --- |
| `momo init ROOT ...` | 创建记忆库和角色清单 |
| `momo migrate ROOT --agent ID` | 接入兼容的已有记忆库 |
| `momo list [ROOT]` | 列出全部有效记忆的元数据；默认使用全局库 |
| `momo list --json` | 只输出元数据，不输出记忆正文 |
| `momo get ROOT SLUG.md` | 获取当前 ETag |
| `momo get ROOT SLUG.md --content` | 读取一条记忆 |
| `momo put ROOT FILE --agent ID` | 新增记忆 |
| `momo put ... --if-match ETAG` | 在不覆盖并发修改的前提下更新 |
| `momo archive ROOT SLUG.md --agent ID --if-match ETAG` | 归档记忆 |
| `momo access list [ROOT] [--json]` | 查看角色、清单 revision 和 ETag |
| `momo access grant ROOT ID --role ROLE --by WRITER --if-match ETAG` | 授予或调整角色，`set` 是别名 |
| `momo access revoke ROOT ID --by WRITER --if-match ETAG` | 从清单移除 Agent |
| `momo reindex ROOT --agent ID` | 重新生成 `MEMORY.md` |
| `momo doctor ROOT` | 检查清单、记忆、索引与大小限制 |
| `momo configure ...` | 安装 Agent 规则与生命周期 Hook |
| `momo hook ...` | 生命周期 Hook 入口 |
| `momo sync status ROOT` | 查看会话同步状态 |
| `momo sync mark ROOT --session-id ID` | 标记会话已完成同步 |
| `momo logo` | 显示 Momonogi Logo |

完整参数请运行 `momo COMMAND --help`。

面向 Agent 的安装与信任检查见
[docs/AGENT_SETUP.md](docs/AGENT_SETUP.md)，可复用的 Agent 协议见
[skill/SKILL.md](skill/SKILL.md)。

## 桌面应用

可选的 Tauri 桌面工作台可以发现本机 Agent 宿主、管理记忆库角色、在应用前预览
规则与 Hook 变更，并浏览全局及已注册项目中的记忆与标签。应用包内包含匹配版本的
`momo` sidecar，供生命周期 Hook 使用。

构建和部署说明见 [docs/DESKTOP_DEPLOYMENT.md](docs/DESKTOP_DEPLOYMENT.md)。

## 写入协议

先在记忆库外创建草稿，再通过 `momo` 写入：

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

更新前先获取 ETag，再通过 `--if-match` 提交。不要直接编辑规范库中的记忆文件或
`MEMORY.md`。

## 部署到另一台电脑

1. 克隆 Momonogi 仓库，用 Cargo 安装 Rust 二进制。
2. 单独复制或安全同步记忆库。记忆可能含有个人信息，不应提交到公开仓库。
3. 配置前先运行 `momo doctor ROOT`。
4. 根据另一台电脑上的 Agent 运行 `momo configure`。

记忆库协议只依赖 `.momonogi.json`、`MEMORY.md`、Markdown 记忆文件以及可选的
`archive/` 目录。安装完成后不依赖源码目录。

## 开发验证

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Momonogi 使用 MIT 许可证。第三方归属信息见
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)，维护规范见 [docs/](docs/)。
