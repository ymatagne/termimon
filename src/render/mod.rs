//! Rendering engine — converts sprite data to terminal output
//!
//! Two renderers:
//!   - **halfblock**: Full-color pixel art using ▀▄ half-block characters with
//!     24-bit ANSI true-color escape sequences. A 16×16 sprite becomes
//!     16 columns × 8 rows of terminal characters.
//!   - **text**: Minimal fallback that uses emoji + name + state text only.
//!     Works in any terminal, no true-color needed.

pub mod halfblock;
pub mod text;


/// Rendering mode, chosen by config or terminal capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// Full half-block pixel art rendering (requires true-color terminal)
    HalfBlock,
    /// Text-only fallback (emoji + name + state)
    Text,
}

impl RenderMode {
    /// Auto-detect the best rendering mode for the current terminal.
    pub fn auto_detect() -> Self {
        // Check for COLORTERM=truecolor or 24bit
        if let Ok(ct) = std::env::var("COLORTERM") {
            if ct == "truecolor" || ct == "24bit" {
                return RenderMode::HalfBlock;
            }
        }
        // Most modern terminals support true-color even without COLORTERM
        // Default to halfblock, fallback on explicit request
        RenderMode::HalfBlock
    }
}
