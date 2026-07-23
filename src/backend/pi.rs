use std::fs;

use anyhow::{Context, Result};

use super::{AgentBackend, AgentKind, Session, clean_title, content_text, home, mtime_ms, read_lines};

/// Pi (pi-mono) sessions: ~/.pi/agent/sessions/<encoded-cwd>/<ts>_<uuid>.jsonl
/// No CLI delete exists, so deletion removes the session file.
pub struct Pi;

impl AgentBackend for Pi {
    fn kind(&self) -> AgentKind {
        AgentKind::Pi
    }

    fn list(&self) -> Result<Vec<Session>> {
        let root = home().join(".pi/agent/sessions");
        let mut out = Vec::new();
        let Ok(groups) = fs::read_dir(&root) else {
            return Ok(out);
        };

        for group in groups.flatten() {
            let gdir = group.path();
            if !gdir.is_dir() {
                continue;
            }
            let Ok(files) = fs::read_dir(&gdir) else {
                continue;
            };
            for file in files.flatten() {
                let path = file.path();
                if path.extension().is_none_or(|e| e != "jsonl") {
                    continue;
                }

                let mut id = String::new();
                let mut cwd = String::new();
                let mut title = String::new();
                for line in read_lines(&path, 300) {
                    let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else {
                        continue;
                    };
                    match v.get("type").and_then(|t| t.as_str()) {
                        Some("session") => {
                            id = v
                                .get("id")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                            cwd = v
                                .get("cwd")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                        }
                        Some("message") if title.is_empty() => {
                            if let Some(msg) = v.get("message")
                                && msg.get("role").and_then(|r| r.as_str()) == Some("user")
                                && let Some(text) = msg.get("content").and_then(content_text)
                                && !text.trim_start().starts_with('<')
                            {
                                title = clean_title(&text);
                            }
                        }
                        _ => {}
                    }
                    if !id.is_empty() && !title.is_empty() {
                        break;
                    }
                }

                if id.is_empty() {
                    id = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                }
                if id.is_empty() {
                    continue;
                }

                out.push(Session {
                    agent: AgentKind::Pi,
                    id,
                    title,
                    cwd,
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
            .context("pi session has no file path")?;
        fs::remove_file(path).with_context(|| format!("remove {}", path.display()))?;
        Ok(())
    }
}
