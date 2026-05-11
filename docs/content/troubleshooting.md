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

## AI Errors

- Missing key: set `[ai].api_key`.
- Wrong model/provider: ensure model name matches the selected provider.
- Proxy issues: verify `[ai].base_url` is correct and reachable.
