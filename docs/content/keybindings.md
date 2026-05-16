---
title: "Keybindings"
description: "Keybinding syntax, defaults, and action reference."
---

## Binding Syntax

Bindings are strings like `ctrl+t` or `ctrl+shift+tab`.

Supported modifiers:

- `ctrl`
- `shift`
- `alt` (alias: `option`)
- `cmd` (aliases: `command`, `meta`, `super`)

Supported keys:

- `tab`
- Any single character (like `t`, `w`, `c`, `v`, `k`)

Examples:

```toml
[keybindings]
new-tab = "ctrl+t"
close-tab = "ctrl+w"
next-tab = "ctrl+tab"
prev-tab = "ctrl+shift+tab"
copy = "ctrl+shift+c"
paste = "ctrl+v"
open-palette = "ctrl+k"
```

## Actions

These keys live under `[keybindings]` in `settings.toml`:

- `new-tab`
- `close-tab`
- `next-tab`
- `prev-tab`
- `copy`
- `paste`
- `open-palette`

## Defaults

Defaults differ on macOS vs other platforms.

Non-macOS (defaults):

```toml
[keybindings]
new-tab = "ctrl+t"
close-tab = "ctrl+w"
next-tab = "ctrl+tab"
prev-tab = "ctrl+shift+tab"
paste = "ctrl+v"
copy = "ctrl+shift+c"
open-palette = "ctrl+k"
```

macOS (defaults):

```toml
[keybindings]
new-tab = "cmd+t"
close-tab = "cmd+w"
next-tab = "ctrl+tab"
prev-tab = "ctrl+shift+tab"
paste = "cmd+v"
copy = "cmd+c"
open-palette = "cmd+k"
```

## Fixed Shortcuts

These shortcuts are hardcoded and cannot be changed via `settings.toml`.

### Split Pane

| Action | macOS | Other |
|---|---|---|
| Open split pane | `Cmd+Shift+T` | `Ctrl+Shift+T` |
| Close split pane | `Cmd+Shift+W` | `Ctrl+Shift+W` |

When a split pane is open, `close-tab` (`Cmd+W` / `Ctrl+W`) closes the **active pane** instead of the entire tab.

The divider between panes is draggable. Hover over it to see a resize cursor, then click and drag to adjust the split ratio. Grids resize when you release the mouse. The ratio is clamped to 10%–90%.

### Font Size

| Action | macOS | Other |
|---|---|---|
| Increase font size | `Cmd+=` or `Cmd++` | `Ctrl+=` or `Ctrl++` |
| Decrease font size | `Cmd+-` | `Ctrl+-` |
| Reset font size | `Cmd+0` | `Ctrl+0` |

Font size steps snap to even numbers (e.g. 15 → 16 → 18 → 20). Range: 8–72. Reset returns to the size that was configured at app start.

### Search

| Action | macOS | Other |
|---|---|---|
| Open search bar | `Cmd+F` | `Ctrl+F` |
| Next match | `Enter` | `Enter` |
| Previous match | `Shift+Enter` | `Shift+Enter` |
| Close search | `Escape` | `Escape` |

Search is case-insensitive and covers both the scrollback buffer and the visible terminal rows. The current match is highlighted in yellow; other matches are dimly highlighted. The match count is shown in the search bar (e.g. `3 / 15`). Searching closes automatically when switching tabs.

## Notes

- If `open-palette` is invalid, Nova falls back to `ctrl+k`.
