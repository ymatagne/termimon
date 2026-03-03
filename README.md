# 🎮 TermiMon

**Your AI agents, alive in the terminal.**

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/ymatagne/termimon/actions/workflows/ci.yml/badge.svg)](https://github.com/ymatagne/termimon/actions/workflows/ci.yml)

TermiMon turns your AI coding agents into animated pixel art creatures inside tmux. Each agent gets its own creature that reacts in real-time — typing when writing code, thinking when planning, sleeping when idle, and evolving as it completes work.

---

## ✨ Features

- 🐾 **Living Creatures** — AI agents rendered as animated pixel art companions
- 🎨 **Beautiful Pixel Art** — 16×16 sprites with half-block rendering (▀▄)
- ⚡ **Lightweight** — Pure Rust, single binary, <1% CPU, <20MB RAM
- 🔌 **Agent Detection** — Auto-detects Claude Code, Codex, and aider
- 📊 **tmux Integration** — Status bar + popup dashboard
- 🏆 **XP & Evolution** — Creatures evolve through 3 stages as your agents work
- 🎮 **Interactive** — Pokedex, assignments, and creature management

## 🐾 Creatures

| Creature | Element | Agent | Description |
|----------|---------|-------|-------------|
| **Embercli** 🔥 | Fire | Claude Code | A fiery command-line spirit. Thrives on fast execution and hot loops. |
| **Voltprompt** ⚡ | Electric | Codex | A crackling prompt engineer. Sparks fly when tokens flow. |
| **Shelloise** 💧 | Water | aider | A calm, resilient shell dweller. Patient and deeply connected to pipes. |

Each creature has **9 animation states**: idle, typing, reading, thinking, running, sleeping, celebrating, error, waiting.

Each creature evolves through **3 stages** as it gains XP from agent activity.

## 🚀 Install

**One-liner (macOS/Linux):**

```bash
curl -fsSL https://raw.githubusercontent.com/ymatagne/termimon/main/install.sh | sh
```

**Homebrew:**

```bash
brew tap ymatagne/termimon
brew install termimon
```

**Cargo:**

```bash
cargo install termimon
```

**From source:**

```bash
git clone https://github.com/ymatagne/termimon.git
cd termimon
make install
```

## 🏁 Quick Start

```bash
# Start the daemon (inside a tmux session)
termimon start

# Check what's running
termimon status

# Open the dashboard popup
termimon dash

# See your creature collection
termimon pokedex
```

TermiMon automatically detects Claude Code, Codex, and aider running in your tmux panes and assigns creatures to them.

## 📸 Screenshots

> _Coming soon!_

## 🔧 How It Works

1. **Discovery** — TermiMon polls tmux panes every 2 seconds, inspecting process trees and terminal output
2. **Detection** — Recognizes AI agents by process name and output patterns (also reads Claude Code JSONL transcripts)
3. **Binding** — Each detected agent gets a creature assigned (auto or manual)
4. **Rendering** — Creatures animate in the tmux status bar and popup dashboard using half-block pixel art
5. **Evolution** — Agents earn XP for their creatures by writing files, completing tasks, and fixing errors

## ⚙️ Configuration

Config lives at `~/.termimon/config.toml`:

```toml
[general]
poll_interval_ms = 2000
display_mode = "statusbar"  # statusbar | popup | pane
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
```

## 🤝 Contributing

Contributions welcome! Especially:

- 🎨 New creature designs (pixel art sprites)
- 🔌 New agent detectors
- 🌈 Theme packs

1. Fork the repo
2. Create a feature branch (`git checkout -b feat/amazing-thing`)
3. Commit your changes (`git commit -m 'feat: amazing thing'`)
4. Push to the branch (`git push origin feat/amazing-thing`)
5. Open a Pull Request

## 📄 License

[MIT](LICENSE) © 2026 Yan Matagne
