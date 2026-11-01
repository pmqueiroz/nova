---
type: concept
title: Troubleshooting
description: "Common problems and fixes."
source: "https://nova.pmqueiroz.dev/troubleshooting/"
path: /troubleshooting/
updated: 2026-11-01
okf:
  generated_by: "@docmd/plugin-okf"
  generated_at: "2026-11-01T23:42:00.782Z"
---
---
title: "Troubleshooting"
description: "Common problems and fixes."
---

## macOS: "Nova is damaged and can't be opened"

Nova is not notarized. macOS Gatekeeper may block it.

If you installed the app bundle, you can clear the quarantine attribute:

```bash
xattr -cr /Applications/Nova.app
```

## Settings Keep Resetting

Nova overwrites `settings.toml` with defaults if it fails to parse as TOML.

If your settings keep resetting:

- Validate your TOML syntax (missing quotes, stray commas, etc.)
- Keep a backup copy while editing

## `nova ask` Must Be Run Inside Nova

`nova ask` is an integration command that signals the running Nova app via OSC.

If you run it in another terminal, you'll get:

```text
error: 'nova ask' must be run inside Nova terminal
```

Run it inside Nova, or use the in-app command palette action.

## Pagers (`git log`, `man`) Stay Inside a Block

Nova shows a program fullscreen when it switches to the terminal's alternate
screen, or when it turns on bracketed paste (interactive apps such as Claude
Code, REPLs and nested shells over `ssh`). An app that does bracketed paste
stays fullscreen until it exits. `less` skips that switch when its options contain `X`, and
Git runs it with `LESS=FRX` when `LESS` is unset.

Nova therefore starts its shells with `LESS=FR` (pager output that fits on one
screen still prints inline and exits). If you set `LESS` yourself, Nova leaves
it alone: drop the `X` flag from it to get fullscreen pagers.

In PowerShell, `help` (and its alias `man`) normally pipes through Windows'
`more`, which never goes fullscreen and waits for a key after every page.
Nova's PowerShell session replaces `help`: it pages through `less` when one is
on `PATH`, otherwise it prints the whole topic into the block, which you can
scroll.

While a program is fullscreen, the mouse wheel scrolls it: Nova sends arrow
keys, or wheel events if the program asked for mouse reporting.

## AI Errors

- Missing key: set `[ai].api_key`.
- Wrong model/provider: ensure model name matches the selected provider.
- Proxy issues: verify `[ai].base_url` is correct and reachable.
