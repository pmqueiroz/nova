---
title: "AI"
description: "Providers, configuration, and how Nova builds AI requests."
---

## Overview

Nova's AI features live in the UI (Ask AI and Explain Error). They use:

- Your question
- Terminal context extracted from the last command output

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
