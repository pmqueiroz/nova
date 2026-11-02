---
title: "CLI"
description: "nova CLI commands and how they integrate with the Nova app."
---

## Commands

Nova includes a small CLI surface:

- `nova help`: Show usage
- `nova ask [preset...]`: Open the Ask AI modal in the running Nova app
- `nova update [--install]` (alias `nova --check-update`): Check for a newer release, and optionally install it

## `nova update`

Asks GitHub for the latest release, compares it with the running version and says how Nova was installed. Works outside Nova and ignores `general.check-updates`.

```text
$ nova update
nova 1.1.0 is available (current 1.0.0), installed via the portable ZIP
run 'nova update --install' to update
```

`nova update --install` installs it the way that matches your install:

| Installed with | What runs |
|---|---|
| Windows installer / winget | Downloads the new setup; it runs silently the next time Nova starts (or on *Restart to update* in the app) |
| Portable ZIP | Downloads it and replaces `nova.exe` in place |
| DMG (macOS) | Downloads it and replaces `Nova.app` in place |
| AppImage | Downloads it and replaces the AppImage in place |
| Scoop | `scoop update nova` |
| Homebrew | `brew upgrade --cask nova` (macOS) / `brew upgrade nova` (Linux) |
| apt / .deb | Downloads the `.deb`, then `sudo apt install <file>` |
| dnf / .rpm | Downloads the `.rpm`, then `sudo dnf install <file>` (or `sudo rpm -U`) |
| AUR | `paru -S nova-bin` or `yay -S nova-bin` |
| cargo | `cargo install --git https://github.com/pmqueiroz/nova --tag vX.Y.Z --locked` |

Every download is checked against the release's `checksums.txt` before anything is replaced. Builds Nova can't classify (e.g. `target/release`) just print the release page. Exits `1` if the check or update fails.

In the app, the same thing is behind the orange **Update** button in the title bar (next to settings) and Settings → About: package-manager updates open in a new tab, and other installs download in the background and then offer **Restart to update**.

## `nova ask`

`nova ask` is not a standalone AI client.

It must be run *inside* Nova. Nova sets `NOVA_TERMINAL=1` for its child shells; the CLI checks that environment variable.

Examples:

```bash
nova ask
```

```bash
nova ask explain this error
```

If you're not in Nova, you'll see:

```text
error: 'nova ask' must be run inside Nova terminal
```

## How It Works

The CLI emits a private OSC sequence that Nova's terminal parser understands.

Details are documented in [OSC Integration](/docs/internals-osc/).
