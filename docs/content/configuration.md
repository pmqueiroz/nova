---
title: "Configuration"
description: "Config file location, behavior, and full settings reference."
---

## Location

Nova stores settings at `settings.toml` under your OS config directory:

- Windows: `%APPDATA%\nova\settings.toml`
- macOS: `~/Library/Application Support/nova/settings.toml`
- Linux: `~/.config/nova/settings.toml`

Internally, this is `dirs::config_dir().join("nova").join("settings.toml")`.

## File Lifecycle

- If `settings.toml` does not exist, Nova writes an embedded default config.
- If `settings.toml` exists but TOML parsing fails, Nova overwrites it with defaults.

This means a malformed edit can be "repaired" by reset-to-default (but you may lose changes).

## Defaults

Nova ships embedded defaults (slightly different on macOS).

Non-macOS default excerpt:

```toml
[general]
editor = "nano"
bell = "none"

[status-bar]
visible = true
date-format = "%b %d"
time-format = "%H:%M:%S"

[theme.font]
size = 16
family = "FiraCode Nerd Font"

[theme.colors]
background = "#0D0D0D"
foreground = "#E5E5E5"
accent = "#3ECF8E"
foreground-muted = "#666666"
border = "#FFFFFF12"
cursor = "#3ECF8E"

[theme.cursor]
style = "underline"
blink = false

[keybindings]
new-tab = "ctrl+t"
close-tab = "ctrl+w"
next-tab = "ctrl+tab"
prev-tab = "ctrl+shift+tab"
paste = "ctrl+v"
copy = "ctrl+shift+c"
open-palette = "ctrl+k"

[ai]
provider = "anthropic"
model = "claude-haiku-4-5-20251001"
api_key = ""
```

## Reference

### `[general]`

- `editor` (string): External editor command.
- `bell` (string): One of `none`, `audio`, `blink`.
- `shells` (array of strings, optional): If set and non-empty, Nova uses this list instead of auto-detect.
- `window-controls` (string): One of `traffic-lights` or `system`.
- `scrollback` (integer, optional): Maximum scrollback lines per pane. Defaults to `10000`.

Example:

```toml
[general]
editor = "code -w"
bell = "none"
shells = ["zsh", "bash"]
window-controls = "system"
```

### `[status-bar]`

- `visible` (bool)
- `date-format` (string)
- `time-format` (string)

Example:

```toml
[status-bar]
visible = true
date-format = "%Y-%m-%d"
time-format = "%H:%M"
```

### `[theme.font]`

- `size` (number)
- `family` (string)

Example:

```toml
[theme.font]
size = 15
family = "FiraCode Nerd Font"
```

### `[theme.colors]`

All values are hex strings.

- `#RRGGBB` is supported.
- `#RRGGBBAA` is supported (alpha channel).

Keys:

- `background`
- `foreground`
- `accent`
- `foreground-muted`
- `border`
- `cursor`

Example:

```toml
[theme.colors]
background = "#0D0D0D"
foreground = "#E5E5E5"
accent = "#3ECF8E"
foreground-muted = "#666666"
border = "#FFFFFF12"
cursor = "#3ECF8E"

[theme.cursor]
style = "underline"
blink = true

[ai]
provider = "anthropic"
model = "claude-haiku-4-5-20251001"
api_key = ""
```

### `[theme.cursor]`

- `style` (string): Cursor shape. One of `block`, `beam`, `underline`.
- `blink` (bool): Whether the cursor blinks.

Example:

```toml
[theme.cursor]
style = "underline"
blink = true
```

### `[keybindings]`

See [Keybindings](/keybindings) for syntax and examples.

Keys:

- `new-tab`
- `close-tab`
- `next-tab`
- `prev-tab`
- `paste`
- `copy`
- `open-palette`

### `[ai]`

See [AI](/ai) for behavior and troubleshooting.

- `provider` (string): `anthropic` or `openai`
- `model` (string)
- `api_key` (string)
- `base_url` (string, optional): Override API base URL (useful for proxies).

Example:

```toml
[ai]
provider = "openai"
model = "gpt-4.1-mini"
api_key = "..."
base_url = "https://api.openai.com/v1/"
```
