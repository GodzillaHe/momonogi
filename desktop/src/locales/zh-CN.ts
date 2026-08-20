import type { en } from "./en";

type TranslationShape<T> = {
  [K in keyof T]: T[K] extends string ? string : TranslationShape<T[K]>;
};

export const zhCN: TranslationShape<typeof en> = {
  translation: {
    common: {
      yes: "是", no: "否", all: "全部", loading: "加载中", refresh: "刷新", refreshing: "正在刷新",
      close: "关闭", dismiss: "收起", open: "打开{{name}}", remove: "移除{{name}}",
      notesCount_one: "{{count}} 条记忆", notesCount_other: "{{count}} 条记忆",
      tagsCount_one: "{{count}} 个标签", tagsCount_other: "{{count}} 个标签",
      storesCount_one: "{{count}} 个仓库", storesCount_other: "{{count}} 个仓库",
    },
    language: { group: "语言", chinese: "切换到中文", english: "切换到英文" },
    nav: { primary: "主导航", agents: "Agent", memories: "记忆", tags: "标签", settings: "设置", storeStatus: "仓库状态", storeReady: "仓库就绪" },
    toolbar: {
      agents: { title: "Agent 权限", context: "全局仓库", search: "搜索 Agent" },
      memories: { title: "记忆浏览器", context: "全部仓库", search: "搜索记忆" },
      tags: { title: "标签", context: "全部仓库", search: "搜索标签" },
      settings: { title: "设置", context: "本地运行环境", search: "搜索设置" },
      clearSearch: "清除搜索",
    },
    role: { writer: "写入", reader: "只读", none: "无权限", noneShort: "无权限", access: "{{role}}权限", group: "{{name}}权限", option: "{{name}}：{{role}}", finalWriter: "至少需要保留一个写入者" },
    hooks: { active: "Hook 已启用", partial: "Hook 不完整", invalid: "Hook 无效", missing: "Hook 未启用", notApplicable: "不适用" },
    config: {
      issue: "配置异常", managed: "已托管", found: "发现配置", notConfigured: "未配置",
      paths: "配置路径", noPaths: "没有主机适配器路径", preview: "预览配置",
      title: "{{role}}配置", dismiss: "关闭配置预览", files: "配置文件",
      noHostPaths: "没有可用的主机配置路径。", current: "配置已是最新",
      filesChange_one: "将修改 {{count}} 个文件", filesChange_other: "将修改 {{count}} 个文件", apply: "应用更改",
      kind: { rules: "规则", hooks: "Hook" },
      action: { create: "创建", update: "更新", removeManaged: "移除 Momonogi", unchanged: "无变化" },
      noAdapter: "此 Agent 没有托管的主机适配器，但仍可配置其清单权限。",
      alreadyCurrent: "主机配置已是最新。",
      synchronized_one: "已同步 {{count}} 个主机配置文件。", synchronized_other: "已同步 {{count}} 个主机配置文件。",
      stale: "主机配置已在其他位置更改，预览已刷新。",
    },
    agents: {
      heading: "权限矩阵", description: "当前仓库清单中的角色", writerIdentity: "写入者身份", chooseWriter: "选择写入者",
      summary: "Agent 概览", detected: "已检测", writers: "写入者", readers: "只读者", installed: "已安装", notInstalled: "未安装",
      scanning: "正在扫描本机 Agent", discoveryFailed: "Agent 检测失败", noMatch: "没有匹配的 Agent", open: "打开{{name}}",
      closeDetails: "关闭 Agent 详情", command: "命令", access: "权限", rules: "规则", hooks: "Hook",
      updateRevision: "权限已更新至修订 {{revision}}。", alreadyCurrent: "权限已是最新。", noManifest: "当前仓库没有可写入的清单。",
      conflict: "仓库已在其他位置更改，当前角色已重新加载。",
    },
    store: {
      active: "当前仓库", scope: "范围", global: "全局", project: "项目", health: "状态", unavailable: "不可用", ready: "就绪", missing: "缺失", invalid: "无效",
      schema: "数据结构", revision: "修订", root: "根目录", openFolder: "打开文件夹",
    },
    memories: {
      heading: "已索引记忆", description: "全局仓库与已注册的项目仓库", filters: "记忆筛选",
      type: "类型", status: "状态", scope: "范围", archive: "归档", typeLabel: "记忆类型", statusLabel: "记忆状态", scopeLabel: "记忆范围", archiveLabel: "归档状态",
      types: { user: "用户", feedback: "反馈", project: "项目", reference: "参考" },
      statuses: { active: "有效", archived: "已归档" }, scopes: { global: "全局", repo: "项目" },
      activeOnly: "仅有效", archivedOnly: "仅归档", unreadable_one: "有 {{count}} 条记忆无法读取。", unreadable_other: "有 {{count}} 条记忆无法读取。",
      list: "记忆", globalGroup: "全局", projectsGroup: "项目", readingStores: "正在读取已注册仓库", noMatch: "没有符合筛选条件的记忆",
      readingDetail: "正在读取完整记忆", select: "选择一条记忆查看详情",
    },
    memoryTags: {
      title: "标签", writerIdentity: "标签写入者身份", chooseWriter: "选择写入者", remove: "移除标签 {{tag}}", removeTitle: "移除 {{tag}}", none: "暂无标签",
      readOnly: "已归档记忆为只读。", new: "新标签", addPlaceholder: "添加标签", adding: "正在添加标签", add: "添加标签",
      chooseBeforeEdit: "更改标签前请选择写入者身份。", added: "标签已添加，当前修订为 {{revision}}。", removed: "标签已移除，当前修订为 {{revision}}。",
      current: "标签已是最新。", conflict: "此记忆已在其他位置更改，当前标签已重新加载。",
    },
    tagIndex: {
      heading: "标签索引", description: "跨已注册仓库标准化整理", table: "标签", tag: "标签", scope: "范围", notes: "记忆数",
      scopes: { global: "全局", project: "项目", mixed: "混合" }, open: "打开{{name}}", noMatch: "没有匹配的标签",
    },
    settings: {
      runtime: "运行环境", description: "桌面桥接与 Momonogi 核心", application: "应用", version: "版本", coreSchema: "核心数据结构", bridge: "桥接模式",
      registry: "仓库注册表", registryDescription: "全局仓库与显式注册的项目仓库", projectPath: "项目仓库路径", register: "注册",
      registered: "已注册仓库", globalStore: "全局仓库", projectStore: "项目仓库", removeRegistry: "从注册表移除",
    },
    bridge: { desktop: "桌面桥接", browser: "开发桥接" },
    mock: {
      "agent-access-policy": { name: "Agent 权限策略", description: "平权写入者与可配置的只读者", body: "Codex 和 Claude Code 是平权写入者。OpenCode 和 OpenClaw 根据当前清单读取共享记忆。\n\n原因：Agent 权限必须清晰明确且可撤销。\n\n使用方式：通过 Momonogi，以当前写入者身份更改角色。" },
      "momonogi-desktop": { name: "Momonogi 桌面应用", description: "共享记忆的桌面管理器", body: "Momonogi Desktop 管理 Agent 权限、已注册仓库、搜索和标签。\n\n原因：本地记忆操作需要一个紧凑的控制界面。\n\n使用方式：CLI 继续作为自动化接口，桌面应用用于查看与管理。" },
      "interface-preferences": { name: "界面偏好", description: "紧凑的工作型界面", body: "工作型工具应保持紧凑、直接，并便于在重复使用时快速浏览。\n\n原因：密集工作流适合稳定、可预测的信息布局。\n\n使用方式：使用行布局、细分隔线和克制的状态提示。" },
      "old-layout": { name: "旧版布局决策", description: "已归档的桌面布局方向", body: "此前的布局方向已经归档。\n\n原因：它不再符合当前的工作台定位。\n\n使用方式：仅保留用于历史参考。" },
    },
  },
};
