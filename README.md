<div align="center">

# 🎮 TermiMon

**Your AI coding agents deserve pets.**

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/ymatagne/termimon/actions/workflows/ci.yml/badge.svg)](https://github.com/ymatagne/termimon/actions/workflows/ci.yml)
[![GitHub release](https://img.shields.io/github/v/release/ymatagne/termimon)](https://github.com/ymatagne/termimon/releases)
![macOS](https://img.shields.io/badge/macOS-supported-brightgreen)
![Linux](https://img.shields.io/badge/Linux-supported-brightgreen)
[![v0.5.0](https://img.shields.io/badge/version-0.5.0-7c5cff)](https://github.com/ymatagne/termimon/releases/tag/v0.5.0)

<br>

*TermiMon turns your AI coding agents into animated pixel art creatures that live in your tmux.*
*They breathe, bounce, evolve, and react to what your agents are doing — in real time.*

<br>

[Install](#-install) · [Quick Start](#-quick-start) · [Creatures](#-creatures) · [Evolution](#-evolution) · [Commands](#-commands) · [Docs](https://ymatagne.github.io/termimon)

</div>

---

## 📸 Screenshots

> _Screenshots coming soon — run `termimon dash` to see it live!_

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

- 🐾 **12 Creature Species** — Each AI agent type gets its own pixel art companion
- 🎨 **Pixel Art Sprites** — 16×16 half-block Unicode art (▀▄), no images needed
- 📊 **tmux Status Bar** — Creatures appear in your tmux status-right, always visible
- 🖥️ **Interactive Dashboard** — Full TUI with sprites, stats, activity feed, sorting & filtering
- 🏆 **3-Stage Evolution** — Creatures evolve at 100 XP and 500 XP with new names & sprites
- 🎬 **Animated Sprites** — Breathing, bouncing, typing, sleeping — they feel alive
- 🔌 **Auto-Detection** — Finds Claude Code, Codex, aider, Cursor, Copilot, Continue, Cline, GPT, Docker, and more
- 💰 **Token & Cost Tracking** — Parses Claude JSONL transcripts for per-agent spend
- 📋 **Activity Feed** — File reads/writes, commands, errors, thinking — all tracked per agent
- 🔀 **Pane Switching** — Press Enter in the dashboard to jump to any agent's tmux pane
- ⚡ **Lightweight** — Pure Rust, single binary, <1% CPU, <20MB RAM
- 🔧 **Daemon + CLI** — Background daemon with Unix socket IPC, clean CLI interface
- 🎨 **Theme Packs** — Default, Retro CRT, Neon Cyberpunk, Pastel themes
- 🔊 **Sound Effects** — Terminal bell patterns for evolution, battle wins, XP milestones
- 🔍 **mDNS Auto-Discovery** — Zero-config team mode, just `termimon team auto`
- ⚔️ **Spectator Battles** — Watch battles play out across all connected peers
- 📖 **Pokédex** — Browse your creature collection with lifetime stats
- 🔌 **Plugin System** — Custom creatures via TOML files in `~/.termimon/plugins/`
- 🤖 **GitHub Action** — Post creature stats as PR comments

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

# 4. Browse your creature collection
termimon pokedex

# 5. Press Enter on any creature to jump to that agent's pane
```

That's it. Your creatures are alive. Go write some code and watch them react.

## 🐾 Creatures

Twelve species, each tied to an AI agent or tool type:

| Creature | Type | Agent | Vibe |
|----------|------|-------|------|
| **Embercli** | 🔥 Fire | Claude Code | A fiery CLI spirit. Burns bright when code flows fast. |
| **Voltprompt** | ⚡ Electric | Codex | Sparks fly when tokens flow. Crackles with static charge. |
| **Shelloise** | 💧 Water | aider | Calm shell dweller. Patient, deeply connected to pipes. |
| **Rustacean** | 🦀 Rust | Cursor | Forged in metal, compiles on first try. Memory-safe. |
| **Pythorn** | 🌿 Grass | Copilot / Continue | Grows steadily, wraps around problems. |
| **Gitbat** | 🦇 Dark | Generic / Unknown | Lurks in repos, echolocates through commit history. |
| **Neuromorph** | 🧠 Psychic | GPT / OpenAI | A brain creature pulsing with neural energy. |
| **Dockersaur** | ⚙️ Steel | Docker / Containers | A metallic dinosaur orchestrating containers. |
| **Termignite** | 💥 Fire/Dark | vim / tmux | Dark flame creature forged by terminal power users. |
| **Pixelbyte** | 🎮 Digital | Game Dev | Glitchy digital creature from game engines. |
| **Cloudwisp** | ☁️ Air | Railway / Fly / Vercel | Floating cloud that drifts through deploy pipelines. |
| **Dataslime** | 🟢 Poison | Database / Data | Green slime oozing through queries and schemas. |

## 🏆 Evolution

Creatures evolve as their agents work. XP comes from file writes, commands, git operations, token usage, and task completions.

| Species | Stage 1 (0 XP) | Stage 2 (100 XP) | Stage 3 (500 XP) |
|---------|----------------|-------------------|-------------------|
| 🔥 Fire | Embercli | Blazecli | Infernocli |
| ⚡ Electric | Voltprompt | Sparkprompt | Thunderprompt |
| 💧 Water | Shelloise | Torrentoise | Tsunamoise |
| 🦀 Rust | Rustacean | Ferrocrab | Oxidragon |
| 🌿 Grass | Pythorn | Vineconda | Thornviper |
| 🦇 Dark | Gitbat | Commitwing | Mergefiend |
| 🧠 Psychic | Neuromorph | Synaptrix | Omnimind |
| ⚙️ Steel | Dockersaur | Composaurus | Kubernox |
| 💥 Fire/Dark | Termignite | Blazeshell | Infernotty |
| 🎮 Digital | Pixelbyte | Voxelcore | Renderex |
| ☁️ Air | Cloudwisp | Stratolift | Cumulonimbus |
| 🟢 Poison | Dataslime | Queryblob | Schemazoid |

## 📖 Commands

| Command | Description |
|---------|-------------|
| `termimon start` | Start the background daemon (must be in tmux) |
| `termimon stop` | Stop the daemon |
| `termimon status` | Show detected agents, creatures, XP, and stats |
| `termimon dash` | Interactive dashboard with pixel art, activity feed, sorting & filtering |
| `termimon pokedex` | Browse your creature collection and lifetime stats |
| `termimon theme list` | List available color themes |
| `termimon theme set <name>` | Set the active theme (default, retro, neon, pastel) |
| `termimon team auto` | Auto-discover peers on local network via mDNS |
| `termimon team host` | Host a team session |
| `termimon team join <addr>` | Join a team session |
| `termimon team status` | Show team connection status |

### Dashboard Controls

| Key | Action |
|-----|--------|
| **↑/↓** or **j/k** | Navigate between agents |
| **Enter** | Switch to agent's tmux pane |
| **Tab** | Cycle dashboard panels |
| **s** | Sort agents |
| **f** | Filter agents |
| **t** | Toggle team view |
| **b** | Challenge a peer to battle |
| **q** | Quit |

## 🎨 Theme Packs (v0.5.0)

Customize your TermiMon experience with built-in themes:

- **default** — Classic dark theme
- **retro** — Green-on-black CRT terminal feel
- **neon** — Cyberpunk pink/cyan neon aesthetic
- **pastel** — Soft, easy-on-the-eyes colors

```bash
termimon theme list          # see all themes
termimon theme set neon      # apply a theme
```

Themes affect dashboard colors, status bar appearance, and sprite display.

## 🔊 Sound Effects (v0.5.0)

Terminal bell patterns for key events:

- **Evolution**: Ascending 3-bell pattern 🎵
- **Battle Win**: Victory fanfare (5 bells) 🏆
- **XP Milestone** (every 100): Single bell 🔔

Enable in config:
```toml
[notifications]
sounds = true
```

## 🔍 mDNS Auto-Discovery (v0.5.0)

Zero-config team mode — just works on local WiFi/LAN:

```bash
termimon team auto  # broadcasts + discovers peers automatically
```

No need to know IP addresses. TermiMon finds other instances on your network and connects automatically. Falls back to manual mode if mDNS isn't available.

## 🔌 Plugin System (v0.5.0)

Create custom creatures by dropping TOML files in `~/.termimon/plugins/`:

```toml
name = "dockersaur"
element = "steel"
default_agent = "docker"
description = "A metallic beast that orchestrates containers"
evolution_names = ["Dockersaur", "Composaurus", "Kubernox"]
detect_process = ["docker", "docker-compose"]
```

Plugins are loaded on daemon startup. Custom agent detection from process names.

## 🤖 GitHub Action (v0.5.0)

Add creature stats to your PRs:

```yaml
- uses: ymatagne/termimon/.github/actions/termimon-comment@main
  with:
    github-token: ${{ secrets.GITHUB_TOKEN }}
```

Posts comments like: "🔥 Infernocli (Stage 3) helped build this PR — 1,500 XP earned, 42 files changed"

## ⚔️ Team Mode

Connect multiple TermiMon instances over the network! See each other's creatures, and battle them.

```bash
# Auto-discover (recommended)
termimon team auto

# Or manual
termimon team host              # listens on port 4662
termimon team join 192.168.1.50:4662

# Check status
termimon team status
```

**Battle system**: Stats derived from real agent metrics — ATK from lines/dollar, DEF from build success rate, SPD from commits/hour, HP from XP. Element type advantages apply. Battles are broadcast to all spectators in team mode.

## ⚙️ Configuration

Config lives at `~/.termimon/config.toml`:

```toml
[general]
poll_interval_ms = 2000
animation_fps = 4
theme = "default"  # default, retro, neon, pastel

[statusbar]
position = "right"
max_creatures = 5
show_xp = false
show_state = true

[notifications]
evolution = true
sounds = true
system_notify = true

[team]
name = "yan"
port = 4662
auto_host = false

[creatures.assignments]
claude = "embercli"
codex = "voltprompt"
aider = "shelloise"
```

## 🗺️ Roadmap

- [x] Core daemon + tmux agent detection
- [x] 12 creatures with pixel art sprites & 3-stage evolution
- [x] Per-agent token & cost tracking (Claude Code JSONL)
- [x] Activity feed (file I/O, commands, errors, thinking)
- [x] Interactive dashboard with sort, filter, pane switching
- [x] Animated sprites (breathing, bounce, typing, sleeping)
- [x] tmux status bar widget
- [x] Theme packs (retro, neon, pastel)
- [x] Sound effects (terminal bell patterns)
- [x] Creature battles with spectator mode
- [x] Team mode (TCP + mDNS auto-discovery)
- [x] Plugin system for custom creatures
- [x] Pokédex creature collection browser
- [x] GitHub Action for PR comments
- [ ] Web dashboard export
- [ ] Creature trading between peers

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

Especially wanted:
- 🎨 New creature sprites and evolution designs
- 🔌 New agent detectors and plugins
- 🌈 Theme packs

## 📄 License

[MIT](LICENSE) © 2026 Yan Matagne
