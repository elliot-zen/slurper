# Agent 存储适配

## Purpose

不同 AI coding agent 把 session 存在完全不同的位置与格式（JSONL 文件、
目录加索引、sqlite 数据库）。本功能把这些差异收敛为统一的「列出 session」
与「删除 session」两个能力，使列表浏览与批量删除对任意已支持的 agent
行为一致。

## Users

- 直接用户：使用 slurper 管理多个 agent session 的开发者。
- 间接用户：为 slurper 新增 agent 支持的贡献者（通过统一的扩展点接入）。

## Behavior

当前支持的 agent 及其对用户可见的删除效果：

| agent | session 存储 | 删除效果 |
| --- | --- | --- |
| codex | `~/.codex/sessions/**/*.jsonl` | 优先调用 `codex delete --force <uuid>`；CLI 不可用或拒绝时直接删除对应 rollout 文件 |
| claude | `~/.claude/projects/*/<uuid>.jsonl` | 删除该 session 文件（无官方 CLI 删除） |
| kimi | `~/.kimi-code/sessions/*/session_*/` | 删除 session 目录，并从全局索引 `session_index.jsonl` 中移除对应条目 |
| pi | `~/.pi/agent/sessions/*/*.jsonl` | 删除该 session 文件（无官方 CLI 删除） |
| opencode | sqlite 数据库 | 通过官方 CLI `opencode session list/delete` 列出与删除 |

- 每个 agent 的 session 都以统一形态呈现：标题、工作目录、更新时间。
- 某 agent 未安装或其存储目录不存在时，该 agent 的列表为空而不是报错
  （依赖 CLI 的 agent 除外：CLI 缺失会以该 agent 的错误形式上报）。

## Happy Path

1. 列表功能向某 agent 请求 session，获得统一字段的列表。
2. 删除功能对某 session 发起删除，该 agent 以表格中对应的方式完成删除。
3. 用户在列表刷新后看到 session 已消失，且 agent 自身的存储（含索引）
   保持一致。

## Boundaries

### Included

- 五种 agent 的 session 发现、字段提取（标题 / 目录 / 更新时间）与删除。
- 删除后 agent 存储的一致性维护（如 kimi 的索引条目清理）。
- 新 agent 的统一接入点。

### Excluded

- session 内容的展示、搜索或编辑。
- 各 agent 之外工具的适配。
- 对 agent 存储格式的写入（除删除及其索引维护外）。

## Rules

- 优先使用 agent 官方 CLI 删除；官方 CLI 不存在时采用文件 / 目录删除，
  并同步维护该 agent 自有的索引文件。
- 列出与删除只触碰对应 agent 自己的存储目录。
- 删除是幂等目标：目标已不存在视为可接受，不因「已删除」而报错升级。

## Failure Behavior

- 存储目录不存在：该 agent 列表为空。
- 单个 session 文件损坏或字段缺失：跳过或降级（如标题为空），不影响
  其他 session。
- CLI 缺失或执行失败：codex 降级为文件删除；opencode 无降级路径，作为
  该 agent 的错误上报。

## Compatibility

各 agent 的存储格式由其上游产品定义，可能随上游版本变化；适配以保持对
当前格式的兼容为目标，上游格式变更时需要同步更新对应适配。
