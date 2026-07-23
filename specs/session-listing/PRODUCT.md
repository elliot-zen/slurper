# Session 列表浏览

## Purpose

用户在本地使用多个 AI coding agent（codex、claude code、kimi、pi、opencode），
历史 session 散落在各自的存储目录中，缺乏统一的浏览入口。本功能提供一个
终端界面，把这些 session 汇总成一张列表，帮助用户快速定位想要处理的
session。

## Users

在本地终端中使用上述一个或多个 AI coding agent、需要管理历史 session 的
开发者。

## Behavior

- 启动后默认显示 codex 的 session 列表。
- 列表每行展示：session 标题、所在工作目录、最后更新时间；按更新时间
  从新到旧排序。
- 用户可通过 agent 选择器切换查看单个 agent 的 session，或切换到 `all`
  聚合视图查看全部 agent 的 session。
- 列表支持 vim 风格导航：上下移动、跳到首行 / 末行、半页翻页。
- 用户可随时手动刷新列表，重新从各 agent 的存储中读取最新数据。
- 底部状态栏显示加载结果摘要（如已加载的 session 数量），某个 agent
  读取失败时在状态栏中指出是哪个 agent 及其原因。

## Happy Path

1. 用户启动 slurper，看到 codex 的 session 列表。
2. 用户用导航键浏览列表，或用 agent 选择器切换到 `all` 视图。
3. 列表重新加载，用户找到目标 session。

## Boundaries

### Included

- 列出并排序各 agent 的本地 session。
- 单个 agent 过滤与 `all` 聚合视图的切换。
- 列表导航（移动、首尾跳转、半页翻页）与手动刷新。
- 加载结果与单个 agent 加载失败的状态提示。

### Excluded

- 删除 session（属于 batch-deletion）。
- 各 agent 存储格式的解析细节（属于 agent-backends）。
- 搜索、过滤关键字、按目录分组等高级浏览能力（当前不支持）。
- 恢复或打开某个 session 继续对话。

## Rules

- 默认过滤为 codex；`all` 视图聚合全部已支持的 agent。
- 排序规则固定为更新时间降序，用户不可更改。
- 刷新会重新读取存储并以最新数据替换整个列表。
- 列表为空时导航键不产生任何效果。
- 刷新后光标位置收敛到合法范围内（不超过列表末尾）。

## Failure Behavior

- 某个 agent 读取失败不阻断整体加载：其余 agent 的 session 正常显示，
  状态栏注明失败的 agent 与原因。
- 所有 agent 均无 session 时显示空列表，不报错退出。

## Compatibility

面向终端交互使用，无对外 API 或持久化数据；不改变各 agent 的存储内容
（只读）。
