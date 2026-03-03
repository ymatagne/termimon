//! Interactive ratatui dashboard — full TUI with live creature display
//!
//! Shows tracked creatures with pixel art, XP bars, and live state updates.
//! Refreshes every 2 seconds by querying the daemon via IPC.

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::*,
};
use std::io::stdout;
use std::time::{Duration, Instant};

use crate::creatures::registry;
use crate::creatures::sprites;
use crate::daemon::server::{self, StatusResponse};
use crate::render::halfblock;

/// Refresh interval for daemon polling.
const REFRESH_INTERVAL: Duration = Duration::from_secs(2);

/// Dashboard application state.
struct DashApp {
    status: Option<StatusResponse>,
    selected: usize,
    last_refresh: Instant,
    error_msg: Option<String>,
}

impl DashApp {
    fn new() -> Self {
        Self {
            status: None,
            selected: 0,
            last_refresh: Instant::now() - REFRESH_INTERVAL, // force immediate refresh
            error_msg: None,
        }
    }

    async fn refresh(&mut self) {
        match server::client_request("status").await {
            Ok(response) => {
                match serde_json::from_str::<StatusResponse>(response.trim()) {
                    Ok(status) => {
                        // Clamp selection
                        if !status.agents.is_empty() && self.selected >= status.agents.len() {
                            self.selected = status.agents.len() - 1;
                        }
                        self.status = Some(status);
                        self.error_msg = None;
                    }
                    Err(e) => {
                        self.error_msg = Some(format!("Parse error: {e}"));
                    }
                }
            }
            Err(e) => {
                self.status = None;
                self.error_msg = Some(format!("Cannot connect to daemon: {e}"));
            }
        }
        self.last_refresh = Instant::now();
    }
}

pub async fn run() -> Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = DashApp::new();
    app.refresh().await;

    loop {
        // Draw
        terminal.draw(|frame| draw_dashboard(frame, &app))?;

        // Handle input (non-blocking, 100ms poll)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Up | KeyCode::Char('k') => {
                            if app.selected > 0 {
                                app.selected -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if let Some(ref status) = app.status {
                                if app.selected + 1 < status.agents.len() {
                                    app.selected += 1;
                                }
                            }
                        }
                        KeyCode::Char('r') => {
                            app.refresh().await;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Auto-refresh
        if app.last_refresh.elapsed() >= REFRESH_INTERVAL {
            app.refresh().await;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn draw_dashboard(frame: &mut Frame, app: &DashApp) {
    let area = frame.area();

    // Main layout: header, body, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(10),   // body
            Constraint::Length(3), // footer
        ])
        .split(area);

    // ── Header ───────────────────────────────────────────────────────────
    let header_text = format!("🎮 TermiMon Dashboard v{}", env!("CARGO_PKG_VERSION"));
    let status_info = if let Some(ref status) = app.status {
        let uptime = if let Ok(started) = chrono::DateTime::parse_from_rfc3339(&status.started_at) {
            let elapsed = chrono::Utc::now().signed_duration_since(started);
            let secs = elapsed.num_seconds();
            if secs >= 3600 {
                format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
            } else if secs >= 60 {
                format!("{}m{}s", secs / 60, secs % 60)
            } else {
                format!("{}s", secs)
            }
        } else {
            "??".to_string()
        };
        format!(" | ⏱ {} | 💓 {} | {} agents", uptime, status.heartbeat_count, status.agents.len())
    } else {
        " | ⚠ daemon not connected".to_string()
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(&header_text, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(&status_info, Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title_alignment(Alignment::Center));
    frame.render_widget(header, chunks[0]);

    // ── Body ─────────────────────────────────────────────────────────────
    if let Some(ref error) = app.error_msg {
        if app.status.is_none() {
            let error_block = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("  ⚠ {error}"),
                    Style::default().fg(Color::Red),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  Start the daemon: termimon start",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(Block::default().borders(Borders::ALL).title(" Status "));
            frame.render_widget(error_block, chunks[1]);
        } else {
            draw_creatures(frame, chunks[1], app);
        }
    } else if let Some(ref status) = app.status {
        if status.agents.is_empty() {
            let empty = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  No agents detected yet...",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  Start an AI coding agent (Claude Code, Codex, aider) and it will appear here!",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(Block::default().borders(Borders::ALL).title(" Creatures "));
            frame.render_widget(empty, chunks[1]);
        } else {
            draw_creatures(frame, chunks[1], app);
        }
    } else {
        let loading = Paragraph::new("  Loading...")
            .block(Block::default().borders(Borders::ALL).title(" Creatures "));
        frame.render_widget(loading, chunks[1]);
    }

    // ── Footer ───────────────────────────────────────────────────────────
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" quit  "),
        Span::styled("↑↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" select  "),
        Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" refresh  "),
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Controls "));
    frame.render_widget(footer, chunks[2]);
}

fn draw_creatures(frame: &mut Frame, area: Rect, app: &DashApp) {
    let status = match &app.status {
        Some(s) => s,
        None => return,
    };

    let agent_count = status.agents.len();
    if agent_count == 0 {
        return;
    }

    // Each creature card needs ~12 rows (8 sprite + 4 info lines)
    let card_height = 14u16;

    // Create constraints for each agent card
    let constraints: Vec<Constraint> = status
        .agents
        .iter()
        .map(|_| Constraint::Length(card_height))
        .chain(std::iter::once(Constraint::Min(0))) // absorb remaining space
        .collect();

    let creature_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (i, agent) in status.agents.iter().enumerate() {
        if i >= creature_chunks.len() - 1 {
            break; // don't draw into the filler chunk
        }

        let is_selected = i == app.selected;
        let border_color = if is_selected {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        let card_area = creature_chunks[i];
        let species = &agent.creature_species;
        let creature_def = registry::get_creature_def(species);

        // Title: "🔥 Embercli (Stage 1) — Claude Code [idle]"
        let title = format!(
            " {} {} (Stage {}) — {} [{}] ",
            agent.element_icon,
            agent.creature_name,
            agent.stage,
            agent.kind,
            agent.state,
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                &title,
                Style::default()
                    .fg(if is_selected { Color::Yellow } else { Color::White })
                    .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
            ));

        let inner = block.inner(card_area);
        frame.render_widget(block, card_area);

        // Split inner into sprite area (left) and info area (right)
        let inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20), // sprite (16 chars + some padding)
                Constraint::Min(20),   // info
            ])
            .split(inner);

        // ── Sprite rendering ──────────────────────────────────────────
        let sprite = sprites::sprite_for_agent(&agent.kind);
        let sprite_lines = halfblock::render_sprite(sprite);
        let sprite_text: Vec<Line> = sprite_lines
            .iter()
            .map(|l| Line::from(Span::raw(format!("  {l}"))))
            .collect();
        let sprite_widget = Paragraph::new(sprite_text);
        frame.render_widget(sprite_widget, inner_chunks[0]);

        // ── Info panel ────────────────────────────────────────────────
        let state_emoji = match agent.state.as_str() {
            "idle" => "😊",
            "typing" => "⌨️",
            "thinking" => "🤔",
            "reading" => "📖",
            "running" => "🏃",
            "sleeping" => "💤",
            "error" => "💥",
            _ => "⏳",
        };

        let pid_str = agent.pid.map(|p| format!("PID: {p}")).unwrap_or_else(|| "PID: —".into());
        let xp_bar = halfblock::render_xp_bar(agent.xp as f64 / 100.0, 15);
        let desc = creature_def.map(|d| d.description).unwrap_or("").to_string();
        let state_label = format!("{state_emoji} {}", agent.state.to_uppercase());
        let xp_label = format!("XP: {} {xp_bar}", agent.xp);
        let elem_label = format!("Element: {}", agent.element_icon);

        let info_lines = vec![
            Line::from(Span::styled(
                state_label,
                Style::default().fg(state_color(&agent.state)).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(pid_str, Style::default().fg(Color::DarkGray))),
            Line::from(Span::raw(xp_label)),
            Line::from(""),
            Line::from(Span::styled(
                elem_label,
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(desc, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))),
        ];

        let info_widget = Paragraph::new(info_lines);
        frame.render_widget(info_widget, inner_chunks[1]);
    }
}

fn state_color(state: &str) -> Color {
    match state {
        "idle" => Color::Green,
        "typing" => Color::Cyan,
        "thinking" => Color::Magenta,
        "reading" => Color::Blue,
        "running" => Color::Yellow,
        "sleeping" => Color::DarkGray,
        "error" => Color::Red,
        _ => Color::White,
    }
}
