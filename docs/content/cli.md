---
title: "CLI"
description: "nova CLI commands and how they integrate with the Nova app."
---

## Commands

Nova includes a small CLI surface:

- `nova help`: Show usage
- `nova ask [preset...]`: Open the Ask AI modal in the running Nova app

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
