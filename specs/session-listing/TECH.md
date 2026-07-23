# Session 列表浏览：技术规格

## Overview

在 TUI 主循环中加载并渲染 session 列表：通过 `AgentBackend::list()` 从当前
过滤条件对应的一个或多个 backend 收集 session，统一排序后交给 ratatui 表格
渲染；键盘事件驱动光标移动、过滤切换与刷新。

## Current Architecture

- `src/main.rs`：入口与事件循环（100ms poll，draw → handle_key → on_tick）。
- `src/app.rs`：`App` 状态机持有 `filter: Option<AgentKind>`（`None` 表示
  `all`）、`sessions`、`table_state`、当前 `Mode` 等。
- `src/ui.rs`：表格渲染（标题 / 目录 / 更新时间列）与状态栏。
- `src/backend/mod.rs`：`backend(kind)` 工厂返回各 backend；`AgentKind::ALL`
  枚举全部 agent。

## Components

### App::reload

- Responsibility: 按当前 filter 调用各 backend 的 `list()`，合并、按
  `updated_ms` 降序排序，更新状态栏摘要。
- Inputs: 当前 `filter`。
- Outputs: `sessions` 列表、状态栏文本。
- Failure behavior: 单个 backend 出错时记录错误文本，继续加载其余 backend；
  状态栏显示 `已加载 N 个 session（<agent>: <原因>; ...）`。

### 键位处理（Normal mode）

- `j`/`k`/`↑`/`↓`：移动一行；`gg`/`G`：首行 / 末行；`Ctrl-d`/`Ctrl-u`：
  半页（10 行）翻页；`r`：刷新；`q`/`Ctrl-c`：退出。
- `<leader>fl`（Space → f → l）：进入 Picker mode。leader 序列有 1500ms
  超时（`LEADER_TIMEOUT`），超时或输入不匹配则清空 pending 序列。

### Picker mode

- 候选项为 `all` 加上 `AgentKind::ALL` 中的各 agent；`j`/`k` 循环移动，
  `Enter` 应用过滤并触发 `reload()`，`Esc`/`q` 取消。

## Data Model

`Session { agent, id, title, cwd, updated_ms, path }`（定义于
`src/backend/mod.rs`），列表数据完全在内存中，无持久化。

## Execution Flow

1. `App::new()` 以 `filter = Some(Codex)` 构造并调用 `reload()`。
2. 事件循环每轮重绘表格；按键进入 `handle_key` 按当前 `Mode` 分发。
3. Picker 确认后更新 `filter` 并 `reload()`，列表替换为新数据。

## Edge Cases

- 空列表：导航直接返回，`table_state` 不选中任何行。
- 刷新后列表变短：光标 clamp 到 `len - 1`。
- leader 序列中途超时不应残留 pending 状态（在按键处理与 `on_tick` 中
  双重清理）。

## Error Handling

backend 的 `list()` 错误按 agent 粒度收集并拼接进状态栏；不弹出、不退出。

## Testing

- Unit tests（`src/app.rs` 内）：`gg`/`G` 导航、`<leader>fl` 打开 picker、
  picker 切换 filter。
- 手动冒烟：`cargo run --example list` 验证各 backend 列出数据。

## Rollout and Rollback

本地工具，随 `cargo build --release` 发布，无迁移与回滚需求。
