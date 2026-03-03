<![CDATA[<div align="center">

```
 _____                   _ __  __
|_   _|__ _ __ _ __ ___ (_)  \/  | ___  _ __
  | |/ _ \ '__| '_ ` _ \| | |\/| |/ _ \| '_ \
  | |  __/ |  | | | | | | | |  | | (_) | | | |
  |_|\___|_|  |_| |_| |_|_|_|  |_|\___/|_| |_|
```

### Your AI agents, alive in the terminal.

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/yanmatagne/termimon/actions/workflows/ci.yml/badge.svg)](https://github.com/yanmatagne/termimon/actions/workflows/ci.yml)

</div>

---

## ✨ Features

- 🐾 **Living Creatures** — AI agents rendered as interactive terminal pets
- 🎨 **Beautiful TUI** — Smooth animations powered by Ratatui
- ⚡ **Lightweight** — Pure Rust, minimal resource usage
- 🔌 **Agent Integration** — Monitor your AI agents in real-time
- 🎮 **Interactive** — Feed, train, and evolve your creatures
- 📊 **Status Dashboard** — See agent health, mood, and activity at a glance
- 🌈 **Themeable** — Customize colors and appearance

## 🚀 Quick Install

```bash
cargo install termimon
```

## 🏁 Quick Start

```bash
termimon start
```

## 📸 Screenshots

> _Coming soon!_

## 🐾 Creature Showcase

| Creature | Type | Element | Description |
|----------|------|---------|-------------|
| **Embercli** 🔥 | Fire | CLI | A fiery command-line spirit. Thrives on fast execution and hot loops. |
| **Voltprompt** ⚡ | Electric | Prompt | A crackling prompt engineer. Sparks fly when tokens flow. |
| **Shelloise** 💧 | Water | Shell | A calm, resilient shell dweller. Patient, steady, and deeply connected to pipes. |

## 🔧 How It Works

TermiMon connects to your running AI agents and represents each one as a unique creature in your terminal. Creatures react to agent activity in real-time — they animate when busy, sleep when idle, and evolve as your agents grow.

1. **Discovery** — TermiMon detects running agents via configured endpoints
2. **Binding** — Each agent is matched to a creature based on its profile
3. **Rendering** — The TUI renders creatures with smooth frame-based animation
4. **Interaction** — You can interact with creatures to inspect or control agents

## ⚙️ Configuration

TermiMon looks for config at `~/.config/termimon/config.toml`:

```toml
[display]
fps = 30
theme = "default"

[agents]
poll_interval_ms = 1000

[[agents.sources]]
name = "my-agent"
endpoint = "http://localhost:8080/health"
creature = "embercli"
```

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repo
2. Create a feature branch (`git checkout -b feat/amazing-thing`)
3. Commit your changes (`git commit -m 'feat: amazing thing'`)
4. Push to the branch (`git push origin feat/amazing-thing`)
5. Open a Pull Request

## 📄 License

[MIT](LICENSE) © 2026 Yan Matagne
]]>