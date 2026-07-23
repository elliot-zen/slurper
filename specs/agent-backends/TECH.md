# Agent 存储适配：技术规格

## Overview

以 `AgentBackend` trait 抽象各 agent 的 session 存储：每个 backend 实现
`list()`（发现 session 并提取统一字段）与 `delete()`（按 agent 特定方式
删除）。`backend(kind)` 工厂按 `AgentKind` 分发。新增 agent = 实现 trait
+ 在 `AgentKind::ALL` 与工厂中注册。

## Current Architecture

```
src/backend/
├── mod.rs       # AgentKind / Session / AgentBackend / 共享 helper
├── codex.rs     # 文件遍历 + `codex delete --force`，失败降级删文件
├── claude.rs    # 文件遍历 + 直接删文件
├── kimi.rs      # 目录遍历 + state.json 解析 + 索引维护
├── pi.rs        # 文件遍历 + 直接删文件
└── opencode.rs  # 官方 CLI（session list/delete）
```

### 统一接口（`src/backend/mod.rs`）

```rust
pub trait AgentBackend: Send + Sync {
    fn kind(&self) -> AgentKind;
    fn list(&self) -> Result<Vec<Session>>;
    fn delete(&self, session: &Session) -> Result<()>;
}
```

`Session { agent, id, title, cwd, updated_ms, path }`；`path` 是 backend
私有定位符（文件或目录），作为删除降级路径使用。

### 共享 helper

- `walk_jsonl()` 递归收集 `*.jsonl`；`read_lines(path, max)` 限量读行；
  `mtime_ms()` 取文件修改时间；`clean_title()` 折叠空白并截断 80 字符；
  `content_text()` 从 string 或 `{type,text}[]` 数组中提取文本。

## Components

### codex

- list: 遍历 `~/.codex/sessions/**/*.jsonl`，前 300 行内找
  `session_meta`（`session_id`/`id`、`cwd`）与第一条真实 user 消息作标题
  （跳过 `<`、`#` 开头的环境上下文）；id 缺失时用文件名末尾 36 字符的
  uuid 兜底。
- delete: `codex delete --force <uuid>`；CLI 失败或不存在且有 `path` 时
  降级 `fs::remove_file`。

### claude / pi

- list: 分别遍历 `~/.claude/projects/*/<uuid>.jsonl` 与
  `~/.pi/agent/sessions/*/*.jsonl`，从 JSONL 行提取 id/cwd/首条 user 消息。
- delete: `fs::remove_file(session.path)`。

### kimi

- list: 遍历 `~/.kimi-code/sessions/wd_*/session_*/`，从 `state.json`
  读 `title` 与 `updatedAt`（RFC3339）；cwd 优先查全局索引
  `~/.kimi-code/session_index.jsonl`（`sessionId`→`workDir`），缺失时从
  目录名 `wd_<name>_<hash>` 剥离 hash 兜底。
- delete: `remove_dir_all(session 目录)`，再重写 `session_index.jsonl`
  剔除该 `sessionId` 的行。

### opencode

- list: `opencode session list --format json` 解析 JSON 数组
  （`id`/`title`/`directory`/`updated`）。
- delete: `opencode session delete <id>`，无降级路径。

## Error Handling

- 目录不存在 / 不可读：返回空列表（`read_dir` 失败即返回）。
- 行级 JSON 解析失败：跳过该行。
- CLI 缺失（spawn 失败）：codex 降级删文件；opencode 返回
  `anyhow!("opencode not found: ...")` 错误。
- CLI 非零退出：codex 降级删文件；opencode 把 stderr 包进错误返回。

## Edge Cases

- session 文件存在但无 id 且无文件名兜底：跳过该文件。
- kimi 索引文件不存在：删除时跳过索引重写，仅删目录。
- 标题超过 80 字符：截断并追加 `…`。

## Testing

- 冒烟测试：`cargo run --example list` 逐个 backend 调 `list()`（只读，
  不删除）。
- 解析逻辑目前依赖真实目录结构，暂无 mock；改动某 backend 的解析时应在
  真实环境用 example 验证。

## Rollout and Rollback

无独立发布；随主程序构建。上游 agent 存储格式变更时更新对应 backend 即可，
互不影响。
