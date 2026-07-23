pub mod claude;
pub mod codex;
pub mod kimi;
pub mod opencode;
pub mod pi;

use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind {
    Codex,
    Claude,
    Kimi,
    Pi,
    OpenCode,
}

impl AgentKind {
    pub const ALL: [AgentKind; 5] = [
        AgentKind::Codex,
        AgentKind::Claude,
        AgentKind::Kimi,
        AgentKind::Pi,
        AgentKind::OpenCode,
    ];

    pub fn name(self) -> &'static str {
        match self {
            AgentKind::Codex => "codex",
            AgentKind::Claude => "claude",
            AgentKind::Kimi => "kimi",
            AgentKind::Pi => "pi",
            AgentKind::OpenCode => "opencode",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub agent: AgentKind,
    pub id: String,
    pub title: String,
    pub cwd: String,
    /// unix timestamp in milliseconds
    pub updated_ms: i64,
    /// backend-specific locator (file or directory) used as delete fallback
    pub path: Option<PathBuf>,
}

/// Uniform abstraction over agent session stores.
/// To support a new agent: implement this trait and register it in `backend()`.
pub trait AgentBackend: Send + Sync {
    fn kind(&self) -> AgentKind;
    fn list(&self) -> Result<Vec<Session>>;
    fn delete(&self, session: &Session) -> Result<()>;
}

pub fn backend(kind: AgentKind) -> Box<dyn AgentBackend> {
    match kind {
        AgentKind::Codex => Box::new(codex::Codex),
        AgentKind::Claude => Box::new(claude::Claude),
        AgentKind::Kimi => Box::new(kimi::Kimi),
        AgentKind::Pi => Box::new(pi::Pi),
        AgentKind::OpenCode => Box::new(opencode::OpenCode),
    }
}

pub fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_default()
}

// ---------- shared helpers ----------

pub(crate) fn read_lines(path: &Path, max: usize) -> Vec<String> {
    let Ok(f) = File::open(path) else {
        return Vec::new();
    };
    BufReader::new(f)
        .lines()
        .take(max)
        .map_while(|l| l.ok())
        .collect()
}

pub(crate) fn mtime_ms(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub(crate) fn clean_title(s: &str) -> String {
    let collapsed: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut t: String = collapsed.chars().take(80).collect();
    if collapsed.chars().count() > 80 {
        t.push('…');
    }
    t
}

/// Recursively collect *.jsonl files under `dir`.
pub(crate) fn walk_jsonl(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_jsonl(&path, out);
        } else if path.extension().is_some_and(|e| e == "jsonl") {
            out.push(path);
        }
    }
}

/// Extract the text of a content field that is either a plain string
/// or an array of `{ "type": "...", "text": "..." }` items.
pub(crate) fn content_text(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Array(items) => items
            .iter()
            .find_map(|i| i.get("text").and_then(|t| t.as_str()))
            .map(|s| s.to_string()),
        _ => None,
    }
}
