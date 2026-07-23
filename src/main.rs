use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;
use slurper::{app, ui};

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut app = app::App::new();
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key);
        }
        app.on_tick();
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
