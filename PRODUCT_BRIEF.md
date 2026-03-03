# 🎮 TermiMon — Product Brief & Technical Spec

> **"Your AI agents, alive in the terminal."**

---

## Table of Contents

1. [Product Vision](#1-product-vision)
2. [Why This Will Win](#2-why-this-will-win)
3. [Technical Architecture](#3-technical-architecture)
4. [Creature Mechanics](#4-creature-mechanics)
5. [Agent Detection System](#5-agent-detection-system)
6. [MVP Scope](#6-mvp-scope)
7. [Post-MVP Roadmap](#7-post-mvp-roadmap)
8. [Legal Strategy](#8-legal-strategy)
9. [File Structure](#9-file-structure)
10. [Go-to-Market](#10-go-to-market)

---

## 1. Product Vision

### Name: **TermiMon**

**Terminal + Monster.** Short, memorable, ".com available" tier. Sounds like "terminal" and "monster" — immediately conveys what it is. Easy to type, easy to say, easy to Google.

**Runner-up names** (in case TermiMon is taken):
- **ShellMon** — Shell + Monster
- **tmuxmon** — direct, descriptive
- **Crittermux** — cute but less brandable
- **Kodamon** — Code + Monster (original IP feel)

**Recommendation: Go with `TermiMon` for branding, `termimon` for the CLI/package name.**

### Tagline Options

1. **"Your AI agents, alive in the terminal."** ← primary
2. "Gotta spawn 'em all."
3. "The Pokédex for your coding agents."
4. "Watch your agents evolve."

### Positioning

TermiMon is a **terminal companion** that gives your AI coding agents visible, animated creature avatars inside tmux. It turns the invisible work of AI agents into something you can *see*, *enjoy*, and *share*.

It's not a productivity tool. It's not a toy. It's a **vibe layer** — the same reason people customize their terminal prompt, use Nerd Fonts, or rice their desktop. Except this one is alive and reacts to what your AI is actually doing.

### Target User

- Senior developers who live in tmux
- People running multiple AI coding agents simultaneously (Claude Code, Codex CLI, aider, Cursor terminal, etc.)
- Terminal customization enthusiasts (r/unixporn crowd)
- The "I have 47 tmux panes open" archetype

### Why tmux Users Would LOVE This

1. **tmux users are power users** — they already care about their terminal environment
2. **Multi-agent is the future** — people run 3-5 AI agents simultaneously. It's genuinely hard to track what they're all doing. TermiMon makes agent status *glanceable*.
3. **Terminal is lonely** — VS Code has its pixel agents, its themes, its personality. Terminal is utilitarian. This changes that.
4. **It's the perfect flex** — screenshot of your tmux with 4 animated creatures coding alongside you? That's a viral tweet.
5. **Zero-commitment install** — `brew install termimon` and it works. No editor plugins, no config files needed.

---

## 2. Why This Will Win

### Viral Mechanics

1. **Screenshot/GIF bait** — Animated pixel creatures in a terminal are inherently shareable. Every screenshot tells a story.
2. **Collection mechanics** — "I've evolved my Claude agent to final form" is something people will tweet about.
3. **Configuration pride** — Custom creature assignments, evolved forms, shared configs = community content.
4. **The "Wait, how?" factor** — People seeing pixel art animals in a terminal will ask how. That's organic reach.
5. **Streamers/YouTubers** — Coding streamers will adopt this immediately. It makes their terminal sessions visually interesting.

### Competitive Moat

- **Pixel Agents** is VS Code only. Terminal users are a different (and underserved) market.
- **No one else** is doing terminal-native agent visualization. First mover.
- **Open source** = community sprites, community agent detectors, community themes.

---

## 3. Technical Architecture

### 3.1 Rendering Strategy

This is the hardest technical decision. Here's the analysis:

| Method | Resolution | Color | Terminal Support | tmux Support | Recommendation |
|--------|-----------|-------|-----------------|-------------|----------------|
| **Half-block (▀▄)** | 2x vertical | 256/true color | Universal | ✅ Full | **✅ PRIMARY** |
| **Braille (⠿)** | 2x4 dots | Mono per cell | Wide | ✅ Full | Good for outlines |
| **Sixel** | Full bitmap | Full color | ~60% terminals | ⚠️ Partial (tmux 3.4+) | Future option |
| **Kitty protocol** | Full bitmap | Full color | Kitty only | ❌ No | Don't use |
| **iTerm inline images** | Full bitmap | Full color | iTerm only | ⚠️ Hack | Don't use |

**Decision: Half-block characters as primary renderer.**

Why:
- Works in **every** terminal that supports 256 colors (which is all of them in 2025)
- Works perfectly inside tmux with zero special configuration
- Each terminal cell becomes a 1×2 pixel canvas using `▀` (upper half block) with foreground = top pixel, background = bottom pixel
- A 32×32 sprite = 32 columns × 16 rows of terminal characters. That's tiny and efficient.
- True color (24-bit) support via `\e[38;2;r;g;bm` gives us gorgeous sprites

**Sixel as optional upgrade path** — tmux 3.4+ added passthrough support for sixel. We detect this and upgrade rendering automatically. But it's never required.

**Sprite Resolution: 32×32 pixels** (renders as 32×16 terminal cells)
- Small enough to fit in a status bar or small pane
- Large enough for recognizable, charming creatures
- Standard size used by GBA-era Pokémon sprites (the aesthetic we want)

### 3.2 tmux Integration Approach

Three display modes, user-configurable:

#### Mode 1: **Status Bar** (Default — Recommended)
```
┌─────────────────────────────────────────────────────────┐
│ 0:code  1:tests  2:deploy │  🔥Charmander[typing] ⚡Pikachu[idle] 💧Squirtle[reading] │
└─────────────────────────────────────────────────────────┘
```
- Uses tmux's `status-right` or a dedicated status line
- Shows: creature emoji/icon + name + current state
- Minimal: just colored Unicode symbols + text
- Click to expand (tmux 3.2+ mouse support)
- **Zero visual overhead** — doesn't steal any pane space

#### Mode 2: **Popup Overlay** (tmux 3.2+)
```
tmux display-popup -E -w 40 -h 20 "termimon dashboard"
```
- Triggered by hotkey (e.g., `prefix + P`)
- Shows full pixel art creatures with animations
- Dashboard view: all agents, their states, stats
- Dismisses cleanly, returns to work
- **Best balance** of visual fidelity and non-intrusiveness

#### Mode 3: **Dedicated Pane**
```
┌──────────────────────────┬────────────────┐
│                          │  TermiMon      │
│   Your code / agent      │  ┌──────────┐  │
│   terminal here          │  │ Charmander│  │
│                          │  │  (typing) │  │
│                          │  └──────────┘  │
│                          │  ┌──────────┐  │
│                          │  │  Pikachu  │  │
│                          │  │  (idle)   │  │
│                          │  └──────────┘  │
└──────────────────────────┴────────────────┘
```
- Vertical or horizontal split pane
- Rich rendering with full sprite animations
- Good for streamers/demos
- **Most visual, most screen cost**

**Recommendation for MVP: Status bar + Popup. Skip dedicated pane until v0.2.**

### 3.3 Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                 termimon daemon                   │
│  (long-running background process)               │
│                                                   │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────┐ │
│  │ Agent        │  │ Creature     │  │ Renderer│ │
│  │ Detector     │  │ Engine       │  │         │ │
│  │              │  │              │  │ - half  │ │
│  │ - watches    │  │ - state mgr  │  │   block │ │
│  │   tmux panes │  │ - animations │  │ - sixel │ │
│  │ - parses     │  │ - evolution  │  │ - text  │ │
│  │   output     │  │ - XP system  │  │         │ │
│  └──────┬───────┘  └──────┬───────┘  └────┬────┘ │
│         │                 │               │       │
│  ┌──────▼─────────────────▼───────────────▼────┐ │
│  │            tmux IPC (control mode)           │ │
│  │  - status bar updates                        │ │
│  │  - popup rendering                           │ │
│  │  - pane capture for agent detection          │ │
│  └──────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
                         │
              ┌──────────▼──────────┐
              │  ~/.termimon/        │
              │  - config.toml       │
              │  - creatures.json    │ (your collection)
              │  - sprites/          │ (sprite sheets)
              │  - state.json        │ (runtime state)
              └─────────────────────┘
```

### 3.4 Process Model

```
termimon start    → spawns daemon, attaches to current tmux session
termimon stop     → kills daemon
termimon status   → shows all tracked agents + creatures
termimon dash     → opens tmux popup with dashboard
termimon assign   → manually assign creature to a pane
termimon config   → edit configuration
termimon pokedex  → show collection / stats
```

The daemon:
1. Runs as a background process (not inside tmux — alongside it)
2. Connects to tmux via `tmux -C` (control mode) for zero-overhead IPC
3. Periodically captures pane content via `tmux capture-pane -p -t <pane>`
4. Updates status bar via `tmux set -g status-right ...`
5. Manages popup rendering on demand

**Performance budget: <1% CPU, <20MB RAM.** The daemon does:
- Pane content polling every 2 seconds (configurable)
- Lightweight regex matching on captured output
- Status bar string updates (trivial)
- Sprite animation only when popup/pane is visible

### 3.5 Tech Stack

| Component | Choice | Why |
|-----------|--------|-----|
| **Language** | **Rust** | Fast startup, low memory, single binary distribution, excellent terminal ecosystem (ratatui, crossterm) |
| **Terminal UI** | **ratatui** | Best terminal UI framework in Rust. Half-block rendering built-in. Battle-tested. |
| **tmux IPC** | **tmux control mode** (`tmux -C`) | Structured output, event-driven, no shell overhead |
| **Sprite format** | **Custom compact binary** | Pre-processed from PNG at build time. Tiny, fast to render. |
| **Config** | **TOML** | Standard for Rust CLI tools. Human-readable. |
| **State** | **JSON** | Simple, portable, human-debuggable |
| **Distribution** | **Single binary** | `cargo install termimon` / `brew install termimon` / GitHub releases |

**Why Rust over Go/TypeScript/Python:**
- **Go**: Viable but larger binaries, no ratatui equivalent as mature
- **TypeScript**: Too heavy. Node.js runtime = 50MB+ just to start
- **Python**: Too slow for animation rendering, painful distribution
- **Rust**: 3MB binary, instant startup, the terminal tool community is Rust-native (starship, zoxide, bat, etc.)

---

## 4. Creature Mechanics

### 4.1 The IP Question (Addressed in Section 8)

**We do NOT use Pokémon directly.** Instead, we create **original creatures** that capture the same magic. Think of it like Palworld — inspired by the concept, original in execution.

**But** — we design the creature system to be **themeable**. The default theme is our original creatures. Community themes can include whatever the community wants (wink wink). This is the Minecraft texture pack model.

### 4.2 Default Creatures: "TermiMon" Original Series

Each creature is inspired by coding/terminal concepts with elemental affinities:

| Creature | Type | Vibe | Assigned To |
|----------|------|------|-------------|
| **Embercli** | 🔥 Fire | A fiery fox with terminal-green eyes | Claude Code |
| **Voltprompt** | ⚡ Electric | Sparky mouse with antenna ears | Codex CLI |
| **Shelloise** | 💧 Water | Turtle with a terminal shell on its back | aider |
| **Rustacean** | 🦀 Steel | A crab (duh) | Rust-based agents |
| **Pythorn** | 🌿 Grass | A thorny snake | Python-based agents |
| **Gitbat** | 🦇 Dark | A bat that hangs from the status bar | git operations |
| **Kernowl** | 🦉 Psychic | Wise owl with glowing eyes | System/kernel tasks |
| **Dockerk** | 🐋 Water | Whale (obviously) | Docker/container agents |
| **Nixfox** | ❄️ Ice | Arctic fox | Nix/NixOS agents |
| **Vimera** | 💜 Poison | A chimera with modal expressions | Vim/Neovim agents |

**Each creature has 3 evolution stages:**
- Stage 1: Base form (new agent, <100 XP)
- Stage 2: Evolved form (>100 XP, agent has completed significant work)
- Stage 3: Final form (>500 XP, agent has been used extensively)

### 4.3 Animation States

Each creature has these animation frames (4 frames each, looping):

| State | Trigger | Animation |
|-------|---------|-----------|
| **Idle** | Agent waiting for input | Gentle breathing/bobbing |
| **Typing** | Agent writing code/files | Rapid paw/claw movements |
| **Reading** | Agent searching/reading files | Eyes scanning left-right |
| **Thinking** | Agent processing/planning | Spinning thought bubble |
| **Running** | Agent executing commands | Running in place |
| **Sleeping** | Agent inactive >5 min | Zzz animation |
| **Celebrating** | Agent completed a task | Jump + sparkles |
| **Error** | Agent hit an error | Dizzy stars |
| **Waiting** | Agent needs user permission | Tapping foot, speech bubble "?" |

### 4.4 XP & Evolution System

```
XP Sources:
  - Agent writes a file:        +5 XP
  - Agent completes a task:      +20 XP
  - Agent runs for >10 min:      +10 XP
  - Agent fixes an error:        +15 XP
  - Agent creates a new file:    +10 XP

Evolution thresholds:
  - Stage 2: 100 XP
  - Stage 3: 500 XP

Evolution event:
  - When threshold is hit, next time the agent completes a task:
  - Flash animation (3 seconds)
  - Status bar shows "🎉 Embercli evolved into Blazecli!"
  - Optional: terminal bell / notification
```

XP is persistent (stored in `~/.termimon/creatures.json`) and tied to the agent identity, not the session.

### 4.5 Creature Assignment Logic

```
Priority order:
1. User explicit assignment (termimon assign --pane 3 --creature embercli)
2. Agent-type mapping (Claude Code → Embercli, Codex → Voltprompt)
3. Config file defaults
4. Random from unassigned pool
```

Agent-type detection → creature mapping is the default "just works" experience. User can override everything.

### 4.6 Fun Mechanics (Post-MVP)

- **Battle Mode**: When two agents edit the same file, their creatures do a brief battle animation. The agent whose changes persist "wins."
- **Friendship**: Creatures that work together often develop a friendship indicator. Co-assigned agents show hearts.
- **Rare Encounters**: Random chance (~1%) of a "shiny" variant spawning. Different color palette.
- **Habitat**: The tmux session is the "habitat." Creatures from different tmux sessions are different "regions."
- **Trading**: Export creature state as a shareable JSON/QR code. Trade with friends.
- **Leaderboard**: Optional anonymous stats — who has the most evolved creatures?

---

## 5. Agent Detection System

This is the core intelligence of TermiMon. It needs to detect:
1. **Which panes have AI agents** (vs. regular shells)
2. **What state the agent is in** (typing, reading, thinking, etc.)

### 5.1 Detection Methods

#### Method A: Process Tree Inspection (Primary)
```bash
# Get the process running in a tmux pane
tmux list-panes -F '#{pane_id} #{pane_pid}'
# Then inspect the process tree
ps -o pid,comm -g <pane_pid>
```

Known agent processes:
- `claude` → Claude Code
- `codex` → Codex CLI
- `aider` → aider
- `opencode` → OpenCode
- `cursor-cli` → Cursor terminal agent
- Custom patterns in config

#### Method B: Output Pattern Matching (Secondary)
```bash
tmux capture-pane -p -t <pane_id> | tail -20
```

Pattern signatures:
- Claude Code: `claude ❯`, `Thinking...`, `Writing to file...`, `Running command...`
- Codex CLI: `codex>`, thinking indicators
- aider: `aider>`, `Tokens:`, `Cost:`
- Generic: Detect structured output patterns (progress bars, file paths, command output)

#### Method C: JSONL Transcript Watching (Best for Claude Code)
```
# Claude Code writes transcripts to:
# ~/.claude/projects/<project>/sessions/<id>/transcript.jsonl

# Watch for new entries, parse tool usage:
{"type":"tool_use","name":"write_to_file",...}  → typing state
{"type":"tool_use","name":"read_file",...}       → reading state
{"type":"tool_use","name":"bash",...}             → running state
{"type":"thinking",...}                           → thinking state
```

This is the same approach Pixel Agents uses. It's the most reliable for Claude Code specifically.

#### Method D: Plugin API (Future)
Expose a simple IPC socket/pipe that agents can write to directly:
```bash
echo '{"agent":"claude","state":"typing","file":"main.rs"}' | termimon-pipe
```

### 5.2 State Machine

```
               ┌──────────┐
               │  UNKNOWN  │ (new pane detected)
               └─────┬─────┘
                     │ agent detected
                     ▼
               ┌──────────┐
          ┌───▶│   IDLE    │◀───┐
          │    └─────┬─────┘    │
          │          │          │
          │    ┌─────▼─────┐   │
          │    │  THINKING  │───┘ (timeout)
          │    └─────┬─────┘
          │          │
          │    ┌─────▼─────┐
          ├────│  TYPING    │
          │    └────────────┘
          │    ┌────────────┐
          ├────│  READING   │
          │    └────────────┘
          │    ┌────────────┐
          ├────│  RUNNING   │
          │    └────────────┘
          │    ┌────────────┐
          └────│  SLEEPING  │ (>5 min idle)
               └────────────┘
```

---

## 6. MVP Scope

### 6.1 What Ships in v0.1.0

**The smallest lovable product:**

1. ✅ `termimon start` / `termimon stop` — daemon lifecycle
2. ✅ Auto-detect Claude Code and Codex CLI in tmux panes
3. ✅ 3 creatures with idle + typing + thinking animations (half-block rendered)
4. ✅ tmux status bar integration showing creature + state
5. ✅ `termimon dash` popup with full sprite rendering
6. ✅ Basic XP tracking (persistent across sessions)
7. ✅ Config file for creature assignment overrides
8. ✅ Works on macOS + Linux, any terminal with 256 color

**What does NOT ship in v0.1.0:**
- ❌ Evolution animations (XP tracks, but all creatures are stage 1)
- ❌ Battle mode
- ❌ Sixel rendering
- ❌ Sound notifications
- ❌ Shiny variants
- ❌ Community sprite packs

### 6.2 Sprint Plan (4 weeks)

**Week 1: Foundation**
- [ ] Project scaffold (Rust, cargo workspace)
- [ ] tmux control mode IPC library
- [ ] Pane detection + process tree inspection
- [ ] Agent state machine (Claude Code first)
- [ ] Config file parsing

**Week 2: Rendering**
- [ ] Half-block sprite renderer
- [ ] Sprite sheet loader (from pre-built assets)
- [ ] Animation loop (4fps is plenty)
- [ ] tmux status bar formatter
- [ ] tmux popup renderer (via ratatui)

**Week 3: Creatures & Polish**
- [ ] 3 initial creature designs (pixel art)
- [ ] All animation states for each creature
- [ ] XP system
- [ ] Dashboard layout
- [ ] CLI commands (start/stop/status/dash/config)

**Week 4: Ship It**
- [ ] Cross-platform testing (macOS, Linux, various terminals)
- [ ] Homebrew formula
- [ ] cargo publish
- [ ] GitHub releases with prebuilt binaries
- [ ] README with screenshots/GIFs
- [ ] Demo video (30 seconds, GIF-able)

### 6.3 Key Dependencies

```toml
[dependencies]
ratatui = "0.29"          # Terminal UI framework
crossterm = "0.28"        # Terminal manipulation
tokio = { version = "1", features = ["full"] }  # Async runtime
serde = { version = "1", features = ["derive"] }  # Serialization
serde_json = "1"          # JSON state files
toml = "0.8"              # Config parsing
notify = "7"              # File watching (for JSONL transcripts)
image = "0.25"            # PNG sprite loading at build time
clap = { version = "4", features = ["derive"] }  # CLI argument parsing
nix = "0.29"              # Unix process inspection
dirs = "6"                # XDG directory paths
tracing = "0.1"           # Logging
tracing-subscriber = "0.3"
```

---

## 7. Post-MVP Roadmap

### v0.2.0 — "Evolution Update"
- Evolution animations when creatures level up
- Stage 2 sprites for all creatures
- Aider agent detection
- Sound notification support (terminal bell + optional system notification)
- `termimon pokedex` command

### v0.3.0 — "Social Update"
- Community sprite pack system (drop PNG sheets in `~/.termimon/themes/`)
- Creature export/import (share your evolved creatures)
- GitHub badge integration ("My TermiMon team" in README)
- Shiny variants (1% random chance)

### v0.4.0 — "Battle Update"
- Conflict detection (same file edited by multiple agents)
- Battle animations
- Battle log
- Win/loss tracking

### v0.5.0 — "Ecosystem Update"
- Plugin API for custom agent detectors
- Sixel rendering mode (auto-detected)
- Web dashboard (termimon serve → localhost view)
- More creatures (community submissions)

### v1.0.0 — "Full Release"
- 20+ creatures, all with 3 evolution stages
- Stable API for integrations
- Theme engine
- Comprehensive agent support (10+ agent types)

---

## 8. Legal Strategy

### The Pokémon Problem

**You cannot use Pokémon IP.** Period. Here's why and what to do instead:

#### What You Can't Do:
- ❌ Use any Pokémon names (Pikachu, Charmander, etc.)
- ❌ Use Pokémon sprite art (even "inspired by" is risky)
- ❌ Use the word "Pokémon" in the product name
- ❌ Reference specific Pokémon mechanics by trademarked names (Pokedex™, etc.)
- ❌ Distribute fan art sprites as part of the product

The Pokémon Company is **aggressive** about enforcement. They DMCA fan games regularly. An open-source project with Pokémon assets in the repo = takedown within weeks.

#### What You CAN Do:

1. **Create original creatures** — This is the Palworld strategy. Monster-collecting is a *genre*, not IP. You can have creatures that evolve, have types, battle, etc. That's all generic game mechanics.

2. **Design a theme/skin system** — Ship with original creatures. Let the community create "skin packs." What the community does on their own machines is their business. You never host or distribute Pokémon assets.

3. **Use creature-collecting vocabulary carefully:**
   - ✅ "Creatures" / "Monsters" / "Companions"
   - ✅ "Evolution" (generic concept)
   - ✅ "Types" / "Elements" (generic)
   - ✅ "Collection" / "Codex" / "Bestiary"
   - ❌ "Pokédex" (trademarked)
   - ❌ "Pokémon" (trademarked)
   - ❌ "Gotta catch 'em all" (trademarked)

4. **Community theme packs** — Ship the infrastructure. Let people create a `pokemon-classic` theme pack hosted on their own repos. Your project's README can say "supports custom sprite themes" without ever mentioning Pokémon.

#### The Recommended Approach:

```
Default: Original "TermiMon" creatures (safe, unique, brandable)
System:  Theme engine that loads sprite packs from ~/.termimon/themes/
Result:  Community creates Pokémon/Digimon/whatever themes independently
You:     Never distribute, endorse, or link to infringing themes
```

This is exactly how Minecraft texture packs, Firefox themes, and VS Code icon packs work. The platform is clean; the community does what communities do.

**Bonus: Original creatures are BETTER for branding.** If this takes off, you own the IP. "Embercli" can be on merch, stickers, social media. Pikachu can't.

### Pixel Art Licensing

For original sprites, options:
1. **Commission pixel artists** — $50-200 per creature sprite sheet on Fiverr/Twitter. Get 3 for MVP.
2. **Use CC0/open-source sprite bases** — Modify existing open game art.
3. **AI-generate base designs** — Use as reference, manually pixel by hand for final art.
4. **Community contributions** — After launch, artists will contribute (it's fun to make pixel art).

License all original art under **CC BY-SA 4.0** — copyleft ensures community contributions stay open.

---

## 9. File Structure

```
termimon/
├── Cargo.toml
├── Cargo.lock
├── LICENSE                     # MIT
├── README.md
├── CONTRIBUTING.md
├── assets/
│   ├── sprites/
│   │   ├── embercli/
│   │   │   ├── stage1.png      # 32x32 sprite sheet (8 frames x 9 states = 72 frames)
│   │   │   ├── stage2.png
│   │   │   └── stage3.png
│   │   ├── voltprompt/
│   │   │   └── ...
│   │   └── shelloise/
│   │       └── ...
│   └── themes/
│       └── default/
│           └── manifest.toml
├── build.rs                    # Converts PNGs to compact binary format at compile time
├── src/
│   ├── main.rs                 # CLI entry point (clap)
│   ├── lib.rs                  # Library root
│   ├── daemon/
│   │   ├── mod.rs              # Daemon lifecycle (start/stop/status)
│   │   ├── server.rs           # Unix socket IPC for CLI↔daemon
│   │   └── heartbeat.rs        # Main polling loop
│   ├── tmux/
│   │   ├── mod.rs
│   │   ├── control.rs          # tmux control mode (-C) connection
│   │   ├── pane.rs             # Pane discovery + capture
│   │   └── status.rs           # Status bar formatting + updates
│   ├── agents/
│   │   ├── mod.rs              # Agent trait + registry
│   │   ├── detector.rs         # Process tree + output pattern matching
│   │   ├── claude.rs           # Claude Code specific detection (JSONL)
│   │   ├── codex.rs            # Codex CLI detection
│   │   ├── aider.rs            # aider detection
│   │   └── generic.rs          # Fallback heuristic detector
│   ├── creatures/
│   │   ├── mod.rs              # Creature trait, state machine
│   │   ├── registry.rs         # All creatures + their metadata
│   │   ├── animation.rs        # Frame sequencing, timing
│   │   ├── evolution.rs        # XP tracking, evolution logic
│   │   └── assignment.rs       # Creature↔agent mapping
│   ├── render/
│   │   ├── mod.rs              # Renderer trait
│   │   ├── halfblock.rs        # Half-block Unicode renderer
│   │   ├── sixel.rs            # Sixel renderer (optional)
│   │   ├── text.rs             # Minimal text-only fallback
│   │   └── sprite.rs           # Sprite sheet parser, frame extraction
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── dashboard.rs        # Popup dashboard (ratatui)
│   │   ├── statusbar.rs        # Status bar content generator
│   │   └── notification.rs     # Evolution alerts, battle results
│   ├── config/
│   │   ├── mod.rs              # Config loading, defaults, validation
│   │   └── schema.rs           # Config struct definitions
│   └── state/
│       ├── mod.rs              # Persistent state management
│       └── creatures.rs        # Creature collection, XP, evolution state
├── tests/
│   ├── integration/
│   │   ├── tmux_test.rs        # Tests with real tmux sessions
│   │   └── detection_test.rs   # Agent detection tests
│   └── unit/
│       ├── animation_test.rs
│       ├── evolution_test.rs
│       └── renderer_test.rs
└── docs/
    ├── ARCHITECTURE.md
    ├── THEMES.md               # How to create custom themes
    └── AGENTS.md               # How to add agent detectors
```

### Config File (~/.termimon/config.toml)

```toml
[general]
poll_interval_ms = 2000
display_mode = "statusbar"  # "statusbar" | "popup" | "pane"
animation_fps = 4
theme = "default"

[statusbar]
position = "right"          # "left" | "right"
max_creatures = 5
show_xp = false
show_state = true
format = "{icon} {name}[{state}]"

[popup]
width = 60
height = 20
hotkey = "P"                # prefix + P

[creatures]
# Explicit assignments override auto-detection
[creatures.assignments]
claude = "embercli"
codex = "voltprompt"
aider = "shelloise"

[agents]
# Custom agent detection patterns
[[agents.custom]]
name = "my-agent"
process = "my-agent-binary"
patterns = ["my-agent>", "Processing..."]
creature = "kernowl"

[notifications]
evolution = true
terminal_bell = false
system_notify = true        # macOS: osascript, Linux: notify-send
```

---

## 10. Go-to-Market

### Launch Strategy

1. **Pre-launch (1 week before)**
   - Create repo with just README + concept art + "coming soon"
   - Post to X/Twitter with a 15-second GIF showing the concept
   - Submit to Hacker News as "Show HN" (but just the concept, gauge interest)

2. **Launch Day**
   - Full repo with working v0.1.0
   - `brew install termimon` works on day 1
   - 30-second demo GIF in README
   - Post to:
     - Hacker News (Show HN)
     - r/programming, r/rust, r/commandline, r/unixporn
     - X/Twitter (target terminal/dev influencers)
     - Dev.to blog post
     - Lobste.rs (if you have an invite)

3. **Post-launch**
   - Engage with every GitHub issue personally
   - Merge community sprite contributions fast
   - Weekly releases for first month
   - "Creature of the Week" community votes

### Key Metrics to Track

- GitHub stars (vanity but real signal)
- brew install count
- cargo install count
- GitHub issues/PRs (community engagement)
- Mentions on X/Twitter/Reddit (organic spread)

### Community Plays

1. **"Design a Creature" contest** — Let community submit creature designs. Winner gets added to default theme.
2. **Streamer outreach** — DM coding streamers, offer to help them set it up on stream.
3. **Integration bounties** — $50 bounties for PRs adding new agent detectors.
4. **Monthly "evolution showcase"** — People share screenshots of their most evolved creatures.

---

## Appendix A: Half-Block Rendering Deep Dive

Here's how the actual rendering works:

```rust
// Each terminal cell represents 2 vertical pixels using the upper half block character
// ▀ (U+2580) with:
//   - Foreground color = top pixel
//   - Background color = bottom pixel

fn render_sprite_row(top_pixels: &[Color], bottom_pixels: &[Color]) -> String {
    let mut output = String::new();
    for (top, bottom) in top_pixels.iter().zip(bottom_pixels.iter()) {
        if top.is_transparent() && bottom.is_transparent() {
            output.push(' ');  // Both transparent = space
        } else if top.is_transparent() {
            // Only bottom pixel: use lower half block ▄ with fg=bottom
            output.push_str(&format!("\x1b[38;2;{};{};{}m▄\x1b[0m",
                bottom.r, bottom.g, bottom.b));
        } else if bottom.is_transparent() {
            // Only top pixel: use upper half block ▀ with fg=top
            output.push_str(&format!("\x1b[38;2;{};{};{}m▀\x1b[0m",
                top.r, top.g, top.b));
        } else {
            // Both pixels: ▀ with fg=top, bg=bottom
            output.push_str(&format!(
                "\x1b[38;2;{};{};{};48;2;{};{};{}m▀\x1b[0m",
                top.r, top.g, top.b,
                bottom.r, bottom.g, bottom.b
            ));
        }
    }
    output
}

// A 32x32 sprite becomes 32 columns × 16 rows
// That's small enough to fit in a tmux popup easily
```

## Appendix B: tmux Control Mode Example

```rust
// Connect to tmux in control mode for structured IPC
use std::process::Command;

fn connect_tmux_control() {
    // tmux -C attaches in control mode — structured output
    let child = Command::new("tmux")
        .args(["-C", "attach-session", "-t", "main"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to connect to tmux");

    // Send commands via stdin:
    // list-panes -a -F '#{pane_id}|#{pane_pid}|#{pane_current_command}'
    //
    // Receive structured responses on stdout:
    // %begin 1234
    // %1|12345|claude
    // %2|12346|zsh
    // %end 1234
}
```

## Appendix C: Sample Status Bar Output

**Minimal mode:**
```
⚡Voltprompt[typing] 🔥Embercli[thinking] 💧Shelloise[idle]
```

**Detailed mode:**
```
⚡Voltprompt Lv.3[typing main.rs] 🔥Embercli Lv.7[thinking] 💧Shelloise Lv.1[💤]
```

**Compact mode:**
```
⚡⌨️ 🔥🤔 💧💤
```

---

## Summary: Why This Will Work

1. **Real problem**: People run multiple AI agents and lose track. TermiMon makes agent state glanceable.
2. **Emotional hook**: Creatures that evolve with your work create attachment. You'll *want* to keep using your agents.
3. **Technical feasibility**: Half-block rendering is proven tech. tmux control mode is stable. This is eminently buildable.
4. **Distribution advantage**: Single binary, brew install, works with any terminal. Zero lock-in.
5. **Community flywheel**: Creature designs + themes + agent detectors = infinite community contribution surface.
6. **Legal safety**: Original IP, theme system for community content, no Pokémon in the repo.
7. **Viral potential**: GIFs of pixel creatures coding in a terminal are inherently shareable.

**The question isn't whether this should exist. It's who builds it first.**

---

*This document is a complete product brief and technical specification. A senior Rust developer with terminal experience could start building from this tomorrow.*

*Estimated time to MVP: 4 weeks for one developer, 2 weeks for two.*

*Let's build it.* 🔥
