---
title: "CLI"
description: "nova CLI commands and how they integrate with the Nova app."
---

## Commands

Nova includes a small CLI surface:

- `nova help`: Show usage
- `nova ask [preset...]`: Open the Ask AI modal in the running Nova app
- `nova update` (alias `nova --check-update`): Check for a newer release

## `nova update`

Asks GitHub for the latest release and compares it with the running version. Works outside Nova and ignores `general.check-updates`.

```text
$ nova update
nova 1.1.0 is available (current 1.0.0)
https://github.com/pmqueiroz/nova/releases/tag/v1.1.0
```

It only reports; it does not download or replace the binary. Update with the installer or package manager you used originally (winget, Scoop, Homebrew, apt, AUR, …). Exits `1` if the check fails (offline, rate limited).

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

Details are documented in [OSC Integration](/internals-osc).
