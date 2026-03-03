//! Theme system for TermiMon
//!
//! Themes affect dashboard colors, sprite tinting, and status bar appearance.

use ratatui::style::Color;
/// Available theme names.
pub const THEME_NAMES: &[&str] = &["default", "retro", "neon", "pastel"];

/// A complete color theme.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub description: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub highlight: Color,
    pub muted: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub border: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub status_bg: Color,
    pub status_fg: Color,
}

/// Default dark theme (current look).
pub const THEME_DEFAULT: Theme = Theme {
    name: "default",
    description: "Classic dark theme",
    bg: Color::Reset,
    fg: Color::White,
    accent: Color::Cyan,
    highlight: Color::Yellow,
    muted: Color::DarkGray,
    success: Color::Green,
    error: Color::Red,
    warning: Color::Yellow,
    border: Color::DarkGray,
    selection_bg: Color::Cyan,
    selection_fg: Color::Black,
    status_bg: Color::Reset,
    status_fg: Color::White,
};

/// Retro green-on-black CRT theme.
pub const THEME_RETRO: Theme = Theme {
    name: "retro",
    description: "Green-on-black CRT terminal",
    bg: Color::Black,
    fg: Color::Green,
    accent: Color::LightGreen,
    highlight: Color::LightGreen,
    muted: Color::Rgb(0, 100, 0),
    success: Color::LightGreen,
    error: Color::Rgb(200, 50, 0),
    warning: Color::Rgb(200, 200, 0),
    border: Color::Rgb(0, 100, 0),
    selection_bg: Color::Green,
    selection_fg: Color::Black,
    status_bg: Color::Black,
    status_fg: Color::Green,
};

/// Neon cyberpunk theme (pink/cyan).
pub const THEME_NEON: Theme = Theme {
    name: "neon",
    description: "Cyberpunk neon (pink/cyan)",
    bg: Color::Rgb(15, 10, 30),
    fg: Color::Rgb(0, 255, 255),
    accent: Color::Rgb(255, 0, 200),
    highlight: Color::Rgb(255, 255, 0),
    muted: Color::Rgb(80, 80, 120),
    success: Color::Rgb(0, 255, 150),
    error: Color::Rgb(255, 50, 80),
    warning: Color::Rgb(255, 200, 0),
    border: Color::Rgb(100, 0, 150),
    selection_bg: Color::Rgb(255, 0, 200),
    selection_fg: Color::Black,
    status_bg: Color::Rgb(15, 10, 30),
    status_fg: Color::Rgb(0, 255, 255),
};

/// Pastel soft colors theme.
pub const THEME_PASTEL: Theme = Theme {
    name: "pastel",
    description: "Soft pastel colors",
    bg: Color::Rgb(40, 40, 50),
    fg: Color::Rgb(220, 210, 230),
    accent: Color::Rgb(150, 180, 255),
    highlight: Color::Rgb(255, 220, 150),
    muted: Color::Rgb(120, 110, 130),
    success: Color::Rgb(150, 230, 150),
    error: Color::Rgb(255, 150, 150),
    warning: Color::Rgb(255, 220, 150),
    border: Color::Rgb(100, 90, 120),
    selection_bg: Color::Rgb(150, 180, 255),
    selection_fg: Color::Rgb(30, 30, 40),
    status_bg: Color::Rgb(40, 40, 50),
    status_fg: Color::Rgb(220, 210, 230),
};

/// Get theme by name.
pub fn get_theme(name: &str) -> &'static Theme {
    match name {
        "retro" => &THEME_RETRO,
        "neon" => &THEME_NEON,
        "pastel" => &THEME_PASTEL,
        _ => &THEME_DEFAULT,
    }
}

/// List all themes with descriptions.
pub fn list_themes() {
    println!("\n🎨 Available Themes:\n");
    let all = [&THEME_DEFAULT, &THEME_RETRO, &THEME_NEON, &THEME_PASTEL];
    for theme in all {
        println!("  {} — {}", theme.name, theme.description);
    }
    println!("\nSet with: termimon theme set <name>");
    println!();
}

/// Set the theme in config file.
pub fn set_theme(name: &str) -> anyhow::Result<()> {
    if !THEME_NAMES.contains(&name) {
        anyhow::bail!("Unknown theme '{}'. Available: {}", name, THEME_NAMES.join(", "));
    }
    let mut config = crate::config::load();
    config.general.theme = name.to_string();
    crate::config::save(&config)?;
    println!("🎨 Theme set to '{name}'");
    Ok(())
}
