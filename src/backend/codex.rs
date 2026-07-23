use std::fs;
use std::process::Command;

use anyhow::{Context, Result, anyhow};

use super::{AgentBackend, AgentKind, Session, clean_title, content_text, home, mtime_ms, read_lines, walk_jsonl};

/// Codex CLI sessions: ~/.codex/sessions/YYYY/MM/DD/rollout-<ts>-<uuid>.jsonl
/// Delete via `codex delete --force <uuid>`, falling back to removing the rollout file.
pub struct Codex;

impl AgentBackend for Codex {
    fn kind(&self) -> AgentKind {
        AgentKind::Codex
    }

    fn list(&self) -> Result<Vec<Session>> {
        let root = home().join(".codex/sessions");
        let mut files = Vec::new();
        walk_jsonl(&root, &mut files);

        let mut out = Vec::new();
        for path in files {
            let lines = read_lines(&path, 300);
            let mut id = String::new();
            let mut cwd = String::new();
            let mut title = String::new();

            for line in &lines {
                let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
                    continue;
                };
                let payload = v.get("payload").cloned().unwrap_or(v.clone());
                // session_meta line carries id + cwd
                if payload.get("session_id").is_some() || payload.get("id").is_some() {
                    if id.is_empty() {
                        id = payload
                            .get("session_id")
                            .or_else(|| payload.get("id"))
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string();
                    }
                    if cwd.is_empty() {
                        cwd = payload
                            .get("cwd")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string();
                    }
                }
                // first real user message as title (skip env-context / AGENTS.md blobs)
                if title.is_empty()
                    && payload.get("role").and_then(|r| r.as_str()) == Some("user")
                    && let Some(text) = payload.get("content").and_then(content_text)
                {
                    let trimmed = text.trim_start();
                    if !trimmed.starts_with('<') && !trimmed.starts_with('#') {
                        title = clean_title(&text);
                    }
                }
                if !id.is_empty() && !title.is_empty() {
                    break;
                }
            }

            // fallback: uuid is the last 36 chars of the rollout filename stem
            if id.is_empty()
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && stem.len() >= 36
            {
                id = stem[stem.len() - 36..].to_string();
            }
            if id.is_empty() {
                continue;
            }

            out.push(Session {
                agent: AgentKind::Codex,
                id,
                title,
                cwd,
                updated_ms: mtime_ms(&path),
                path: Some(path),
            });
        }
        Ok(out)
    }

    fn delete(&self, session: &Session) -> Result<()> {
        let cli = Command::new("codex")
            .args(["delete", "--force", &session.id])
            .output();
        match cli {
            Ok(out) if out.status.success() => Ok(()),
            Ok(out) => {
                // CLI refused (e.g. not a known session); fall back to the file
                if let Some(path) = &session.path {
                    fs::remove_file(path).with_context(|| format!("remove {}", path.display()))?;
                    Ok(())
                } else {
                    Err(anyhow!(
                        "codex delete failed: {}",
                        String::from_utf8_lossy(&out.stderr).trim()
                    ))
                }
            }
            Err(e) => {
                // codex not installed; remove the rollout file directly
                if let Some(path) = &session.path {
                    fs::remove_file(path).with_context(|| format!("remove {}", path.display()))?;
                    Ok(())
                } else {
                    Err(anyhow!("codex not found: {e}"))
                }
            }
        }
    }
}
