# Feature Specifications

| ID | Feature | Description |
| --- | --- | --- |
| session-listing | Session 列表浏览 | 在 TUI 中列出本地 AI coding agent 的历史 session（标题 / 目录 / 更新时间），支持切换 agent 过滤、聚合视图与刷新。 |
| batch-deletion | Session 批量删除 | 通过 vim 风格选择（含 visual mode 多选）与确认对话框，批量删除 session，删除在后台执行并显示实时进度与结果。 |
| agent-backends | Agent 存储适配 | 为各 AI coding agent（codex、claude、kimi、pi、opencode）适配其本地 session 存储格式，提供统一的列出与删除能力。 |
