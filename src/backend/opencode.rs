use std::process::Command;

use anyhow::{Result, anyhow};

use super::{AgentBackend, AgentKind, Session, clean_title};

/// OpenCode sessions live in its sqlite db; use the official CLI for
/// listing (`opencode session list --format json`) and deletion
/// (`opencode session delete <id>`).
pub struct OpenCode;

impl AgentBackend for OpenCode {
    fn kind(&self) -> AgentKind {
        AgentKind::OpenCode
    }

    fn list(&self) -> Result<Vec<Session>> {
        let out = Command::new("opencode")
            .args(["session", "list", "--format", "json"])
            .output()
            .map_err(|e| anyhow!("opencode not found: {e}"))?;
        if !out.status.success() {
            return Err(anyhow!(
                "opencode session list failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        let items: Vec<serde_json::Value> = serde_json::from_slice(&out.stdout)
            .map_err(|e| anyhow!("parse opencode session list: {e}"))?;

        Ok(items
            .into_iter()
            .filter_map(|v| {
                let id = v.get("id")?.as_str()?.to_string();
                Some(Session {
                    agent: AgentKind::OpenCode,
                    id,
                    title: clean_title(v.get("title").and_then(|t| t.as_str()).unwrap_or("")),
                    cwd: v
                        .get("directory")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string(),
                    updated_ms: v.get("updated").and_then(|u| u.as_i64()).unwrap_or(0),
                    path: None,
                })
            })
            .collect())
    }

    fn delete(&self, session: &Session) -> Result<()> {
        let out = Command::new("opencode")
            .args(["session", "delete", &session.id])
            .output()
            .map_err(|e| anyhow!("opencode not found: {e}"))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "opencode session delete failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        }
    }
}
