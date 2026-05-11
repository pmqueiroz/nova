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

## Command Palette

Nova includes a command palette with actions like:

- Ask AI
- Explain Error
- New Tab
- Settings

See [Keybindings](/keybindings) for changing the palette shortcut.

## AI Overlay

If enabled, AI uses your question plus terminal context from the last command output.

See [AI](/ai) for providers, privacy notes, and troubleshooting.
