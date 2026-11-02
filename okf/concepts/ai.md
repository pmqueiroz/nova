---
type: concept
title: AI
description: "Providers, configuration, and how Nova builds AI requests."
source: "https://nova.pmqueiroz.dev/ai/"
path: /ai/
updated: 2026-11-02
okf:
  generated_by: "@docmd/plugin-okf"
  generated_at: "2026-11-02T00:43:21.499Z"
---
---
title: "AI"
description: "Providers, configuration, and how Nova builds AI requests."
---

## Overview

Press `Ctrl+K` (`Cmd+K` on macOS) or click **Ask AI** in the input bar. The AI
sheet offers:

- **Suggest a command for “…”**: describe what you want in plain words.
- **Fix “cmd”**: shown after a command fails; diagnoses it and proposes a fix.
- **Explain last command**: explains what the last command did.

Answers type in; when the reply contains a command, a card with a **Run**
button (or `Enter`) runs it in the current tab. Requests use your question
plus the output of the relevant block as context.

AI is optional. If `api_key` is empty, requests fail with a clear error.

## Configuration

Settings live under `[ai]` in `settings.toml`:

```toml
[ai]
provider = "anthropic"  # or "openai"
model = "claude-haiku-4-5-20251001"
api_key = ""
# base_url = "https://api.anthropic.com"  # optional
```

Fields:

- `provider`: `anthropic` or `openai`
- `model`: Provider-specific model name
- `api_key`: Required
- `base_url`: Optional override (useful for proxies)

## Provider Behavior

Anthropic:

- Default `base_url` is `https://api.anthropic.com`.

OpenAI:

- Default `base_url` is `https://api.openai.com/v1/`.
- If you set `base_url` without a trailing `/`, Nova adds it.

## What Gets Sent

Nova builds a system prompt including OS and shell, and includes terminal context from the last output.

Terminal context is extracted by scanning for the prompt marker and collecting the lines after it, trimming empty lines.

## Troubleshooting

- "No API key configured": Set `[ai].api_key` in Settings or `settings.toml`.
- Model errors: Confirm the model name exists for the selected provider.
- Proxy/base URL issues: Ensure `base_url` includes scheme and is reachable.
