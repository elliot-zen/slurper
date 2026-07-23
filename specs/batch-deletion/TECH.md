# Session 批量删除：技术规格

## Overview

在 `App` 状态机中以 `Visual` / `Confirm` / `Deleting` 三种 mode 实现
「选择 → 确认 → 后台删除」流程；删除在独立线程中顺序执行，通过
`mpsc::channel` 向主循环汇报进度。

## Current Architecture

- `src/app.rs`：`Mode` 枚举（`Normal`/`Visual`/`Picker`/`Confirm`/
  `Deleting`）、`DelProgress` 消息、`App::targets()`、`start_delete()`、
  `on_tick()`。
- `src/backend/mod.rs`：`AgentBackend::delete(&Session)`，各 agent 的删除
  实现见 agent-backends 规格。
- `src/ui.rs`：确认对话框与进度条渲染。

## Components

### 选择（Visual mode）

- `V` 记录 `visual_anchor` 并进入 Visual；`j`/`k`/`G`/`gg` 移动光标，
  选中范围为 `[min(anchor, cursor), max(anchor, cursor)]`。
- `targets()` 返回该范围的 `Session` 克隆；非 Visual 时返回当前行单元素。

### 确认（Confirm mode）

- `x` 把 `targets()` 快照存入 `confirm_targets` 并进入 Confirm（避免
  mode 切换后选中范围丢失）。
- `y`/`Enter` 调 `start_delete()`；其他任意键清空 `confirm_targets` 并
  回到 Normal，无副作用。

### 后台删除（Deleting mode）

- `start_delete()` 取出快照，spawn 线程顺序调用
  `backend(session.agent).delete(&session)`，每个完成发送 `DelProgress
  { done, total, id, err }`。
- 主循环 `on_tick()` 用 `try_recv()` 排空消息：更新 `progress`
  (done/total/current id) 与 `deleted` (ok/failed)；单条失败写状态栏。
- `done == total` 时收尾：清空 `rx`/`progress`、回到 Normal、写结果摘要，
  并 `reload_keep_status()` 刷新列表但保留结果文本。
- Deleting 下 `handle_key` 为空操作。

## Execution Flow

1. `V` + 移动 → `x` → Confirm（快照）。
2. `y` → spawn 删除线程，进入 Deleting，进度条随消息推进。
3. 最后一条消息到达 → 状态摘要 + 刷新列表 → Normal。

## Edge Cases

- 空列表按 `x` / `V`：不产生任何效果。
- 快照为空时 `start_delete()` 直接回到 Normal。
- 列表在确认前被刷新不影响已快照的删除集合。

## Error Handling

- `delete()` 返回 `Err` 时记录 `"{agent}: {错误链}"` 到该条进度消息，
  线程继续处理后续目标，不中断。
- 发送进度消息失败（`tx.send` 出错）被忽略——主循环退出时线程安静结束。

## Testing

- Unit tests（`src/app.rs` 内）：
  - visual 选择范围与 `targets()` 数量（含 `G` 扩展到末尾）。
  - `x` 进入 Confirm 且快照包含全部选中行；取消后快照清空。
  - Esc / `q` 退出 visual mode。
- 回归要求：任何改动选择或确认逻辑时必须保持上述测试通过；快照语义
  （Confirm 后选中范围不丢失）已有 `visual_x_snapshots_whole_selection`
  覆盖。

## Rollout and Rollback

本地工具，无迁移；回滚即回退版本。
