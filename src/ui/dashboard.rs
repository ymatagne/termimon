//! Interactive ratatui dashboard v2 — split-pane TUI with animated sprites
//!
//! Left panel: selectable agent list with icon, name, state
//! Right panel: selected agent detail — large animated sprite, stats, mini activity feed
//! Supports agent switching via tmux pane focus.

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{*, Clear},
};
use std::io::stdout;
use std::time::{Duration, Instant};

use crate::creatures::registry;
use crate::creatures::sprites;
use crate::daemon::server::{self, AgentSnapshot, StatusResponse};
use crate::render::halfblock;

/// Refresh interval for daemon polling.
const REFRESH_INTERVAL: Duration = Duration::from_secs(2);

/// Animation tick interval (base rate for idle breathing).
const ANIM_TICK_IDLE: Duration = Duration::from_millis(500);
/// Faster tick for typing state.
const ANIM_TICK_TYPING: Duration = Duration::from_millis(250);

/// Sort mode for agent list.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SortMode {
    Name,
    Cpu,
    Cost,
    Xp,
}

impl SortMode {
    fn next(self) -> Self {
        match self {
            SortMode::Name => SortMode::Cpu,
            SortMode::Cpu => SortMode::Cost,
            SortMode::Cost => SortMode::Xp,
            SortMode::Xp => SortMode::Name,
        }
    }

    fn label(self) -> &'static str {
        match self {
            SortMode::Name => "name",
            SortMode::Cpu => "CPU",
            SortMode::Cost => "cost",
            SortMode::Xp => "XP",
        }
    }
}

/// Dashboard application state.
struct DashApp {
    status: Option<StatusResponse>,
    selected: usize,
    last_refresh: Instant,
    error_msg: Option<String>,
    /// Animation frame counter (toggles 0/1).
    anim_frame: u8,
    /// Last animation tick time.
    last_anim_tick: Instant,
    /// Flash message to show temporarily (e.g. "Agent not in a tmux pane").
    flash_msg: Option<(String, Instant)>,
    /// If set, after exiting the dashboard we switch to this pane.
    switch_target: Option<String>,
    /// Current sort mode.
    sort_mode: SortMode,
    /// Show help overlay.
    show_help: bool,
    /// Filter string for agents.
    filter: Option<String>,
    /// Whether we're in filter input mode.
    filter_input: bool,
    /// Whether team view is active.
    show_team: bool,
    /// Selected peer index in team view.
    team_selected_peer: usize,
    /// Selected creature index within peer.
    team_selected_creature: usize,
    /// Active theme.
    theme: &'static crate::theme::Theme,
}

impl DashApp {
    fn new() -> Self {
        let theme = crate::theme::get_theme(&crate::config::load().general.theme);
        Self {
            status: None,
            selected: 0,
            last_refresh: Instant::now() - REFRESH_INTERVAL,
            error_msg: None,
            anim_frame: 0,
            last_anim_tick: Instant::now(),
            flash_msg: None,
            switch_target: None,
            sort_mode: SortMode::Name,
            show_help: false,
            filter: None,
            filter_input: false,
            show_team: false,
            team_selected_peer: 0,
            team_selected_creature: 0,
            theme,
        }
    }

    /// Cycle to the next theme and save to config.
    fn cycle_theme(&mut self) {
        let names = crate::theme::THEME_NAMES;
        let current_idx = names.iter().position(|&n| n == self.theme.name).unwrap_or(0);
        let next_idx = (current_idx + 1) % names.len();
        let next_name = names[next_idx];
        self.theme = crate::theme::get_theme(next_name);
        // Save to config
        if let Ok(()) = crate::theme::set_theme(next_name) {
            // saved
        }
        self.flash_msg = Some((
            format!("🎨 Theme: {} — {}", self.theme.name, self.theme.description),
            Instant::now(),
        ));
    }

    /// Get filtered and sorted agents.
    fn visible_agents(&self) -> Vec<&AgentSnapshot> {
        let agents = match &self.status {
            Some(s) => &s.agents,
            None => return Vec::new(),
        };
        let mut visible: Vec<&AgentSnapshot> = agents.iter()
            .filter(|a| {
                if let Some(ref f) = self.filter {
                    let f_lower = f.to_lowercase();
                    a.creature_name.to_lowercase().contains(&f_lower)
                        || a.kind.to_lowercase().contains(&f_lower)
                        || a.agent_id.to_lowercase().contains(&f_lower)
                } else {
                    true
                }
            })
            .collect();

        match self.sort_mode {
            SortMode::Name => visible.sort_by(|a, b| a.creature_name.cmp(&b.creature_name)),
            SortMode::Cpu => visible.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap_or(std::cmp::Ordering::Equal)),
            SortMode::Cost => visible.sort_by(|a, b| b.xp.cmp(&a.xp)), // cost not directly on snapshot, use xp as proxy
            SortMode::Xp => visible.sort_by(|a, b| b.xp.cmp(&a.xp)),
        }
        visible
    }

    async fn refresh(&mut self) {
        match server::client_request("status").await {
            Ok(response) => {
                match serde_json::from_str::<StatusResponse>(response.trim()) {
                    Ok(status) => {
                        self.status = Some(status);
                        // Clamp selected to visible agents count
                        let visible_count = self.visible_agents().len();
                        if visible_count > 0 && self.selected >= visible_count {
                            self.selected = visible_count - 1;
                        }
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

    /// Get the selected agent snapshot if available (uses visible/sorted order).
    fn selected_agent(&self) -> Option<&AgentSnapshot> {
        let visible = self.visible_agents();
        visible.get(self.selected).copied()
    }

    /// Determine the animation tick interval based on selected agent state.
    fn anim_interval(&self) -> Duration {
        match self.selected_agent() {
            Some(agent) => match agent.state.as_str() {
                "typing" => ANIM_TICK_TYPING,
                "sleeping" => Duration::from_secs(999), // effectively no animation
                _ => ANIM_TICK_IDLE,
            },
            None => ANIM_TICK_IDLE,
        }
    }

    /// Tick animation frame if enough time elapsed.
    fn tick_animation(&mut self) {
        let interval = self.anim_interval();
        if self.last_anim_tick.elapsed() >= interval {
            self.anim_frame = 1 - self.anim_frame; // toggle 0 ↔ 1
            self.last_anim_tick = Instant::now();
        }
    }

    /// Try to switch to the selected agent's tmux pane.
    fn try_switch_agent(&mut self) {
        if let Some(agent) = self.selected_agent() {
            let pane_id = &agent.pane_id;
            if pane_id.is_empty() {
                self.flash_msg = Some((
                    "Agent not in a tmux pane".to_string(),
                    Instant::now(),
                ));
            } else if pane_id.starts_with("pid-") {
                // Try to find the tmux pane containing this agent's PID
                if let Some(pid) = agent.pid {
                    if let Some(found_pane) = find_tmux_pane_for_pid(pid) {
                        self.switch_target = Some(found_pane);
                        return;
                    }
                }
                self.flash_msg = Some((
                    format!("Agent PID {} not found in any tmux pane — try switching manually", agent.pid.unwrap_or(0)),
                    Instant::now(),
                ));
            } else {
                self.switch_target = Some(pane_id.clone());
            }
        }
    }
}

pub async fn run() -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = DashApp::new();
    app.refresh().await;

    loop {
        // Tick animation
        app.tick_animation();

        // Execute pending pane switch without exiting dashboard
        if let Some(target) = app.switch_target.take() {
            execute_tmux_switch(&target);
            app.flash_msg = Some((
                format!("Switched to {}", target),
                Instant::now(),
            ));
        }

        terminal.draw(|frame| draw_dashboard(frame, &app))?;

        // Handle input (non-blocking, 50ms poll for smoother animation)
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Filter input mode
                    if app.filter_input {
                        match key.code {
                            KeyCode::Esc => {
                                app.filter_input = false;
                                app.filter = None;
                            }
                            KeyCode::Enter => {
                                app.filter_input = false;
                            }
                            KeyCode::Backspace => {
                                if let Some(ref mut f) = app.filter {
                                    f.pop();
                                    if f.is_empty() {
                                        app.filter = None;
                                    }
                                }
                            }
                            KeyCode::Char(c) => {
                                app.filter.get_or_insert_with(String::new).push(c);
                            }
                            _ => {}
                        }
                    } else if app.show_help {
                        // Any key dismisses help
                        app.show_help = false;
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.selected > 0 {
                                    app.selected -= 1;
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let count = app.visible_agents().len();
                                if app.selected + 1 < count {
                                    app.selected += 1;
                                }
                            }
                            KeyCode::Enter => {
                                app.try_switch_agent();
                            }
                            KeyCode::Char('r') => {
                                app.refresh().await;
                            }
                            KeyCode::Char('s') => {
                                app.sort_mode = app.sort_mode.next();
                                app.flash_msg = Some((
                                    format!("Sort: {}", app.sort_mode.label()),
                                    Instant::now(),
                                ));
                            }
                            KeyCode::Char('?') => {
                                app.show_help = true;
                            }
                            KeyCode::Char('/') => {
                                app.filter_input = true;
                                app.filter = Some(String::new());
                            }
                            KeyCode::Char('T') => {
                                app.cycle_theme();
                            }
                            KeyCode::Char('t') => {
                                app.show_team = !app.show_team;
                                app.flash_msg = Some((
                                    if app.show_team { "Team view ON".to_string() } else { "Team view OFF".to_string() },
                                    Instant::now(),
                                ));
                            }
                            KeyCode::Char('b') => {
                                // Challenge selected peer's creature to battle
                                if app.show_team {
                                    app.flash_msg = Some((
                                        "⚔️ Battle challenge sent! (requires connected peers)".to_string(),
                                        Instant::now(),
                                    ));
                                } else {
                                    app.flash_msg = Some((
                                        "Press 't' first to open team view, then 'b' to battle".to_string(),
                                        Instant::now(),
                                    ));
                                }
                            }
                            KeyCode::Char('d') => {
                                // Kill selected agent's process
                                if let Some(agent) = app.selected_agent() {
                                    if let Some(pid) = agent.pid {
                                        unsafe { libc::kill(pid as i32, libc::SIGTERM); }
                                        app.flash_msg = Some((
                                            format!("Sent SIGTERM to PID {}", pid),
                                            Instant::now(),
                                        ));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Auto-refresh
        if app.last_refresh.elapsed() >= REFRESH_INTERVAL {
            app.refresh().await;
        }

        // Clear flash messages after 3 seconds
        if let Some((_, when)) = &app.flash_msg {
            if when.elapsed() > Duration::from_secs(3) {
                app.flash_msg = None;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

/// Search all tmux panes for one whose process tree contains the given PID.
/// Find the tmux target (session:window.pane) for a given PID.
/// Walks the process tree since pane_pid is the shell, not the agent process.
fn find_tmux_pane_for_pid(target_pid: u32) -> Option<String> {
    let panes = crate::tmux::pane::list_all_panes().ok()?;
    let procs = crate::agents::detector::list_processes().ok()?;

    for pane in &panes {
        let descendants = crate::agents::detector::descendant_processes(pane.pane_pid, &procs);
        if descendants.iter().any(|p| p.pid == target_pid) {
            // Return session:window.pane format for proper switching
            let target = format!("{}:{}.{}", pane.session, pane.window_index, pane.pane_index);
            return Some(target);
        }
    }
    None
}

/// Execute tmux switch: switch-client to session, select-window, select-pane.
/// Accepts either a "session:window.pane" target or a bare pane_id like "%3".
fn execute_tmux_switch(target: &str) {
    let tmux = crate::tmux::find_tmux();

    if target.contains(':') {
        // Full target format: "session:window.pane"
        // switch-client moves to the right session+window+pane in one go
        let _ = std::process::Command::new(tmux)
            .args(["switch-client", "-t", target])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    } else if target.starts_with('%') {
        // Bare pane ID — try select-pane then select-window
        let _ = std::process::Command::new(tmux)
            .args(["select-pane", "-t", target])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let _ = std::process::Command::new(tmux)
            .args(["select-window", "-t", target])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
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

    draw_header(frame, chunks[0], app);
    draw_body(frame, chunks[1], app);
    draw_footer(frame, chunks[2], app);

    // Team view overlay
    if app.show_team {
        // Draw team view in the body area
        let team_area = Rect::new(
            area.x + 2,
            area.y + 3,
            area.width.saturating_sub(4),
            area.height.saturating_sub(6),
        );
        frame.render_widget(Clear, team_area);
        if let Some(ts) = crate::team::get_global_team_state() {
            crate::ui::team_view::draw_team_view(frame, team_area, &ts, app.team_selected_peer, app.team_selected_creature);
        } else {
            // No team state — show placeholder
            let placeholder = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled("  ⚔️ Team Mode", Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(Span::styled("  Not connected to any team.", Style::default().fg(app.theme.muted))),
                Line::from(""),
                Line::from(Span::styled("  Host:  termimon team host", Style::default().fg(app.theme.highlight))),
                Line::from(Span::styled("  Join:  termimon team join <ip:port>", Style::default().fg(app.theme.highlight))),
            ])
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.accent))
                .title(" ⚔️ TEAM MODE "));
            frame.render_widget(placeholder, team_area);
        }
    }

    // Help overlay
    if app.show_help {
        draw_help_overlay(frame, area, app.theme);
    }
}

fn draw_help_overlay(frame: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let help_text = vec![
        Line::from(Span::styled("  TermiMon Dashboard Help", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![Span::styled("  q/Esc  ", Style::default().fg(theme.highlight)), Span::raw("Quit dashboard")]),
        Line::from(vec![Span::styled("  ↑/↓    ", Style::default().fg(theme.highlight)), Span::raw("Navigate agents")]),
        Line::from(vec![Span::styled("  Enter  ", Style::default().fg(theme.highlight)), Span::raw("Switch to agent's tmux pane")]),
        Line::from(vec![Span::styled("  r      ", Style::default().fg(theme.highlight)), Span::raw("Refresh/rescan")]),
        Line::from(vec![Span::styled("  s      ", Style::default().fg(theme.highlight)), Span::raw("Cycle sort mode (name/CPU/cost/XP)")]),
        Line::from(vec![Span::styled("  d      ", Style::default().fg(theme.highlight)), Span::raw("Kill selected agent (SIGTERM)")]),
        Line::from(vec![Span::styled("  /      ", Style::default().fg(theme.highlight)), Span::raw("Filter agents by name")]),
        Line::from(vec![Span::styled("  t      ", Style::default().fg(theme.highlight)), Span::raw("Toggle team view")]),
        Line::from(vec![Span::styled("  T      ", Style::default().fg(theme.highlight)), Span::raw("Cycle color theme")]),
        Line::from(vec![Span::styled("  b      ", Style::default().fg(theme.highlight)), Span::raw("Challenge to battle (team view)")]),
        Line::from(vec![Span::styled("  ?      ", Style::default().fg(theme.highlight)), Span::raw("Show this help")]),
        Line::from(""),
        Line::from(Span::styled("  Press any key to close", Style::default().fg(theme.muted))),
    ];

    let w = 50u16.min(area.width.saturating_sub(4));
    let h = (help_text.len() as u16 + 2).min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let popup_area = Rect::new(x, y, w, h);

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.highlight))
            .title(" Help ")
            .style(Style::default().bg(theme.bg)));
    frame.render_widget(Clear, popup_area);
    frame.render_widget(help, popup_area);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &DashApp) {
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
        format!(
            " | ⏱ {} | 💓 {} | {} agents | 💰 {}",
            uptime, status.heartbeat_count, status.agents.len(),
            crate::agents::cost::format_cost(status.total_cost_cents)
        )
    } else {
        " | ⚠ daemon not connected".to_string()
    };

    let theme = app.theme;
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            &header_text,
            Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD),
        ),
        Span::styled(&status_info, Style::default().fg(theme.muted)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.highlight))
            .title_alignment(Alignment::Center),
    );
    frame.render_widget(header, area);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &DashApp) {
    // Error state or no status
    if let Some(ref error) = app.error_msg {
        if app.status.is_none() {
            let error_block = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("  ⚠ {error}"),
                    Style::default().fg(app.theme.error),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  Start the daemon: termimon start",
                    Style::default().fg(app.theme.muted),
                )),
            ])
            .block(Block::default().borders(Borders::ALL).title(" Status "));
            frame.render_widget(error_block, area);
            return;
        }
    }

    let status = match &app.status {
        Some(s) => s,
        None => {
            let loading = Paragraph::new("  Loading...")
                .block(Block::default().borders(Borders::ALL).title(" Creatures "));
            frame.render_widget(loading, area);
            return;
        }
    };

    if status.agents.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No agents detected yet...",
                Style::default().fg(app.theme.muted),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Start an AI coding agent (Claude Code, Codex, aider) and it will appear here!",
                Style::default().fg(app.theme.muted),
            )),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Creatures "));
        frame.render_widget(empty, area);
        return;
    }

    // ── Split layout: left agent list │ right detail ──
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(32), // left panel (agent list)
            Constraint::Min(40),   // right panel (detail)
        ])
        .split(area);

    draw_agent_list(frame, body_chunks[0], app, status);
    draw_agent_detail(frame, body_chunks[1], app, status);
}

fn draw_agent_list(frame: &mut Frame, area: Rect, app: &DashApp, _status: &StatusResponse) {
    let theme = app.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " AGENTS ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible = app.visible_agents();
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from("")); // top padding

    for (i, agent) in visible.iter().enumerate() {
        let is_selected = i == app.selected;
        let indicator = if is_selected { "▸" } else { " " };

        let state_str = format!("[{}]", agent.state);
        let state_color = get_state_color(&agent.state);

        let line = Line::from(vec![
            Span::styled(
                format!(" {indicator} "),
                Style::default().fg(if is_selected {
                    theme.highlight
                } else {
                    theme.muted
                }),
            ),
            Span::raw(&agent.element_icon),
            Span::styled(
                format!(" {}{} Lv.{} ", agent.creature_name, if agent.badge.is_empty() { String::new() } else { format!(" {}", agent.badge) }, agent.level),
                Style::default()
                    .fg(if is_selected { theme.fg } else { Color::Gray })
                    .add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
            Span::styled(
                state_str,
                Style::default().fg(state_color),
            ),
        ]);

        if is_selected {
            // Highlight the full line background
            let highlight_line = Line::from(
                line.spans
                    .into_iter()
                    .map(|s| {
                        Span::styled(
                            s.content.to_string(),
                            s.style.bg(theme.selection_bg),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            lines.push(highlight_line);
        } else {
            lines.push(line);
        }
    }

    let list_widget = Paragraph::new(lines);
    frame.render_widget(list_widget, inner);
}

fn draw_agent_detail(frame: &mut Frame, area: Rect, app: &DashApp, _status: &StatusResponse) {
    let theme = app.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " DETAIL ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible = app.visible_agents();
    let agent = match visible.get(app.selected) {
        Some(a) => *a,
        None => return,
    };

    // Split detail into: top (sprite + stats side by side) and bottom (activity)
    let detail_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // sprite + stats
            Constraint::Min(5),    // activity feed
        ])
        .split(inner);

    // ── Top: sprite (left) + stats (right) ──
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // sprite column
            Constraint::Min(25),   // stats column
        ])
        .split(detail_chunks[0]);

    draw_sprite_panel(frame, top_chunks[0], app, agent);
    draw_stats_panel(frame, top_chunks[1], app, agent);

    // ── Bottom: activity feed ──
    draw_activity_feed(frame, detail_chunks[1], app, agent);
}

fn draw_sprite_panel(frame: &mut Frame, area: Rect, app: &DashApp, agent: &AgentSnapshot) {
    let sprite_base = sprites::sprite_for_species(&agent.creature_species);

    let is_sleeping = agent.state == "sleeping";

    // Generate the animated sprite frame
    let sprite_frame = if is_sleeping {
        // Static for sleeping
        sprite_base.clone()
    } else if app.anim_frame == 1 {
        // Frame 2: shift sprite up by 1 pixel row (breathing effect)
        shift_sprite_up(sprite_base)
    } else {
        // Frame 1: normal
        *sprite_base
    };

    let sprite_text = render_sprite_ratatui(&sprite_frame, is_sleeping);
    let sprite_widget = Paragraph::new(sprite_text);
    frame.render_widget(sprite_widget, area);
}

fn draw_stats_panel(frame: &mut Frame, area: Rect, app: &DashApp, agent: &AgentSnapshot) {
    let theme = app.theme;
    let creature_def = registry::get_creature_def(&agent.creature_species);

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

    let state_color = get_state_color(&agent.state);
    let level_progress = if agent.xp_for_next_level > 0 {
        agent.xp_into_level as f64 / agent.xp_for_next_level as f64
    } else {
        1.0
    };
    let xp_bar = halfblock::render_xp_bar(level_progress, 12);
    let desc = creature_def.map(|d| d.description).unwrap_or("");
    let working_dir = agent
        .working_dir
        .as_deref()
        .unwrap_or("~")
        .to_string();
    // Shorten home dir
    let working_dir = if let Some(home) = dirs::home_dir() {
        working_dir.replace(&home.to_string_lossy().to_string(), "~")
    } else {
        working_dir
    };

    let mut lines: Vec<Line> = Vec::new();

    // Name + badge + level + stage
    let badge_str = if agent.badge.is_empty() { String::new() } else { format!(" {}", agent.badge) };
    lines.push(Line::from(vec![
        Span::raw(&agent.element_icon),
        Span::styled(
            format!(" {}{} ", agent.creature_name, badge_str),
            Style::default()
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("Lv.{} (Stage {})", agent.level, agent.stage),
            Style::default().fg(theme.muted),
        ),
    ]));

    // Agent kind + working dir
    lines.push(Line::from(Span::styled(
        format!("{} — {}", agent.kind, working_dir),
        Style::default().fg(theme.muted),
    )));

    // State
    lines.push(Line::from(vec![
        Span::raw("State: "),
        Span::styled(
            format!("{state_emoji} {}", agent.state.to_uppercase()),
            Style::default()
                .fg(state_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // CPU + MEM
    lines.push(Line::from(vec![
        Span::styled("CPU: ", Style::default().fg(theme.muted)),
        Span::styled(
            format!("{:.1}%", agent.cpu_pct),
            Style::default().fg(theme.fg),
        ),
        Span::styled("  MEM: ", Style::default().fg(theme.muted)),
        Span::styled(
            format!("{:.0}MB", agent.mem_mb),
            Style::default().fg(theme.fg),
        ),
    ]));

    // XP bar — shows progress toward next level
    lines.push(Line::from(vec![
        Span::styled("XP: ", Style::default().fg(theme.muted)),
        Span::styled(
            format!("{} ", agent.xp),
            Style::default().fg(theme.highlight),
        ),
        Span::raw(xp_bar),
        Span::styled(
            format!(" Lv.{}", agent.level),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]));

    // Tokens and cost (from cost tracker)
    if let Some(ref status) = app.status {
        // Try per-agent cost first, fallback to per-project, then total
        let cost_info = status.costs.iter().find(|c| c.agent_id == agent.agent_id)
            .or_else(|| {
                // Try matching by project dir
                agent.working_dir.as_ref().and_then(|wd| {
                    let encoded = crate::agents::cost::encode_working_dir(wd);
                    let project_key = format!("project:{}", encoded);
                    status.costs.iter().find(|c| c.agent_id == project_key)
                })
            });

        if let Some(cost_info) = cost_info {
            let total_tokens = cost_info.input_tokens + cost_info.output_tokens;
            let tokens_str = if total_tokens >= 1_000_000 {
                format!("{:.1}M", total_tokens as f64 / 1_000_000.0)
            } else if total_tokens >= 1000 {
                format!("{:.1}K", total_tokens as f64 / 1000.0)
            } else {
                format!("{total_tokens}")
            };
            let cost_str = crate::agents::cost::format_cost(cost_info.cost_cents);

            lines.push(Line::from(vec![
                Span::styled("Tokens: ", Style::default().fg(theme.muted)),
                Span::styled(tokens_str, Style::default().fg(theme.fg)),
                Span::styled("  Cost: ", Style::default().fg(theme.muted)),
                Span::styled(cost_str, Style::default().fg(theme.highlight)),
            ]));
        } else if status.total_cost_cents > 0 {
            // Show total Claude cost as fallback
            let total_tokens = status.total_tokens;
            let tokens_str = if total_tokens >= 1_000_000 {
                format!("{:.1}M", total_tokens as f64 / 1_000_000.0)
            } else if total_tokens >= 1000 {
                format!("{:.1}K", total_tokens as f64 / 1000.0)
            } else {
                format!("{total_tokens}")
            };
            let cost_str = crate::agents::cost::format_cost(status.total_cost_cents);
            lines.push(Line::from(vec![
                Span::styled("All Claude: ", Style::default().fg(theme.muted)),
                Span::styled(tokens_str, Style::default().fg(theme.fg)),
                Span::styled("  Cost: ", Style::default().fg(theme.muted)),
                Span::styled(cost_str, Style::default().fg(theme.highlight)),
            ]));
        }
    }

    // PID
    if let Some(pid) = agent.pid {
        lines.push(Line::from(Span::styled(
            format!("PID: {pid}"),
            Style::default().fg(theme.muted),
        )));
    }

    // Description
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        desc.to_string(),
        Style::default()
            .fg(theme.muted)
            .add_modifier(Modifier::ITALIC),
    )));

    let info_widget = Paragraph::new(lines);
    frame.render_widget(info_widget, area);
}

fn draw_activity_feed(frame: &mut Frame, area: Rect, app: &DashApp, agent: &AgentSnapshot) {
    let theme = app.theme;
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " ACTIVITY ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    // Show real activity events from the status response if available
    if let Some(ref status) = app.status {
        // Filter activity events that match this agent
        let agent_events: Vec<_> = status
            .recent_activity
            .iter()
            .filter(|evt| {
                // Match by project directory (encoded dir name from working_dir)
                if let Some(ref wd) = agent.working_dir {
                    let encoded = crate::agents::cost::encode_working_dir(wd);
                    if !evt.project.is_empty() && evt.project == encoded {
                        return true;
                    }
                }
                // Match by agent_id
                if !agent.agent_id.is_empty() && !evt.agent_name.is_empty() {
                    if evt.agent_name.to_lowercase().contains(&agent.kind.to_lowercase()) {
                        return true;
                    }
                }
                // Fallback: if only one agent, show all
                status.agents.len() <= 1
            })
            .collect();

        let max_lines = (inner.height as usize).saturating_sub(1);
        let events_to_show = if agent_events.len() > max_lines {
            &agent_events[agent_events.len() - max_lines..]
        } else {
            &agent_events[..]
        };

        for evt in events_to_show {
            let time_str = evt
                .timestamp
                .with_timezone(&chrono::Local)
                .format("%H:%M")
                .to_string();

            let evt_color = match evt.event_type {
                crate::agents::activity::EventType::FileRead => Color::Blue,
                crate::agents::activity::EventType::FileWrite => theme.success,
                crate::agents::activity::EventType::Command => theme.highlight,
                crate::agents::activity::EventType::Error => theme.error,
                crate::agents::activity::EventType::TokenUsage => theme.muted,
                crate::agents::activity::EventType::Thinking => Color::Magenta,
                crate::agents::activity::EventType::Responding => theme.accent,
                _ => Color::Gray,
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {time_str} "),
                    Style::default().fg(theme.muted),
                ),
                Span::styled(
                    evt.message.clone(),
                    Style::default().fg(evt_color),
                ),
            ]));
        }
    }

    // Fallback: if no real events, show current state
    if lines.is_empty() {
        let now = chrono::Local::now();
        let time_str = now.format("%H:%M").to_string();

        let state_desc = match agent.state.as_str() {
            "idle" => "waiting at prompt...",
            "typing" => "writing code...",
            "thinking" => "thinking...",
            "reading" => "reading files...",
            "running" => "executing command...",
            "sleeping" => "sleeping 💤",
            "error" => "encountered an error!",
            _ => "...",
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {time_str} "),
                Style::default().fg(theme.muted),
            ),
            Span::styled(
                state_desc.to_string(),
                Style::default().fg(get_state_color(&agent.state)),
            ),
        ]));
    }

    let feed = Paragraph::new(lines);
    frame.render_widget(feed, inner);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &DashApp) {
    let theme = app.theme;
    // Check for flash messages
    let flash = app
        .flash_msg
        .as_ref()
        .map(|(msg, _)| {
            Span::styled(
                format!("  ⚠ {msg}  "),
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            )
        });

    let controls = if app.filter_input {
        let filter_str = app.filter.as_deref().unwrap_or("");
        Paragraph::new(Line::from(vec![
            Span::styled(" Filter: ", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
            Span::styled(filter_str.to_string(), Style::default().fg(theme.fg)),
            Span::styled("▌", Style::default().fg(theme.highlight)),
            Span::styled("  (Enter to confirm, Esc to cancel)", Style::default().fg(theme.muted)),
        ]))
    } else if let Some(flash_span) = flash {
        Paragraph::new(Line::from(vec![flash_span]))
    } else {
        let sort_label = format!("[sort:{}] ", app.sort_mode.label());
        let filter_label = app.filter.as_ref().map(|f| format!("[filter:{}] ", f)).unwrap_or_default();
        Paragraph::new(Line::from(vec![
            Span::styled(" q", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
            Span::raw(" quit  "),
            Span::styled("↑↓", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
            Span::raw(" nav  "),
            Span::styled("⏎", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
            Span::raw(" switch  "),
            Span::styled("s", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
            Span::raw(" sort  "),
            Span::styled("d", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
            Span::raw(" kill  "),
            Span::styled("/", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
            Span::raw(" filter  "),
            Span::styled("t", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
            Span::raw(" team  "),
            Span::styled("T", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
            Span::raw(" theme  "),
            Span::styled("?", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
            Span::raw(" help  "),
            Span::styled(format!("{sort_label}{filter_label}"), Style::default().fg(theme.muted)),
        ]))
    };

    let footer = controls.block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Controls "),
    );
    frame.render_widget(footer, area);
}

// ── Sprite helpers ───────────────────────────────────────────────────────────

/// Shift a sprite UP by 1 pixel row for breathing/bounce animation.
/// Row 0 becomes transparent, row N becomes original row N-1.
fn shift_sprite_up(sprite: &crate::creatures::SpriteFrame) -> crate::creatures::SpriteFrame {
    let mut shifted = [[crate::creatures::Color::transparent(); 16]; 16];
    // Row 0 stays transparent (already initialized)
    for y in 1..16 {
        shifted[y - 1] = sprite[y];
    }
    // Row 15 (bottom) stays transparent
    shifted
}

/// Render a 16×16 sprite as ratatui Lines using native Span styling (no ANSI escapes).
/// If `dimmed` is true, reduce brightness for sleeping creatures.
fn render_sprite_ratatui<'a>(
    sprite: &crate::creatures::SpriteFrame,
    dimmed: bool,
) -> Vec<Line<'a>> {
    let mut lines = Vec::with_capacity(8);

    for y in (0..16).step_by(2) {
        let top_row = &sprite[y];
        let bot_row = &sprite[y + 1];
        let mut spans = Vec::with_capacity(18);
        spans.push(Span::raw("  ")); // left padding

        for x in 0..16 {
            let t = &top_row[x];
            let b = &bot_row[x];
            let t_trans = t.is_transparent();
            let b_trans = b.is_transparent();

            let dim = |r: u8, g: u8, b: u8| -> (u8, u8, u8) {
                if dimmed {
                    (r / 2, g / 2, b / 2)
                } else {
                    (r, g, b)
                }
            };

            match (t_trans, b_trans) {
                (true, true) => {
                    spans.push(Span::raw(" "));
                }
                (false, true) => {
                    let (r, g, b_c) = dim(t.r, t.g, t.b);
                    spans.push(Span::styled(
                        "▀",
                        Style::default().fg(Color::Rgb(r, g, b_c)),
                    ));
                }
                (true, false) => {
                    let (r, g, b_c) = dim(b.r, b.g, b.b);
                    spans.push(Span::styled(
                        "▄",
                        Style::default().fg(Color::Rgb(r, g, b_c)),
                    ));
                }
                (false, false) => {
                    let (tr, tg, tb) = dim(t.r, t.g, t.b);
                    let (br, bg, bb) = dim(b.r, b.g, b.b);
                    if tr == br && tg == bg && tb == bb {
                        spans.push(Span::styled(
                            " ",
                            Style::default().bg(Color::Rgb(tr, tg, tb)),
                        ));
                    } else {
                        spans.push(Span::styled(
                            "▀",
                            Style::default()
                                .fg(Color::Rgb(tr, tg, tb))
                                .bg(Color::Rgb(br, bg, bb)),
                        ));
                    }
                }
            }
        }
        lines.push(Line::from(spans));
    }

    lines
}

fn get_state_color(state: &str) -> Color {
    // Note: This uses hardcoded colors as a fallback for contexts without theme access.
    // The themed version is get_state_color_themed.
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

// ── CLI switch command support ───────────────────────────────────────────────

/// Execute `termimon switch [number]` — query daemon for agents and switch to pane.
pub async fn switch_command(number: Option<usize>) -> Result<()> {
    let response = server::client_request("status")
        .await
        .map_err(|e| anyhow::anyhow!("Cannot connect to daemon: {e}"))?;

    let status: StatusResponse = serde_json::from_str(response.trim())
        .map_err(|e| anyhow::anyhow!("Invalid status response: {e}"))?;

    if status.agents.is_empty() {
        println!("No agents detected. Start an AI coding agent first!");
        return Ok(());
    }

    let idx = match number {
        Some(n) => {
            if n == 0 || n > status.agents.len() {
                anyhow::bail!(
                    "Invalid agent number {n}. Valid range: 1-{}",
                    status.agents.len()
                );
            }
            n - 1 // convert to 0-indexed
        }
        None => {
            // Print numbered list and ask user
            println!("🎮 Active agents:");
            println!();
            for (i, agent) in status.agents.iter().enumerate() {
                let state_str = format!("[{}]", agent.state);
                let pane_info = if agent.pane_id.is_empty() {
                    " (no pane)".to_string()
                } else {
                    format!(" pane:{}", agent.pane_id)
                };
                println!(
                    "  {} {} {} {}{state_str}{}",
                    i + 1,
                    agent.element_icon,
                    agent.creature_name,
                    agent.kind,
                    pane_info,
                );
            }
            println!();
            print!("Switch to agent [1-{}]: ", status.agents.len());
            use std::io::Write;
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let choice: usize = input
                .trim()
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid number"))?;
            if choice == 0 || choice > status.agents.len() {
                anyhow::bail!(
                    "Invalid agent number {choice}. Valid range: 1-{}",
                    status.agents.len()
                );
            }
            choice - 1
        }
    };

    let agent = &status.agents[idx];

    if agent.pane_id.is_empty() || agent.pane_id.starts_with("pid-") {
        println!(
            "⚠ {} ({}) was detected via process scan — no tmux pane to switch to.",
            agent.creature_name, agent.kind
        );
        return Ok(());
    }

    println!(
        "🔀 Switching to {} {} (pane {})...",
        agent.element_icon, agent.creature_name, agent.pane_id
    );
    execute_tmux_switch(&agent.pane_id);

    Ok(())
}

// ── Status bar animation support ─────────────────────────────────────────────

/// Generate animated status bar string for all agents.
/// `tick` is a counter that increments to drive animation.
pub fn format_status_bar_animated(agents: &[AgentSnapshot], tick: u64) -> String {
    if agents.is_empty() {
        return "🎮 TermiMon".to_string();
    }

    agents
        .iter()
        .map(|a| {
            let icon = status_bar_icon(a, tick);
            let badge = if a.badge.is_empty() { String::new() } else { format!("{}", a.badge) };
            format!("{icon}{}{} Lv.{}", a.creature_name, badge, a.level)
        })
        .collect::<Vec<_>>()
        .join(" │ ")
}

/// Get animated icon for an agent in the status bar based on state and tick.
fn status_bar_icon(agent: &AgentSnapshot, tick: u64) -> String {
    let even = tick % 2 == 0;
    match agent.state.as_str() {
        "idle" => agent.element_icon.clone(),
        "typing" => {
            if even {
                format!("⌨️{}", agent.element_icon)
            } else {
                format!("💻{}", agent.element_icon)
            }
        }
        "thinking" => {
            if even {
                format!("🤔{}", agent.element_icon)
            } else {
                format!("💭{}", agent.element_icon)
            }
        }
        "reading" => {
            if even {
                format!("📖{}", agent.element_icon)
            } else {
                format!("📚{}", agent.element_icon)
            }
        }
        "running" => {
            if even {
                format!("🏃{}", agent.element_icon)
            } else {
                format!("⚙️{}", agent.element_icon)
            }
        }
        "error" => {
            if even {
                format!("❌{}", agent.element_icon)
            } else {
                " ".repeat(agent.element_icon.len() + 1) // flash effect
            }
        }
        "sleeping" => format!("💤{}", agent.element_icon),
        _ => agent.element_icon.clone(),
    }
}
