//! Team dashboard panel — shows all connected peers' creatures side-by-side.

use ratatui::{
    prelude::*,
    widgets::*,
};

use crate::team::SharedTeamState;
use crate::team::battle::BattleResult;
use crate::team::protocol::CreatureSync;

/// Draw the team view overlay/panel.
pub fn draw_team_view(frame: &mut Frame, area: Rect, team_state: &SharedTeamState, selected_peer: usize, selected_creature: usize) {
    let theme = crate::theme::get_theme(&crate::config::load().general.theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            " ⚔️ TEAM MODE ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let ts = match team_state.lock() {
        Ok(ts) => ts,
        Err(_) => return,
    };

    if !ts.connected && !ts.hosting {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Not connected to any team.",
                Style::default().fg(theme.muted),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Host:  termimon team host",
                Style::default().fg(theme.highlight),
            )),
            Line::from(Span::styled(
                "  Join:  termimon team join <ip:port>",
                Style::default().fg(theme.highlight),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press 't' to close team view",
                Style::default().fg(theme.muted),
            )),
        ]);
        frame.render_widget(msg, inner);
        return;
    }

    // Split: peers list (left) | battle log (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65),
            Constraint::Percentage(35),
        ])
        .split(inner);

    draw_peers_panel(frame, chunks[0], &ts, selected_peer, selected_creature, theme);
    draw_battle_log(frame, chunks[1], &ts.battle_log, theme);
}

fn draw_peers_panel(
    frame: &mut Frame,
    area: Rect,
    ts: &crate::team::TeamState,
    selected_peer: usize,
    selected_creature: usize,
    theme: &crate::theme::Theme,
) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme.muted))
        .title(Span::styled(
            format!(
                " Peers ({}) ",
                ts.registry.peers.len() + 1 // +1 for self
            ),
            Style::default().fg(theme.accent),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    // Show local creatures first
    lines.push(Line::from(vec![
        Span::styled(
            format!(" 👤 {} ", ts.local_name),
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            if ts.hosting { "(host)" } else { "(client)" },
            Style::default().fg(theme.muted),
        ),
    ]));

    // Get local creatures from daemon state
    if let Some(daemon_state) = crate::daemon::server::get_global_state() {
        if let Ok(st) = daemon_state.lock() {
            for agent in &st.agents {
                let icon = agent.element_icon.clone();
                let name = agent.creature_name.clone();
                let xp = agent.xp;
                let state = agent.state.clone();
                lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::raw(icon),
                    Span::styled(
                        format!(" {} ", name),
                        Style::default().fg(theme.fg),
                    ),
                    Span::styled(
                        format!("Lv{} ", xp),
                        Style::default().fg(theme.highlight),
                    ),
                    Span::styled(
                        format!("[{}]", state),
                        Style::default().fg(state_color(&state)),
                    ),
                ]));
            }
        }
    }

    lines.push(Line::from(""));

    // Show each peer's creatures
    let peer_names: Vec<String> = ts.registry.peer_names();
    for (pi, peer_name) in peer_names.iter().enumerate() {
        let is_selected = pi == selected_peer;
        let indicator = if is_selected { "▸" } else { " " };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{indicator}👤 {} ", peer_name),
                Style::default()
                    .fg(if is_selected { theme.accent } else { theme.accent })
                    .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
            ),
        ]));

        if let Some(peer) = ts.registry.peers.get(peer_name) {
            for (ci, creature) in peer.creatures.iter().enumerate() {
                let is_creature_selected = is_selected && ci == selected_creature;
                let c_indicator = if is_creature_selected { "►" } else { " " };
                let element_icon = creature_element_icon(&creature.species);

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {c_indicator}"),
                        Style::default().fg(if is_creature_selected { theme.highlight } else { theme.muted }),
                    ),
                    Span::raw(element_icon),
                    Span::styled(
                        format!(" {} ", creature.name),
                        Style::default().fg(if is_creature_selected { theme.fg } else { Color::Gray }),
                    ),
                    Span::styled(
                        format!("S{} ", creature.stage),
                        Style::default().fg(theme.highlight),
                    ),
                    Span::styled(
                        format!("XP:{} ", creature.xp),
                        Style::default().fg(theme.muted),
                    ),
                    Span::styled(
                        format!("[{}]", creature.state),
                        Style::default().fg(state_color(&creature.state)),
                    ),
                ]));
            }

            if peer.creatures.is_empty() {
                lines.push(Line::from(Span::styled(
                    "   (no creatures)",
                    Style::default().fg(theme.muted),
                )));
            }
        }

        lines.push(Line::from(""));
    }

    if peer_names.is_empty() {
        lines.push(Line::from(Span::styled(
            " Waiting for peers to connect...",
            Style::default().fg(theme.muted),
        )));
    }

    let widget = Paragraph::new(lines);
    frame.render_widget(widget, inner);
}

fn draw_battle_log(frame: &mut Frame, area: Rect, battle_log: &[BattleResult], theme: &crate::theme::Theme) {
    let block = Block::default()
        .title(Span::styled(
            " ⚔️ Battles ",
            Style::default().fg(theme.error),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    if battle_log.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " No battles yet.",
            Style::default().fg(theme.muted),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Press 'b' to challenge!",
            Style::default().fg(theme.muted),
        )));
    } else {
        // Show last few battles
        let start = battle_log.len().saturating_sub(5);
        for result in &battle_log[start..] {
            lines.push(Line::from(vec![
                Span::styled("🏆 ", Style::default()),
                Span::styled(
                    &result.winner,
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" beat ", Style::default().fg(theme.muted)),
                Span::styled(
                    &result.loser,
                    Style::default().fg(theme.error),
                ),
                Span::styled(
                    format!(" ({}R)", result.rounds.len()),
                    Style::default().fg(theme.muted),
                ),
            ]));

            // Show key rounds
            if let Some(last) = result.rounds.last() {
                lines.push(Line::from(Span::styled(
                    format!("  └ {}", last.message),
                    Style::default().fg(theme.muted),
                )));
            }
            lines.push(Line::from(""));
        }
    }

    let widget = Paragraph::new(lines);
    frame.render_widget(widget, inner);
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

fn creature_element_icon(species: &str) -> &'static str {
    crate::creatures::registry::get_creature_def(species)
        .map(|d| d.element.icon())
        .unwrap_or("❓")
}
