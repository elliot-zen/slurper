use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Result};

use super::{AgentBackend, AgentKind, Session, clean_title, home, mtime_ms, read_lines};

/// Kimi Code sessions: ~/.kimi-code/sessions/wd_<name>_<hash>/session_<uuid>/
/// with a global index at ~/.kimi-code/session_index.jsonl.
/// No CLI delete exists: remove the session dir and drop its index entry.
pub struct Kimi;

fn index_path() -> std::path::PathBuf {
    home().join(".kimi-code/session_index.jsonl")
}

fn workdir_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in read_lines(&index_path(), usize::MAX) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line)
            && let (Some(id), Some(wd)) = (
                v.get("sessionId").and_then(|x| x.as_str()),
                v.get("workDir").and_then(|x| x.as_str()),
            )
        {
            map.insert(id.to_string(), wd.to_string());
        }
    }
    map
}

impl AgentBackend for Kimi {
    fn kind(&self) -> AgentKind {
        AgentKind::Kimi
    }

    fn list(&self) -> Result<Vec<Session>> {
        let root = home().join(".kimi-code/sessions");
        let workdirs = workdir_map();
        let mut out = Vec::new();
        let Ok(wdirs) = fs::read_dir(&root) else {
            return Ok(out);
        };

        for wdir in wdirs.flatten() {
            let wpath = wdir.path();
            if !wpath.is_dir() {
                continue;
            }
            let Ok(sdirs) = fs::read_dir(&wpath) else {
                continue;
            };
            for sdir in sdirs.flatten() {
                let spath = sdir.path();
                if !spath.is_dir() {
                    continue;
                }
                let id = sdir.file_name().to_string_lossy().into_owned();
                if !id.starts_with("session_") {
                    continue;
                }

                let state_path = spath.join("state.json");
                let mut title = String::new();
                let mut updated_ms = mtime_ms(&spath);
                if let Ok(raw) = fs::read_to_string(&state_path)
                    && let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw)
                {
                    if let Some(t) = v.get("title").and_then(|t| t.as_str()) {
                        title = clean_title(t);
                    }
                    if let Some(u) = v.get("updatedAt").and_then(|u| u.as_str())
                        && let Ok(ts) = chrono::DateTime::parse_from_rfc3339(u)
                    {
                        updated_ms = ts.timestamp_millis();
                    }
                }

                let cwd = workdirs.get(&id).cloned().unwrap_or_else(|| {
                    // fallback: strip the trailing _<hash> from wd_<name>_<hash>
                    let name = wdir.file_name().to_string_lossy().into_owned();
                    name.rsplit_once('_')
                        .map(|(n, _)| n.trim_start_matches("wd_").to_string())
                        .unwrap_or(name)
                });

                out.push(Session {
                    agent: AgentKind::Kimi,
                    id,
                    title,
                    cwd,
                    updated_ms,
                    path: Some(spath),
                });
            }
        }
        Ok(out)
    }

    fn delete(&self, session: &Session) -> Result<()> {
        let dir = session
            .path
            .as_ref()
            .context("kimi session has no directory path")?;
        if dir.is_dir() {
            fs::remove_dir_all(dir).with_context(|| format!("remove {}", dir.display()))?;
        }
        // drop the entry from the global session index
        let idx = index_path();
        let kept: Vec<String> = read_lines(&idx, usize::MAX)
            .into_iter()
            .filter(|line| {
                serde_json::from_str::<serde_json::Value>(line)
                    .ok()
                    .and_then(|v| v.get("sessionId").and_then(|x| x.as_str()).map(String::from))
                    .is_none_or(|sid| sid != session.id)
            })
            .collect();
        if idx.exists() {
            let mut content = kept.join("\n");
            if !content.is_empty() {
                content.push('\n');
            }
            fs::write(&idx, content).with_context(|| format!("rewrite {}", idx.display()))?;
        }
        Ok(())
    }
}
