use chrono::{DateTime, Local};
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Table};

use crate::app::{App, Mode};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(if app.mode == Mode::Deleting { 3 } else { 1 }),
    ])
    .split(area);

    draw_header(f, app, chunks[0]);
    draw_table(f, app, chunks[1]);
    if app.mode == Mode::Deleting {
        draw_progress(f, app, chunks[2]);
    } else {
        draw_footer(f, app, chunks[2]);
    }

    match app.mode {
        Mode::Picker => draw_picker(f, app),
        Mode::Confirm => draw_confirm(f, app),
        _ => {}
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let mode = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Visual => "VISUAL",
        Mode::Picker => "PICKER",
        Mode::Confirm => "CONFIRM",
        Mode::Deleting => "DELETING",
    };
    let line = Line::from(vec![
        Span::styled(
            format!(" {mode} "),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            "  slurper | agent: {} | {} sessions | {}",
            app.filter_label(),
            app.sessions.len(),
            app.status
        )),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn draw_table(f: &mut Frame, app: &mut App, area: Rect) {
    let show_agent = app.filter.is_none();
    let mut header = vec![Cell::from(""), Cell::from("标题")];
    if show_agent {
        header.push(Cell::from("agent"));
    }
    header.push(Cell::from("目录"));
    header.push(Cell::from("更新时间"));

    let visual = app.visual_range();
    let rows = app.sessions.iter().enumerate().map(|(i, s)| {
        let in_visual = visual.is_some_and(|(a, b)| i >= a && i <= b);
        let marker = if in_visual { "●" } else { "" };
        let mut cells = vec![
            Cell::from(marker),
            Cell::from(s.title.clone()),
        ];
        if show_agent {
            cells.push(Cell::from(s.agent.name()));
        }
        cells.push(Cell::from(s.cwd.clone()));
        cells.push(Cell::from(fmt_time(s.updated_ms)));
        let mut row = Row::new(cells);
        if in_visual {
            row = row.style(Style::default().bg(Color::DarkGray));
        }
        row
    });

    let mut widths = vec![
        Constraint::Length(2),
        Constraint::Percentage(if show_agent { 35 } else { 40 }),
    ];
    if show_agent {
        widths.push(Constraint::Length(10));
    }
    widths.push(Constraint::Percentage(38));
    widths.push(Constraint::Length(14));

    let table = Table::new(rows, widths)
        .header(
            Row::new(header)
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        )
        .row_highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶");
    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let hints = match app.mode {
        Mode::Visual => "j/k 移动  gg/G 首/尾  x 删除所选  V/Esc 退出可视模式",
        _ => "j/k 移动  gg/G 首/尾  V 可视选择  x 删除  <SPC>fl 切换agent  r 刷新  q 退出",
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {hints}"),
            Style::default().fg(Color::DarkGray),
        ))),
        area,
    );
}

fn draw_progress(f: &mut Frame, app: &App, area: Rect) {
    let Some((done, total, id)) = &app.progress else {
        return;
    };
    let ratio = if *total == 0 {
        1.0
    } else {
        *done as f64 / *total as f64
    };
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("删除进度"))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(ratio)
        .label(format!("{done}/{total}  {id}"));
    f.render_widget(gauge, area);
}

fn draw_picker(f: &mut Frame, app: &App) {
    let items = App::picker_items();
    let area = popup_rect(30, items.len() as u16 + 4, f.area());
    f.render_widget(Clear, area);
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|(name, _)| ListItem::new(format!(" {name}")))
        .collect();
    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("选择 agent"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶");
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.picker_cursor));
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_confirm(f: &mut Frame, app: &App) {
    let n = app.confirm_targets.len();
    let area = popup_rect(50, 5, f.area());
    f.render_widget(Clear, area);
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  确认删除 {n} 个 session？"),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  y/Enter 确认    n/Esc 取消",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let p = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("删除确认", Style::default().fg(Color::Red))),
    );
    f.render_widget(p, area);
}

fn popup_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);
    let [v] = vertical.areas(area);
    let [h] = horizontal.areas(v);
    h
}

fn fmt_time(ms: i64) -> String {
    let Some(dt) = DateTime::from_timestamp_millis(ms) else {
        return "-".into();
    };
    let local: DateTime<Local> = dt.into();
    local.format("%m-%d %H:%M").to_string()
}
