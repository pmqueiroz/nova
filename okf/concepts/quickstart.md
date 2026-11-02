---
type: concept
title: Quickstart
description: "First run, config location, and common workflows."
source: "https://nova.pmqueiroz.dev/quickstart/"
path: /quickstart/
updated: 2026-11-02
okf:
  generated_by: "@docmd/plugin-okf"
  generated_at: "2026-11-02T00:43:21.504Z"
---
---
title: "Quickstart"
description: "First run, config location, and common workflows."
---

## First Run

On first launch, Nova creates a settings file if it doesn't exist.

Config file path is:

- Windows: `%APPDATA%\nova\settings.toml`
- macOS: `~/Library/Application Support/nova/settings.toml`
- Linux: `~/.config/nova/settings.toml`

## Editing Settings

Nova reads `settings.toml` on startup.

Important behavior:

- If the file is missing, Nova writes an embedded default config.
- If the file exists but fails to parse as TOML, Nova overwrites it with defaults.

If you're experimenting, keep a backup of `settings.toml`.

## Command blocks

Nova runs your shell behind its own input bar. Type a command and press
`Enter`: the command and its output appear as a block above the input. Failed
commands show their output in red. Click inside an output card to select text,
`Ctrl+click` a link to open it.

Block mode needs Nova's shell integration (PowerShell, cmd, bash, zsh and Git
Bash are set up automatically). Shells that don't report their prompt (for
example fish, or WSL distributions) fall back to a classic terminal view.

## Command Palette

`Ctrl+P` (`Cmd+P` on macOS) opens the command palette: new or close tab,
clear screen, Ask AI, larger/smaller text, toggle timestamps, settings,
split pane, find and new tab with a specific shell. Type to filter.

See [Keybindings](/keybindings) for changing the palette shortcut.

## AI Overlay

If enabled, AI uses your question plus terminal context from the last command output.

See [AI](/ai) for providers, privacy notes, and troubleshooting.
