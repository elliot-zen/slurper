use std::fs;

use anyhow::{Context, Result};

use super::{AgentBackend, AgentKind, Session, clean_title, content_text, home, mtime_ms, read_lines};

/// Claude Code sessions: ~/.claude/projects/<encoded-cwd>/<session-uuid>.jsonl
/// No CLI delete exists, so deletion removes the transcript file.
pub struct Claude;

impl AgentBackend for Claude {
    fn kind(&self) -> AgentKind {
        AgentKind::Claude
    }

    fn list(&self) -> Result<Vec<Session>> {
        let root = home().join(".claude/projects");
        let mut out = Vec::new();
        let Ok(projects) = fs::read_dir(&root) else {
            return Ok(out);
        };

        for project in projects.flatten() {
            let pdir = project.path();
            if !pdir.is_dir() {
                continue;
            }
            let fallback_cwd = project
                .file_name()
                .to_string_lossy()
                .replacen('-', "/", 1)
                .replace('-', "/");
            let Ok(files) = fs::read_dir(&pdir) else {
                continue;
            };
            for file in files.flatten() {
                let path = file.path();
                if path.extension().is_none_or(|e| e != "jsonl") {
                    continue;
                }
                let id = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if id.is_empty() {
                    continue;
                }

                let mut cwd = String::new();
                let mut title = String::new();
                for line in read_lines(&path, 300) {
                    let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else {
                        continue;
                    };
                    if cwd.is_empty()
                        && let Some(c) = v.get("cwd").and_then(|c| c.as_str())
                    {
                        cwd = c.to_string();
                    }
                    if title.is_empty()
                        && v.get("type").and_then(|t| t.as_str()) == Some("user")
                        && !v.get("isMeta").and_then(|m| m.as_bool()).unwrap_or(false)
                        && let Some(text) = v
                            .get("message")
                            .and_then(|m| m.get("content"))
                            .and_then(content_text)
                        && !text.trim_start().starts_with('<')
                    {
                        title = clean_title(&text);
                    }
                    if !cwd.is_empty() && !title.is_empty() {
                        break;
                    }
                }

                out.push(Session {
                    agent: AgentKind::Claude,
                    id,
                    title,
                    cwd: if cwd.is_empty() { fallback_cwd.clone() } else { cwd },
                    updated_ms: mtime_ms(&path),
                    path: Some(path),
                });
            }
        }
        Ok(out)
    }

    fn delete(&self, session: &Session) -> Result<()> {
        let path = session
            .path
            .as_ref()
            .context("claude session has no file path")?;
        fs::remove_file(path).with_context(|| format!("remove {}", path.display()))?;
        // sidecar directory with the same stem (debug output etc.), if any
        if let Some(dir) = path.parent().map(|p| p.join(&session.id))
            && dir.is_dir()
        {
            let _ = fs::remove_dir_all(&dir);
        }
        Ok(())
    }
}
