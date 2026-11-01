# Conventions

Hard rules for authoring code in this project.

## No comments in code

No doc comments, inline notes, or TODOs. Code says *what*; `.refs/` says
*why*. If something needs explaining, write the `.refs/` entry instead —
or extract a well-named function or variable so it doesn't.

Updating the matching `.refs/` file is part of every feature change, not a
follow-up (see the Operating Protocol in [AGENTS.md](../AGENTS.md)).

## Reuse components

UI is built from shared pieces in `src/ui/components/` (DS primitives in
`components/ds/`) and values from `ui/theme/tokens.rs`. One concept = one
component. Before building something, look for an existing piece; if a near
match exists, generalize it instead of copying a variant.

## Layering

Pure logic stays in `core/` and `sys/`, out of the view layer.

## Formatting and lint

`cargo fmt` (`rustfmt.toml`) and `cargo clippy -- -D warnings` are
authoritative; CI rejects either failing.
