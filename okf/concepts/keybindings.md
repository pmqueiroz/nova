---
type: concept
title: Keybindings
description: "Keybinding syntax, defaults, and action reference."
source: "https://nova.pmqueiroz.dev/keybindings/"
path: /keybindings/
updated: 2026-11-01
okf:
  generated_by: "@docmd/plugin-okf"
  generated_at: "2026-11-01T17:28:16.639Z"
---
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
open-palette = "ctrl+p"
ask-ai = "ctrl+k"
settings = "ctrl+,"
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
- `ask-ai`
- `settings`

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
open-palette = "ctrl+p"
ask-ai = "ctrl+k"
settings = "ctrl+,"
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
open-palette = "cmd+p"
ask-ai = "cmd+k"
settings = "cmd+,"
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

Font size steps by 2 within 14–20 (odd sizes snap to the next even step). Reset returns to the size that was configured at app start.

### Block input

| Action | Key |
|---|---|
| Run the command in the input bar | `Enter` |
| Previous / next history entry | `Up` / `Down` |
| Complete the word at the end / cycle back | `Tab` / `Shift+Tab` |
| Clear the input (or copy the selection, when text is selected) | `Ctrl+C` |
| Clear all blocks | `Ctrl+L` |

`Tab` completes the last word of the input. Nova does this itself, not the shell: the first word completes to commands from your `PATH` and history, any other word (or one with a `/`) to files and folders relative to the pane's current directory (`~` is your home). One match is inserted outright (folders end in `/`, everything else gets a trailing space); several matches first fill in their common prefix, then each further `Tab` cycles through them (`Shift+Tab` goes backwards). Names with spaces are quoted. Shell-specific completions (git branches, PowerShell cmdlets, option flags) are not available.

While a command runs, every key goes to that program (`Ctrl+C` interrupts it).
Full-screen programs (vim, htop, less) take over the whole pane until they exit.

### Search

| Action | macOS | Other |
|---|---|---|
| Open search bar | `Cmd+F` | `Ctrl+F` |
| Next match | `Enter` | `Enter` |
| Previous match | `Shift+Enter` | `Shift+Enter` |
| Close search | `Escape` | `Escape` |

Search is case-insensitive and covers the output of every block in the active pane (or the scrollback and visible rows in classic mode). The current match is highlighted in yellow; other matches are dimly highlighted. The match count is shown in the search bar (e.g. `3 / 15`). Searching closes automatically when switching tabs.

## Notes

- If `open-palette`, `ask-ai` or `settings` is invalid, Nova falls back to its default (`ctrl+p`, `ctrl+k`, `ctrl+,`; `cmd` on macOS).
- Configs still using the old default palette binding (`ctrl+k` / `cmd+k`) or the old default accent (`#3ECF8E`) are migrated to the new defaults on startup.
