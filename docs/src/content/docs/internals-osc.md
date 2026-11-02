---
title: "OSC Integration"
description: "Private OSC sequences used for CLI-to-app integration."
---

Nova supports a small set of OSC (Operating System Command) sequences.

Some are standard-ish (OSC 7, OSC 8), and one is a Nova-private channel used by `nova` CLI commands.

## Private Nova OSC Channel

Nova uses a private OSC code `777`.

Format:

```text
ESC ] 777 ; <command> [ ; <arg> ... ] BEL
```

Where:

- `ESC` is `\x1b`
- Terminator is BEL (`\x07`)

### Command: `ask_ai`

`nova ask` sends:

```text
ESC ] 777 ; ask_ai [ ; <preset_b64> ] BEL
```

If a preset is provided, it is base64-encoded (standard base64, no newlines).

Examples:

No preset:

```text
\x1b]777;ask_ai\x07
```

With preset `"hello"` (base64: `aGVsbG8=`):

```text
\x1b]777;ask_ai;aGVsbG8=\x07
```

## OSC 7: Current Working Directory

Nova parses OSC 7 to track a working directory string.

It expects a `file://` URL or `ssh://` host-like value.

This is used for display/state and may differ from your shell's own concept of PWD.

## OSC 8: Hyperlinks

Nova parses OSC 8 URIs and stores the "current" URI while rendering.

This enables link hover behavior.
