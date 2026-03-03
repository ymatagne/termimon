//! Sound effects via terminal bell patterns
//!
//! Uses \x07 (BEL) character sequences with timing patterns
//! to create simple sound effects for key events.

use std::io::Write;

/// Play evolution sound: ascending 3-bell pattern.
pub fn play_evolution() {
    if !sounds_enabled() { return; }
    std::thread::spawn(|| {
        let mut out = std::io::stderr();
        for _ in 0..3 {
            let _ = out.write_all(b"\x07");
            let _ = out.flush();
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
    });
}

/// Play battle win: victory fanfare (5 bells).
pub fn play_battle_win() {
    if !sounds_enabled() { return; }
    std::thread::spawn(|| {
        let mut out = std::io::stderr();
        // Short-short-short-pause-long-long
        for i in 0..5 {
            let _ = out.write_all(b"\x07");
            let _ = out.flush();
            let delay = if i == 2 { 300 } else { 120 };
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
    });
}

/// Play XP milestone: single bell.
pub fn play_xp_milestone() {
    if !sounds_enabled() { return; }
    let _ = std::io::stderr().write_all(b"\x07");
    let _ = std::io::stderr().flush();
}

/// Check if sounds are enabled in config.
fn sounds_enabled() -> bool {
    let cfg = crate::config::load();
    cfg.notifications.sounds
}
