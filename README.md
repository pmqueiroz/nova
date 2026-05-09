<div align="center">

<img src=".github/assets/banner.png" width="100%" />

</div>

## ✨ Features

- **Multi-tab workflow:** Open, close, and switch between terminal tabs without leaving the window.
- **ANSI & VTE support:** Full escape code processing via `vte` for accurate rendering of colors, cursor movement, and control sequences.
- **Built-in font:** Ships with Fira Code Nerd Font — no system font installation required.
- **AI Agentic Features:** Integrated LLM capabilities directly within the terminal, including "Ask AI" for natural language command generation and automated explanations for complex command outputs.

## 🚀 Installation

### Homebrew

| Platform | Command |
|----------|---------|
| macOS | `brew install --cask pmqueiroz/tap/nova` |
| Linux | `brew install pmqueiroz/tap/nova` |

### Winget (Windows)

```sh
winget install pmqueiroz.Nova
```

### Scoop (Windows)

```sh
scoop bucket add pmqueiroz https://github.com/pmqueiroz/scoop-bucket
scoop install nova
```

### AUR (Arch Linux)

```sh
yay -S nova-bin
```

### APT (Debian/Ubuntu)

```sh
curl -fsSL https://pmqueiroz.github.io/apt-repo/KEY.gpg | sudo gpg --dearmor -o /etc/apt/keyrings/nova.gpg
echo "deb [trusted=yes arch=amd64] https://pmqueiroz.github.io/apt-repo stable main" \
  | sudo tee /etc/apt/sources.list.d/nova.list
sudo apt update && sudo apt install nova
```

### DNF/RPM (Fedora/RHEL)

```sh
sudo curl -fsSL https://pmqueiroz.github.io/rpm-repo/nova.repo \
  -o /etc/yum.repos.d/nova.repo
sudo dnf install nova
```

### Download a release

| Platform | Download |
|----------|----------|
| Windows x86_64 | [`.exe` installer](https://github.com/pmqueiroz/nova/releases/download/v0.12.5/nova_0.12.5_x64-setup.exe) · [portable `.zip`](https://github.com/pmqueiroz/nova/releases/download/v0.12.5/nova_0.12.5_x64_portable.zip) |
| macOS x86_64 | [`.dmg` disk image](https://github.com/pmqueiroz/nova/releases/download/v0.12.5/nova_0.12.5_x64.dmg) |
| macOS Apple Silicon | [`.dmg` disk image](https://github.com/pmqueiroz/nova/releases/download/v0.12.5/nova_0.12.5_aarch64.dmg) |
| Linux x86_64 | [`.deb`](https://github.com/pmqueiroz/nova/releases/download/v0.12.5/nova_0.12.5_amd64.deb) · [`.AppImage`](https://github.com/pmqueiroz/nova/releases/download/v0.12.5/nova_0.12.5_x86_64.AppImage) · [`.rpm`](https://github.com/pmqueiroz/nova/releases/download/v0.12.5/nova_0.12.5_x86_64.rpm) |

Each release includes a [`checksums.txt`](https://github.com/pmqueiroz/nova/releases/download/v0.12.5/checksums.txt) for verifying the download.

> [!WARNING]
> Nova is not notarized — macOS may block it with *"Nova is damaged and can't be opened."*
> Run this once after installing:
> ```sh
> xattr -cr /Applications/Nova.app
> ```

> [!TIP]
> If you'd like to see Nova become a signed and notarized app, consider [sponsoring the project](https://github.com/sponsors/pmqueiroz). ❤️

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
