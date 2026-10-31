# Coding conventions & authoring rules

How code is written in this project. These are hard rules, not suggestions. This file is about **how to
author** code and where documentation lives.

## No comments in code

Code carries **no comments**. Do not:

- Explain a function with a comment above it.
- Annotate a line because it "might need documenting".
- Leave TODO/NOTE/why-notes inline.

If something needs explaining — a business rule, a non-obvious decision, a
gotcha — it belongs in `.refs/`, **always**. The code says *what*; `.refs/`
says *why*. A comment is a signal that a `.refs/` entry is missing: write the
entry instead.

Name things well enough that the code reads without prose. Extract a
well-named function or variable instead of writing a comment.

## Copy rules

No project-specific copy rules yet. Add them here when they appear.

## Documentation lives in `.refs/`

Every explanation of code behavior or business rule goes in the matching
`.refs/` file (see [AGENTS.md](../AGENTS.md) for the map). When you add or
change a feature, updating the `.refs/` doc is **part of the task**, not a
follow-up — the Operating Protocol in AGENTS.md is binding.

## Components must be reusable, and reused

The UI is built from **shared, reusable pieces** in `src/ui/components/`
(colors from `ui/theme/color.rs`, text via `ui/typography.rs`). One
concept = one component, used everywhere that concept appears. Do not
reimplement the same behavior or interaction in two places.

Before building something new, check whether an existing piece already does
it (or nearly does it). If a near-match exists, **generalize and reuse it**
rather than copy-pasting a variant. If you build something likely to appear
again, extract it to the shared location from the start.

Consistency is the goal: the same interaction should look and behave the same
everywhere in the project.

## Formatting and lint

`cargo fmt` (see `rustfmt.toml`) and `cargo clippy -- -D warnings` are
authoritative; CI rejects either failing. Keep pure logic in `core/`/`sys/`,
out of the view layer.
