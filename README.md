<div align="center">

<img src=".github/assets/banner.png" width="100%" />

</div>

## ✨ Features

- **Multi-tab workflow:** Open, close, and switch between terminal tabs without leaving the window.
- **ANSI & VTE support:** Full escape code processing via `vte` for accurate rendering of colors, cursor movement, and control sequences.
- **Built-in font:** Ships with Fira Code Nerd Font — no system font installation required.

## 🚀 Installation

Nova is built from source with Cargo. You'll need [Rust](https://rustup.rs/) (stable, 2024 edition).

```sh
git clone https://github.com/pmqueiroz/nova.git
cd nova
cargo build --release
```

The binary will be at:

```
./target/release/nova
```

You can move it into your `$PATH`:

```sh
cp ./target/release/nova ~/.local/bin/
```

## ⌨️ Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl` + `V` | Paste from clipboard |

Tab and window controls are available via the UI (title bar buttons and tab bar).

## 🛣️ Roadmap

- [x] Multi-tab support
- [x] PTY resize on window resize
- [x] Clipboard paste
- [x] Cross-platform icons
- [ ] Configurable keybindings
- [ ] Font size configuration
- [ ] Scrollback buffer
