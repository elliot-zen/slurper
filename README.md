# slurper

A ratatui-based TUI for managing local sessions of AI coding agents, focused on **batch deletion**.

Supported agents: codex, claude code, kimi, pi, opencode.

## Features

- Shows codex sessions by default (title / directory / updated time, newest first)
- `<leader>fl` (Space → f → l) to switch agents, including an `all` aggregated view
- Vim-like keybindings with visual-mode multi-select for batch deletion
- Deletion runs on a background thread with a live progress bar
- Confirmation dialog before deleting, to prevent accidents

## Keybindings

| Key | Action |
| --- | --- |
| `j` / `k` / `↑` / `↓` | Move up / down |
| `gg` / `G` | Jump to first / last row |
| `Ctrl-d` / `Ctrl-u` | Half-page down / up |
| `V` | Enter visual mode; `j/k/G/gg` extends the selection |
| `x` | Delete current row / visual selection (confirmation dialog, `y` to confirm) |
| `Space f l` | Open the agent picker (`j/k` + `Enter`) |
| `r` | Refresh the list |
| `q` / `Ctrl-c` | Quit |

## Build & Run

```sh
cargo build --release
./target/release/slurper
```

Smoke-test all backends (lists only, never deletes):

```sh
cargo run --example list
```

Run unit tests:

```sh
cargo test
```

## Session storage & deletion per agent

| agent | Storage location | Deletion method |
| --- | --- | --- |
| codex | `~/.codex/sessions/**/*.jsonl` | `codex delete --force <uuid>`, falls back to removing the file |
| claude | `~/.claude/projects/*/<uuid>.jsonl` | Removes the file (no CLI delete exists) |
| kimi | `~/.kimi-code/sessions/*/session_*/` | Removes the directory and drops its entry from `session_index.jsonl` |
| pi | `~/.pi/agent/sessions/*/*.jsonl` | Removes the file (no CLI delete exists) |
| opencode | sqlite (`~/.local/share/opencode/opencode.db`) | Official CLI: `opencode session list/delete` |

## Adding a new agent

Implement the unified trait in `src/backend/mod.rs` and register it in `backend()`:

```rust
pub trait AgentBackend: Send + Sync {
    fn kind(&self) -> AgentKind;
    fn list(&self) -> Result<Vec<Session>>;
    fn delete(&self, session: &Session) -> Result<()>;
}
```

## Project layout

```
src/
├── main.rs          # entry point + event loop
├── app.rs           # state machine and key handling
├── ui.rs            # ratatui rendering
└── backend/
    ├── mod.rs       # AgentKind / Session / AgentBackend trait
    ├── codex.rs
    ├── claude.rs
    ├── kimi.rs
    ├── pi.rs
    └── opencode.rs
examples/list.rs     # backend smoke test
```
