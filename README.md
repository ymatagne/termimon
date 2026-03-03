<div align="center">

# 🎮 TermiMon

**htop for AI agents, but every process is an evolving pixel creature.**

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/ymatagne/termimon/actions/workflows/ci.yml/badge.svg)](https://github.com/ymatagne/termimon/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/termimon.svg)](https://crates.io/crates/termimon)
[![GitHub release](https://img.shields.io/github/v/release/ymatagne/termimon)](https://github.com/ymatagne/termimon/releases)

<br>

*TermiMon turns your AI coding agents into animated pixel art creatures inside tmux.*
*Each agent gets its own creature that reacts in real-time — typing, thinking, sleeping, evolving.*

<br>

[Install](#-install) · [Quick Start](#-quick-start) · [Creatures](#-creatures) · [Commands](#-commands) · [Docs](https://ymatagne.github.io/termimon)

</div>

---

## 📸 Screenshots

> _Screenshots coming soon — run `termimon dash` to see it live!_

<!--
<div align="center">
<img src="docs/assets/dashboard.png" width="700" alt="TermiMon Dashboard">
</div>
-->

## ✨ Features

- 🐾 **Living Creatures** — Each AI agent gets an animated pixel art companion
- 🎨 **Pixel Art Rendering** — 16×16 sprites using half-block Unicode (▀▄)
- ⚡ **Lightweight** — Pure Rust single binary, <1% CPU, <20MB RAM
- 🔌 **Auto-Detection** — Finds Claude Code, Codex, aider, Cursor, Copilot, Continue
- 📊 **Resource Tracking** — Per-agent CPU, memory, and token cost monitoring
- 💰 **Token Costs** — Parses Claude Code JSONL transcripts for spend tracking
- 📋 **Activity Feed** — File reads/writes, commands, git operations per agent
- 🏆 **XP & Evolution** — Creatures evolve at 100 and 500 XP from agent activity
- 🎬 **Sprite Animations** — Breathing idle, fast typing, dimmed sleeping states
- 🔀 **Agent Switching** — Press Enter in dashboard to jump to agent's tmux pane
- 📁 **Stable Identity** — Agents identified by project directory across sessions
- 💾 **Daily Stats** — Persistent stats in `~/.termimon/stats/`

## 🚀 Install

**One-liner (macOS/Linux):**

```bash
curl -fsSL https://raw.githubusercontent.com/ymatagne/termimon/main/install.sh | sh
```

**Homebrew:**

```bash
brew tap ymatagne/termimon && brew install termimon
```

**Cargo:**

```bash
cargo install termimon
```

**From source:**

```bash
git clone https://github.com/ymatagne/termimon.git
cd termimon
cargo build --release
# binary at target/release/termimon
```

## 🏁 Quick Start

```bash
# Start the daemon (inside tmux)
termimon start

# Check detected agents
termimon status

# Open the interactive dashboard
termimon dash

# Browse your creature collection
termimon pokedex
```

TermiMon automatically detects AI coding agents running in your tmux panes and assigns each one a unique creature.

## 🐾 Creatures

| Creature | Type | Agent | |
|----------|------|-------|-|
| **Embercli** | 🔥 Fire | Claude Code | A fiery command-line spirit |
| **Voltprompt** | ⚡ Electric | Codex | Sparks fly when tokens flow |
| **Shelloise** | 💧 Water | aider | Patient, deeply connected to pipes |
| **Rustacean** | 🦀 Steel | Cursor | Forged in metal, compiles on first try |
| **Pythorn** | 🌿 Grass | Copilot / Continue | Grows steadily, wraps around problems |
| **Gitbat** | 🦇 Dark | Generic / Unknown | Lurks in repos, sees in the dark |

Each creature has multiple animation states (idle, typing, reading, thinking, sleeping, celebrating, error) and evolves through **3 stages** as it gains XP.

## 📖 Commands

| Command | Description |
|---------|-------------|
| `termimon start` | Start the background daemon |
| `termimon stop` | Stop the daemon |
| `termimon status` | Show detected agents and their creatures |
| `termimon dash` | Interactive dashboard with pixel art, stats, activity feed |
| `termimon pokedex` | View your creature collection and evolution progress |
| `termimon switch` | Switch to an agent's tmux pane |

### Dashboard Controls

- **↑/↓** or **j/k** — Navigate between agents
- **Enter** — Switch to selected agent's tmux pane
- **q** — Quit dashboard

## ⚙️ Configuration

Config lives at `~/.termimon/config.toml`:

```toml
[general]
poll_interval_ms = 2000
animation_fps = 4
theme = "default"

[statusbar]
position = "right"
max_creatures = 5
show_xp = false
show_state = true

[creatures.assignments]
claude = "embercli"
codex = "voltprompt"
aider = "shelloise"
cursor = "rustacean"
copilot = "pythorn"
```

## 🗺️ Roadmap

- [x] Core daemon + tmux detection
- [x] 6 creatures with pixel art sprites
- [x] XP system and 3-stage evolution
- [x] Per-agent CPU/memory tracking
- [x] Token cost tracking (Claude Code)
- [x] Activity feed
- [x] Agent switching from dashboard
- [x] Sprite animations
- [ ] tmux status bar widget
- [ ] Theme packs (retro, neon, pastel)
- [ ] Sound effects (terminal bell patterns)
- [ ] Creature battles (compare agent productivity)
- [ ] Team mode (shared creature collection)
- [ ] Plugin system for custom agent detectors
- [ ] Web dashboard export

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Especially wanted:
- 🎨 New creature designs (pixel art sprites)
- 🔌 New agent detectors
- 🌈 Theme packs

## 📄 License

[MIT](LICENSE) © 2026 Yan Matagne
