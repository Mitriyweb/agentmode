# agentmode 🤖⚡

> Keep your Mac awake while AI agents run — lid closed, no power cable required.

[![CI](https://github.com/Mitriyweb/agentmode/actions/workflows/ci.yml/badge.svg)](https://github.com/Mitriyweb/agentmode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A lightweight Rust CLI that prevents macOS from sleeping when you close the laptop lid. Built specifically for AI agent workflows (Claude Code, LLM agents, long-running scripts).

Unlike `caffeinate`, **agentmode** is process-aware: it holds the sleep assertion exactly as long as your agent runs, then cleanly releases it. Optional Telegram notifications tell you when the job is done.

---

## Features

- 🔒 **Prevents lid-close sleep** via IOKit `PreventUserIdleSystemSleep` assertion
- 🎯 **Process-aware** — releases automatically when your command exits
- 👁 **Attach to existing PID** — works with already-running agents
- 📬 **Telegram notifications** — start & done messages with exit code and elapsed time
- ♾️ **Keep mode** — prevent sleep indefinitely until `Ctrl+C`
- 🦀 **Single binary** — no runtime, no dependencies, ~2MB

---

## Installation

### Option 1: One-line install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/Mitriyweb/agentmode/main/install.sh | bash
```

Detects your platform (macOS Apple Silicon or Intel), downloads the binary, and installs it to `~/.local/bin/`.

### Option 2: Via Cargo

```bash
cargo install agentmode
```

### Option 3: From source

```bash
git clone https://github.com/Mitriyweb/agentmode.git
cd agentmode
cargo build --release
cp target/release/agentmode /usr/local/bin/
```

### Via Homebrew (coming soon)

```bash
brew install Mitriyweb/tap/agentmode
```

---

## Usage

### Run a command (most common)

```bash
agentmode run "bun swarm/index.ts"
agentmode run "python train.py --epochs 100"
agentmode run "claude -p 'analyze this codebase' --output-format json"
```

Supports full shell syntax:
```bash
agentmode run "cd ~/project && npm run build && npm test"
```

### Attach to running process

```bash
# Find your agent's PID
ps aux | grep swarm

# Attach agentmode to it
agentmode attach 12345
```

### Keep awake indefinitely

```bash
agentmode keep
agentmode keep "downloading training data"
```

Press `Ctrl+C` to release.

---

## Telegram Notifications

Get notified on your phone when the agent finishes.

**Step 1:** Create a bot via [@BotFather](https://t.me/botfather), get your token.

**Step 2:** Get your chat ID via [@userinfobot](https://t.me/userinfobot).

**Step 3:** Either export env vars or pass as flags:

```bash
# Via environment variables (recommended)
export TELEGRAM_TOKEN="123456:ABC-your-token"
export TELEGRAM_CHAT_ID="987654321"

agentmode run "bun agent-team/index.ts"
```

```bash
# Or inline flags
agentmode run \
  --telegram-token "123456:ABC-your-token" \
  --telegram-chat-id "987654321" \
  "bun swarm/index.ts"
```

You'll receive:
- 🟢 **Start message** — command + PID
- ✅ / ❌ **Done message** — exit code + elapsed time

---

## How it works

macOS goes to sleep on lid close by default, even if you have `caffeinate` running (without `-d` or external display). agentmode acquires an `IOPMAssertionCreateWithName` assertion of type `PreventUserIdleSystemSleep` directly via IOKit — the same mechanism used by professional tools like Amphetamine.

The assertion is held in a Rust RAII guard (`Drop` trait), so it's **guaranteed to be released** even if the process crashes or is killed.

```text
agentmode run "your-command"
  │
  ├─ acquire IOKit assertion  ←─ Mac stays awake
  ├─ spawn child process
  ├─ send Telegram: started
  ├─ wait for child to exit  ←─ polling every 500ms
  ├─ send Telegram: done ✅
  └─ drop assertion           ←─ Mac can sleep again
```

> ⚠️ **Thermal note:** Running with lid closed limits airflow. For CPU-heavy agents, consider monitoring temperature with `sudo powermetrics -s thermal`.

---

## Requirements

- macOS 12+ (Monterey or later)
- Apple Silicon or Intel
- Rust 1.75+ (for building from source)

---

## Development & Testing

### Building Locally

```bash
# Debug build (fast compilation)
cargo build

# Release build (optimized ~2MB binary)
cargo build --release
```

### Running Locally

```bash
# Run directly via Cargo
cargo run -- --help
cargo run -- run "sleep 2"
cargo run -- keep "testing local agentmode"

# Run the compiled debug binary
./target/debug/agentmode --help
./target/debug/agentmode run "echo 'Hello world'"

# Run the compiled release binary
./target/release/agentmode keep

# Install locally to cargo PATH (~/.cargo/bin)
cargo install --path .
```

### Running Tests & Lints

```bash
# Run unit tests
cargo test

# Check code formatting
cargo fmt --check

# Run Clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# Run pre-commit checks matching CI
./.githooks/pre-commit
```

---

## Comparison

| Tool | Process-aware | Telegram | No GUI needed | Rust |
|------|:---:|:---:|:---:|:---:|
| **agentmode** | ✅ | ✅ | ✅ | ✅ |
| caffeinate | ✅ (with `-w`) | ❌ | ✅ | ❌ |
| Amphetamine | ❌ | ❌ | ❌ | ❌ |
| NoSleep | ❌ | ❌ | ❌ | ❌ |

---

## Contributing

PRs welcome. Especially interested in:

- [ ] Homebrew formula
- [ ] `--notify-discord` flag
- [ ] `--max-duration` safety timeout
- [ ] macOS menubar companion app (Swift)
- [ ] `agentmode status` — show active assertions

---

## License

MIT © 2025-2026 Mitriyweb
