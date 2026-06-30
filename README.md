<div align="center">

<img src="assets/ziro_banner.svg" alt="Ziro banner" />

![Status](https://img.shields.io/badge/status-building-blueviolet?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-white?style=flat-square)
![Built with Rust](https://img.shields.io/badge/built%20with-rust-orange?style=flat-square)
![Platform](https://img.shields.io/badge/platform-linux-lightgrey?style=flat-square)

</div>

---

Ziro is a featherlight terminal text editor (TUI) for Linux. No Electron. No Chromium. No unnecessary layers.

It opens before you finish blinking. It stays out of your way.

---

## why

Every modern editor makes you pay somewhere.

VSCode ships a whole browser to edit text. Neovim gives you unlimited power after a configuration ritual. Zed is promising but still finding its place on Linux. Sublime costs money.

Ziro is the editor that should have existed already — lightweight, instant, and built for developers who actually care about their tools.

---

## install

**Stable** — prebuilt binary, no Rust required:

```bash
curl -sSL https://raw.githubusercontent.com/FaizeenHoque/ziro/main/install.sh | bash
```

**Rolling** — builds from latest main, requires Cargo:

```bash
curl -sSL https://raw.githubusercontent.com/FaizeenHoque/ziro/main/install-rolling.sh | bash
```

> [!NOTE]
> Stable is recommended for most users. Rolling tracks the latest commit on main and may be unstable.

> [!WARNING]
> Both scripts require `sudo` to move the binary to `/usr/local/bin`. Read the script before running if you're cautious about piping curl to bash.

---

## usage

```bash
ziro              # open blank editor
ziro file.txt     # open a file
```

| Keybind | Action |
| ------- | ------ |
| `Esc` | Toggle command mode |
| `:w` | Save |
| `:q` | Quit |
| `:wq` / `:x` | Save and quit |

> [!IMPORTANT]
> Ziro will warn you if you try to quit with unsaved changes. Use `:wq` to save and exit in one step.

---

## goals

* **Fast cold launch.** Measured, not estimated.
* **Native terminal experience.** Runs anywhere your shell does.
* **Rope-based text engine.** Edits at any scale without copying the world.
* **Tree-sitter syntax highlighting.** Incremental, correct, fast.
* **LSP support.** Autocomplete, go-to-definition, diagnostics — the full deal.
* **Zero config to start.** Sane defaults. Customize when you want to, not before you can use it.
* **Minimal resource usage.** A text editor should not need half your RAM.

---

## stack

| Layer       | Tech             |
| ----------- | ---------------- |
| Language    | Rust             |
| UI          | TUI (`ratatui`)  |
| Text buffer | `ropey`          |
| Syntax      | `tree-sitter`    |
| Config      | `toml` + `serde` |
| Async       | `tokio`          |

---

## status

> [!CAUTION]
> Ziro is in early development. Nothing is stable. Everything is being built. Do not use this as your daily driver yet.

* [x] Project scaffold
* [x] Terminal interface
* [x] Editor UI
* [x] File open/save
* [x] Syntax highlighting
* [ ] LSP integration
* [ ] Config system
* [ ] Plugin system

---

## building

```bash
git clone https://github.com/FaizeenHoque/ziro
cd ziro
cargo run
```

Requires Rust 1.78+. That's it.

---

## license

MIT © 2026 Faizeen Hoque