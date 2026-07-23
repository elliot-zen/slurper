use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::TableState;

use crate::backend::{self, AgentKind, Session};

const LEADER_TIMEOUT: Duration = Duration::from_millis(1500);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Visual,
    Picker,
    Confirm,
    Deleting,
}

pub struct DelProgress {
    pub done: usize,
    pub total: usize,
    pub id: String,
    pub err: Option<String>,
}

pub struct App {
    pub filter: Option<AgentKind>, // None = all agents
    pub sessions: Vec<Session>,
    pub table_state: TableState,
    pub mode: Mode,
    pub visual_anchor: usize,
    pub picker_cursor: usize,
    pending: String,
    pending_at: Option<Instant>,
    pub status: String,
    pub progress: Option<(usize, usize, String)>, // done, total, current id
    pub deleted: (usize, usize),                  // ok, failed
    /// selection snapshot taken when entering Confirm mode
    pub confirm_targets: Vec<Session>,
    rx: Option<Receiver<DelProgress>>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            filter: Some(AgentKind::Codex),
            sessions: Vec::new(),
            table_state: TableState::default(),
            mode: Mode::Normal,
            visual_anchor: 0,
            picker_cursor: 1,
            pending: String::new(),
            pending_at: None,
            status: String::new(),
            progress: None,
            deleted: (0, 0),
            confirm_targets: Vec::new(),
            rx: None,
            should_quit: false,
        };
        app.reload();
        app
    }

    pub fn reload(&mut self) {
        let kinds: Vec<AgentKind> = match self.filter {
            Some(k) => vec![k],
            None => AgentKind::ALL.to_vec(),
        };
        let mut sessions = Vec::new();
        let mut errors = Vec::new();
        for &kind in &kinds {
            match backend::backend(kind).list() {
                Ok(list) => sessions.extend(list),
                Err(e) => errors.push(format!("{}: {e:#}", kind.name())),
            }
        }
        sessions.sort_by(|a, b| b.updated_ms.cmp(&a.updated_ms));
        self.sessions = sessions;
        let count = self.sessions.len();
        let cursor = self.cursor().min(count.saturating_sub(1));
        self.table_state.select((count > 0).then_some(cursor));
        self.status = if errors.is_empty() {
            format!("已加载 {count} 个 session")
        } else {
            format!("已加载 {count} 个 session（{}）", errors.join("; "))
        };
    }

    pub fn cursor(&self) -> usize {
        self.table_state.selected().unwrap_or(0)
    }

    fn move_by(&mut self, delta: i64) {
        let len = self.sessions.len() as i64;
        if len == 0 {
            return;
        }
        let next = (self.cursor() as i64 + delta).clamp(0, len - 1) as usize;
        self.table_state.select(Some(next));
    }

    fn go_top(&mut self) {
        if !self.sessions.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    fn go_bottom(&mut self) {
        if !self.sessions.is_empty() {
            self.table_state.select(Some(self.sessions.len() - 1));
        }
    }

    /// Sessions targeted by the next delete: visual range, or the current row.
    pub fn targets(&self) -> Vec<Session> {
        if self.sessions.is_empty() {
            return Vec::new();
        }
        match self.mode {
            Mode::Visual => {
                let (a, b) = (
                    self.visual_anchor.min(self.cursor()),
                    self.visual_anchor.max(self.cursor()),
                );
                self.sessions[a..=b].to_vec()
            }
            _ => vec![self.sessions[self.cursor()].clone()],
        }
    }

    pub fn visual_range(&self) -> Option<(usize, usize)> {
        (self.mode == Mode::Visual).then(|| {
            (
                self.visual_anchor.min(self.cursor()),
                self.visual_anchor.max(self.cursor()),
            )
        })
    }

    pub fn filter_label(&self) -> String {
        self.filter.map_or("all".into(), |k| k.name().into())
    }

    pub fn picker_items() -> Vec<(&'static str, Option<AgentKind>)> {
        let mut items = vec![("all", None)];
        items.extend(AgentKind::ALL.iter().map(|k| (k.name(), Some(*k))));
        items
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => self.normal_key(key),
            Mode::Visual => self.visual_key(key),
            Mode::Picker => self.picker_key(key),
            Mode::Confirm => self.confirm_key(key),
            Mode::Deleting => {}
        }
    }

    fn clear_pending(&mut self) {
        self.pending.clear();
        self.pending_at = None;
    }

    fn normal_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => self.should_quit = true,
                KeyCode::Char('d') => self.move_by(10),
                KeyCode::Char('u') => self.move_by(-10),
                _ => {}
            }
            return;
        }
        let KeyCode::Char(c) = key.code else {
            match key.code {
                KeyCode::Down => self.move_by(1),
                KeyCode::Up => self.move_by(-1),
                KeyCode::Esc => self.clear_pending(),
                _ => {}
            }
            return;
        };

        // pending sequences: <leader>fl and gg
        if !self.pending.is_empty()
            && self
                .pending_at
                .is_some_and(|t| t.elapsed() > LEADER_TIMEOUT)
        {
            self.clear_pending();
        }
        let seq = format!("{}{c}", self.pending);
        match seq.as_str() {
            " fl" => {
                self.mode = Mode::Picker;
                self.picker_cursor = 0;
                self.clear_pending();
                return;
            }
            "gg" => {
                self.go_top();
                self.clear_pending();
                return;
            }
            " " | " f" | "g" => {
                self.pending = seq;
                self.pending_at = Some(Instant::now());
                return;
            }
            _ if !self.pending.is_empty() => self.clear_pending(),
            _ => {}
        }

        match c {
            'q' => self.should_quit = true,
            'j' => self.move_by(1),
            'k' => self.move_by(-1),
            'G' => self.go_bottom(),
            'V' => {
                if !self.sessions.is_empty() {
                    self.visual_anchor = self.cursor();
                    self.mode = Mode::Visual;
                }
            }
            'x' => {
                if !self.sessions.is_empty() {
                    self.confirm_targets = vec![self.sessions[self.cursor()].clone()];
                    self.mode = Mode::Confirm;
                }
            }
            'r' => self.reload(),
            _ => {}
        }
    }

    fn visual_key(&mut self, key: KeyEvent) {
        let KeyCode::Char(c) = key.code else {
            match key.code {
                KeyCode::Down => self.move_by(1),
                KeyCode::Up => self.move_by(-1),
                KeyCode::Esc => self.mode = Mode::Normal,
                _ => {}
            }
            return;
        };
        // gg support in visual mode
        if !self.pending.is_empty()
            && self
                .pending_at
                .is_some_and(|t| t.elapsed() > LEADER_TIMEOUT)
        {
            self.clear_pending();
        }
        let seq = format!("{}{c}", self.pending);
        match seq.as_str() {
            "gg" => {
                self.go_top();
                self.clear_pending();
                return;
            }
            "g" => {
                self.pending = seq;
                self.pending_at = Some(Instant::now());
                return;
            }
            _ if !self.pending.is_empty() => self.clear_pending(),
            _ => {}
        }

        match c {
            'j' => self.move_by(1),
            'k' => self.move_by(-1),
            'G' => self.go_bottom(),
            'x' => {
                self.confirm_targets = self.targets();
                self.mode = Mode::Confirm;
            }
            'V' | 'q' => self.mode = Mode::Normal,
            _ => {}
        }
    }

    fn picker_key(&mut self, key: KeyEvent) {
        let items = Self::picker_items();
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.picker_cursor = (self.picker_cursor + 1) % items.len();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.picker_cursor = (self.picker_cursor + items.len() - 1) % items.len();
            }
            KeyCode::Enter => {
                self.filter = items[self.picker_cursor].1;
                self.mode = Mode::Normal;
                self.reload();
            }
            KeyCode::Esc | KeyCode::Char('q') => self.mode = Mode::Normal,
            _ => {}
        }
    }

    fn confirm_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => self.start_delete(),
            _ => {
                self.confirm_targets.clear();
                self.mode = Mode::Normal;
            }
        }
    }

    fn start_delete(&mut self) {
        let targets = std::mem::take(&mut self.confirm_targets);
        if targets.is_empty() {
            self.mode = Mode::Normal;
            return;
        }
        let total = targets.len();
        self.deleted = (0, 0);
        self.progress = Some((0, total, String::new()));
        self.mode = Mode::Deleting;

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        thread::spawn(move || {
            for (i, session) in targets.iter().enumerate() {
                let backend = backend::backend(session.agent);
                let res = backend.delete(session);
                let _ = tx.send(DelProgress {
                    done: i + 1,
                    total,
                    id: session.id.clone(),
                    err: res.err().map(|e| format!("{}: {e:#}", backend.kind().name())),
                });
            }
        });
    }

    /// Called on every loop iteration: leader timeout + deletion progress.
    pub fn on_tick(&mut self) {
        if !self.pending.is_empty()
            && self
                .pending_at
                .is_some_and(|t| t.elapsed() > LEADER_TIMEOUT)
        {
            self.clear_pending();
        }

        let mut finished = false;
        if let Some(rx) = &self.rx {
            while let Ok(p) = rx.try_recv() {
                if p.err.is_some() {
                    self.deleted.1 += 1;
                } else {
                    self.deleted.0 += 1;
                }
                self.progress = Some((p.done, p.total, p.id.clone()));
                if let Some(err) = p.err {
                    self.status = format!("删除失败 {}: {err}", p.id);
                }
                if p.done == p.total {
                    finished = true;
                }
            }
        }
        if finished {
            self.rx = None;
            self.progress = None;
            self.mode = Mode::Normal;
            let (ok, failed) = self.deleted;
            self.status = if failed == 0 {
                format!("已删除 {ok} 个 session")
            } else {
                format!("已删除 {ok} 个，失败 {failed} 个")
            };
            self.reload_keep_status();
        }
    }

    fn reload_keep_status(&mut self) {
        let status = self.status.clone();
        self.reload();
        self.status = status;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Session;

    fn fake_sessions(n: usize) -> Vec<Session> {
        (0..n)
            .map(|i| Session {
                agent: AgentKind::Codex,
                id: format!("id-{i}"),
                title: format!("title {i}"),
                cwd: "/tmp".into(),
                updated_ms: 1000 - i as i64,
                path: None,
            })
            .collect()
    }

    fn app_with(n: usize) -> App {
        let mut app = App::new();
        app.sessions = fake_sessions(n);
        app.table_state.select(Some(0));
        app
    }

    fn key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    #[test]
    fn visual_select_and_targets() {
        let mut app = app_with(5);
        app.handle_key(key('V'));
        assert_eq!(app.mode, Mode::Visual);
        app.handle_key(key('j'));
        app.handle_key(key('j'));
        let targets = app.targets();
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0].id, "id-0");
        assert_eq!(targets[2].id, "id-2");
        // G extends to the end
        app.handle_key(key('G'));
        assert_eq!(app.targets().len(), 5);
        // Esc leaves visual mode
        app.handle_key(KeyEvent::from(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn x_confirms_then_cancels() {
        let mut app = app_with(3);
        app.handle_key(key('x'));
        assert_eq!(app.mode, Mode::Confirm);
        assert_eq!(app.confirm_targets.len(), 1);
        app.handle_key(key('n'));
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.confirm_targets.is_empty());
    }

    #[test]
    fn visual_x_snapshots_whole_selection() {
        let mut app = app_with(5);
        app.handle_key(key('V'));
        app.handle_key(key('j'));
        app.handle_key(key('j'));
        app.handle_key(key('x'));
        assert_eq!(app.mode, Mode::Confirm);
        // mode is Confirm now, but the snapshot must keep all 3 selected rows
        assert_eq!(app.confirm_targets.len(), 3);
        assert_eq!(app.confirm_targets[0].id, "id-0");
        assert_eq!(app.confirm_targets[2].id, "id-2");
    }

    #[test]
    fn leader_fl_opens_picker() {
        let mut app = app_with(3);
        app.handle_key(key(' '));
        app.handle_key(key('f'));
        assert_eq!(app.mode, Mode::Normal); // still pending
        app.handle_key(key('l'));
        assert_eq!(app.mode, Mode::Picker);
    }

    #[test]
    fn gg_and_g_navigation() {
        let mut app = app_with(10);
        app.handle_key(key('G'));
        assert_eq!(app.cursor(), 9);
        app.handle_key(key('g'));
        app.handle_key(key('g'));
        assert_eq!(app.cursor(), 0);
    }

    #[test]
    fn picker_switches_filter() {
        let mut app = app_with(3);
        app.handle_key(key(' '));
        app.handle_key(key('f'));
        app.handle_key(key('l'));
        // move to "all" (first item) and confirm
        app.picker_cursor = 0;
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.filter, None);
    }
}
