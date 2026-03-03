//! Half-block character renderer 🎨
//!
//! Converts 16×16 sprite frames into terminal strings using Unicode half-block
//! characters (▀ U+2580, ▄ U+2584) with 24-bit ANSI true-color escape sequences.
//!
//! A 16×16 sprite renders as 16 columns × 8 rows of terminal characters,
//! because each terminal cell encodes TWO vertical pixels:
//!
//!   ▀  = top pixel as foreground color, bottom as background
//!   ▄  = bottom pixel as foreground color (top as background)
//!
//! Transparency handling:
//!   - Both transparent → space ' '
//!   - Top only         → ▀ with fg = top color
//!   - Bottom only      → ▄ with fg = bottom color
//!   - Both opaque      → ▀ with fg = top, bg = bottom

use crate::creatures::{Color, SpriteFrame};

// ── Core rendering ───────────────────────────────────────────────────────────

/// Render a 16×16 sprite as a vector of 8 terminal lines.
pub fn render_sprite(sprite: &SpriteFrame) -> Vec<String> {
    let mut lines = Vec::with_capacity(8);

    for y in (0..16).step_by(2) {
        let top_row = &sprite[y];
        let bot_row = &sprite[y + 1];
        lines.push(render_row_pair(top_row, bot_row));
    }

    lines
}

/// Render one pair of pixel rows (top + bottom) as a single terminal line.
fn render_row_pair(top: &[Color; 16], bottom: &[Color; 16]) -> String {
    let mut out = String::with_capacity(320); // generous pre-alloc for escape codes
    let mut last_was_escape = false;

    for x in 0..16 {
        let t = &top[x];
        let b = &bottom[x];

        match (t.is_transparent(), b.is_transparent()) {
            // Both transparent → empty space
            (true, true) => {
                if last_was_escape {
                    out.push_str("\x1b[0m");
                    last_was_escape = false;
                }
                out.push(' ');
            }

            // Only top pixel visible → upper half block with fg color
            (false, true) => {
                out.push_str(&format!(
                    "\x1b[38;2;{};{};{}m▀",
                    t.r, t.g, t.b
                ));
                last_was_escape = true;
            }

            // Only bottom pixel visible → lower half block with fg color
            (true, false) => {
                out.push_str(&format!(
                    "\x1b[38;2;{};{};{}m▄",
                    b.r, b.g, b.b
                ));
                last_was_escape = true;
            }

            // Both opaque → upper half block, fg=top bg=bottom
            (false, false) => {
                if t.r == b.r && t.g == b.g && t.b == b.b {
                    // Same color: use full block (█) or just colored space
                    out.push_str(&format!(
                        "\x1b[48;2;{};{};{}m ",
                        t.r, t.g, t.b
                    ));
                } else {
                    out.push_str(&format!(
                        "\x1b[38;2;{};{};{};48;2;{};{};{}m▀",
                        t.r, t.g, t.b,
                        b.r, b.g, b.b
                    ));
                }
                last_was_escape = true;
            }
        }
    }

    // Always reset at end of line
    if last_was_escape {
        out.push_str("\x1b[0m");
    }

    out
}

// ── Boxed rendering (with border) ────────────────────────────────────────────

/// Render a sprite inside a cute Unicode box frame with a title.
pub fn render_sprite_boxed(sprite: &SpriteFrame, title: &str) -> Vec<String> {
    let sprite_lines = render_sprite(sprite);
    let mut lines = Vec::with_capacity(10);

    // Box width = 16 (sprite) + 2 (borders) = 18 inner
    let box_inner: usize = 18;

    // Title centered in top border
    let title_display = if title.len() > 14 { &title[..14] } else { title };
    let pad_total = box_inner.saturating_sub(title_display.len() + 2);
    let pad_left = pad_total / 2;
    let pad_right = pad_total - pad_left;

    lines.push(format!(
        "╭{}─ {} {}╮",
        "─".repeat(pad_left),
        title_display,
        "─".repeat(pad_right),
    ));

    for line in &sprite_lines {
        lines.push(format!("│{line}│"));
    }

    lines.push(format!("╰{}╯", "─".repeat(box_inner)));

    lines
}

// ── Status bar compact rendering ─────────────────────────────────────────────

/// Generate a compact status-bar string: `🔥Embercli[typing]`
pub fn render_status_compact(
    icon: &str,
    name: &str,
    state: &str,
    show_xp: bool,
    xp: u64,
    level: u32,
) -> String {
    if show_xp {
        format!("{icon}{name} Lv.{level}[{state}]")
    } else {
        format!("{icon}{name}[{state}]")
    }
}

// ── XP bar rendering ─────────────────────────────────────────────────────────

/// Render a progress bar for XP: `[████░░░░░░] 65%`
pub fn render_xp_bar(progress: f64, width: usize) -> String {
    let filled = (progress * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    let pct = (progress * 100.0).round() as u32;

    format!(
        "[{}{}] {}%",
        "█".repeat(filled),
        "░".repeat(empty),
        pct,
    )
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::sprites;

    #[test]
    fn render_produces_8_lines() {
        let lines = render_sprite(&sprites::EMBERCLI_IDLE_1);
        assert_eq!(lines.len(), 8, "16×16 sprite should produce 8 terminal lines");
    }

    #[test]
    fn render_all_creatures() {
        for (name, sprite) in sprites::all_idle_sprites() {
            let lines = render_sprite(sprite);
            assert_eq!(lines.len(), 8, "{name} render should be 8 lines");
        }
    }

    #[test]
    fn boxed_render_adds_borders() {
        let lines = render_sprite_boxed(&sprites::EMBERCLI_IDLE_1, "Embercli");
        assert_eq!(lines.len(), 10, "boxed = 8 sprite + 2 border lines");
        assert!(lines[0].contains("Embercli"));
        assert!(lines[0].starts_with('╭'));
        assert!(lines[9].starts_with('╰'));
    }

    #[test]
    fn xp_bar_rendering() {
        assert_eq!(render_xp_bar(0.0, 10), "[░░░░░░░░░░] 0%");
        assert_eq!(render_xp_bar(0.5, 10), "[█████░░░░░] 50%");
        assert_eq!(render_xp_bar(1.0, 10), "[██████████] 100%");
    }

    #[test]
    fn status_compact() {
        let s = render_status_compact("🔥", "Embercli", "typing", false, 0, 1);
        assert_eq!(s, "🔥Embercli[typing]");

        let s = render_status_compact("⚡", "Volt", "idle", true, 50, 7);
        assert_eq!(s, "⚡Volt Lv.7[idle]");
    }

    #[test]
    fn transparency_handling() {
        // A fully transparent sprite should produce only spaces
        let blank = [[crate::creatures::Color::transparent(); 16]; 16];
        let lines = render_sprite(&blank);
        for line in &lines {
            // Should be 16 spaces (no escape codes)
            assert_eq!(line.trim().len(), 0, "blank sprite should be all spaces");
        }
    }
}
