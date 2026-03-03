<div align="center">

# 🎮 TermiMon

**Your AI coding agents deserve pets.**

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/ymatagne/termimon/actions/workflows/ci.yml/badge.svg)](https://github.com/ymatagne/termimon/actions/workflows/ci.yml)
[![GitHub release](https://img.shields.io/github/v/release/ymatagne/termimon)](https://github.com/ymatagne/termimon/releases)
![macOS](https://img.shields.io/badge/macOS-supported-brightgreen)
![Linux](https://img.shields.io/badge/Linux-supported-brightgreen)
[![v0.4.0](https://img.shields.io/badge/version-0.4.0-7c5cff)](https://github.com/ymatagne/termimon/releases/tag/v0.4.0)

<br>

*TermiMon turns your AI coding agents into animated pixel art creatures that live in your tmux.*
*They breathe, bounce, evolve, and react to what your agents are doing — in real time.*

<br>

[Install](#-install) · [Quick Start](#-quick-start) · [Creatures](#-creatures) · [Evolution](#-evolution) · [Commands](#-commands) · [Docs](https://ymatagne.github.io/termimon)

</div>

---

## 📸 Screenshots

> _Screenshots coming soon — run `termimon dash` to see it live!_

<!--
<div align="center">
<img src="docs/assets/dashboard.png" width="700" alt="TermiMon Dashboard">
<br><br>
<img src="docs/assets/statusbar.png" width="700" alt="TermiMon Status Bar">
</div>
-->

## ✨ What is this?

You're running 3 Claude Code sessions, an aider, and a Copilot. They're all in tmux panes. You have no idea which one is stuck, which one is burning tokens, or which one just finished.

TermiMon fixes that. Each agent gets a pixel art creature that **reacts in real time**:
- Writing code? The creature's typing furiously 🔥
- Thinking? Thought bubbles float up 💭
- Idle for 5 minutes? It falls asleep 💤
- Hit an error? It shakes and flashes red ❌
- Finished a task? It celebrates 🎉

Creatures live in your **tmux status bar** and in a full **interactive dashboard**. They earn XP from agent activity and **evolve through 3 stages**.

## ✨ Features

- 🐾 **6 Creature Species** — Each AI agent type gets its own pixel art companion
- 🎨 **Pixel Art Sprites** — 16×16 half-block Unicode art (▀▄), no images needed
- 📊 **tmux Status Bar** — Creatures appear in your tmux status-right, always visible
- 🖥️ **Interactive Dashboard** — Full TUI with sprites, stats, activity feed, sorting & filtering
- 🏆 **3-Stage Evolution** — Creatures evolve at 100 XP and 500 XP with new names & sprites
- 🎬 **Animated Sprites** — Breathing, bouncing, typing, sleeping — they feel alive
- 🔌 **Auto-Detection** — Finds Claude Code, Codex, aider, Cursor, Copilot, Continue, and Cline
- 💰 **Token & Cost Tracking** — Parses Claude JSONL transcripts for per-agent spend
- 📋 **Activity Feed** — File reads/writes, commands, errors, thinking — all tracked per agent
- 🔀 **Pane Switching** — Press Enter in the dashboard to jump to any agent's tmux pane
- ⚡ **Lightweight** — Pure Rust, single binary, <1% CPU, <20MB RAM
- 🔧 **Daemon + CLI** — Background daemon with Unix socket IPC, clean CLI interface

## 🚀 Install

**One-liner (macOS & Linux):**

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
cd termimon && cargo build --release
# binary at target/release/termimon
```

## 🏁 Quick Start

```bash
# 1. Start the daemon (inside tmux)
termimon start

# 2. Your agents are already detected — check them
termimon status

# 3. Open the interactive dashboard
termimon dash

# 4. Press Enter on any creature to jump to that agent's pane
```

That's it. Your creatures are alive. Go write some code and watch them react.

## 🐾 Creatures

Six species, each tied to an AI agent type:

| Creature | Type | Agent | Vibe |
|----------|------|-------|------|
| **Embercli** | 🔥 Fire | Claude Code | A fiery CLI spirit. Burns bright when code flows fast. |
| **Voltprompt** | ⚡ Electric | Codex | Sparks fly when tokens flow. Crackles with static charge. |
| **Shelloise** | 💧 Water | aider | Calm shell dweller. Patient, deeply connected to pipes. |
| **Rustacean** | 🦀 Rust | Cursor | Forged in metal, compiles on first try. Memory-safe. |
| **Pythorn** | 🌿 Grass | Copilot / Continue / Cline | Grows steadily, wraps around problems. |
| **Gitbat** | 🦇 Dark | Generic / Unknown | Lurks in repos, echolocates through commit history. |

## 🏆 Evolution

Creatures evolve as their agents work. XP comes from file writes, commands, git operations, token usage, and task completions.

| Species | Stage 1 (0 XP) | Stage 2 (100 XP) | Stage 3 (500 XP) |
|---------|----------------|-------------------|-------------------|
| 🔥 Fire | Embercli | Blazecli | Infernocli |
| ⚡ Electric | Voltprompt | Sparkprompt | Thunderprompt |
| 💧 Water | Shelloise | Streamoise | Tidaloise |
| 🦀 Rust | Rustacean | Chromacean | Titanacean |
| 🌿 Grass | Pythorn | Junipythorn | Sequoiathorn |
| 🦇 Dark | Gitbat | Repobat | Monobat |

Each stage gets a new sprite with more detail, special effects, and unique idle animations.

## 📖 Commands

| Command | Description |
|---------|-------------|
| `termimon start` | Start the background daemon (must be in tmux) |
| `termimon stop` | Stop the daemon |
| `termimon status` | Show detected agents, creatures, XP, and stats |
| `termimon dash` | Interactive dashboard with pixel art, activity feed, sorting & filtering |

### Dashboard Controls

| Key | Action |
|-----|--------|
| **↑/↓** or **j/k** | Navigate between agents |
| **Enter** | Switch to agent's tmux pane (without leaving dashboard) |
| **Tab** | Cycle dashboard panels |
| **s** | Sort agents |
| **f** | Filter agents |
| **q** | Quit |

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

## ⚔️ Team Mode (v0.4.0)

Connect multiple TermiMon instances over the network! See each other's creatures, and battle them.

```bash
# Host a team session
termimon team host              # listens on port 4662
termimon team host --port 5000  # custom port

# Join a team
termimon team join 192.168.1.50:4662

# Check status
termimon team status

# Leave
termimon team leave
```

**In the dashboard**, press `t` to toggle the team view, and `b` to challenge a peer's creature to battle.

**Battle system**: Stats are derived from real agent metrics — ATK from lines/dollar, DEF from build success rate, SPD from commits/hour, HP from XP. Element type advantages apply (🔥 > 🌿 > 💧 > 🔥).

**Config** (`~/.termimon/config.toml`):
```toml
[team]
name = "yan"       # your display name
port = 4662        # hosting port
auto_host = false  # auto-host on daemon start
```

## 🗺️ Roadmap

- [x] Core daemon + tmux agent detection
- [x] 6 creatures with pixel art sprites & 3-stage evolution
- [x] Per-agent token & cost tracking (Claude Code JSONL)
- [x] Activity feed (file I/O, commands, errors, thinking)
- [x] Interactive dashboard with sort, filter, pane switching
- [x] Animated sprites (breathing, bounce, typing, sleeping)
- [x] tmux status bar widget
- [ ] Theme packs (retro, neon, pastel)
- [ ] Sound effects (terminal bell patterns)
- [x] Creature battles (compare agent productivity)
- [x] Team mode (TCP-based peer networking)
- [ ] Plugin system for custom agent detectors
- [ ] Web dashboard export

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

Especially wanted:
- 🎨 New creature sprites and evolution designs
- 🔌 New agent detectors
- 🌈 Theme packs

## 📄 License

[MIT](LICENSE) © 2026 Yan Matagne
