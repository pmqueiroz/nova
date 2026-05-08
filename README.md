<div align="center">

<img src=".github/assets/banner.png" width="100%" />

</div>

## ✨ Features

- **Multi-tab workflow:** Open, close, and switch between terminal tabs without leaving the window.
- **ANSI & VTE support:** Full escape code processing via `vte` for accurate rendering of colors, cursor movement, and control sequences.
- **Built-in font:** Ships with Fira Code Nerd Font — no system font installation required.

## 🚀 Installation

### Homebrew (macOS)

```sh
brew install --cask pmqueiroz/tap/nova
```

### Download a release

Grab the latest installer for your platform from the [releases page](https://github.com/pmqueiroz/nova/releases):

| Platform | File |
|----------|------|
| Windows x86_64 | `.exe` installer |
| macOS x86_64 | `.dmg` disk image |
| macOS Apple Silicon | `.dmg` disk image |
| Linux x86_64 | `.deb` package or `.AppImage` |

Each release includes a `checksums.txt` for verifying the download.

### Build from source

You'll need [Rust](https://rustup.rs/) (stable, 2024 edition).

```sh
git clone https://github.com/pmqueiroz/nova.git
cd nova
cargo build --release
```

The binary will be at `./target/release/nova`. Move it into your `$PATH`:

```sh
cp ./target/release/nova ~/.local/bin/
```
