---
title: "Development"
description: "Building Nova, working on the codebase, and building the docs site."
---

## Build Nova

Nova uses Rust (stable, edition 2024).

```bash
git clone https://github.com/pmqueiroz/nova.git
cd nova
cargo build --release
```

Binary output:

```text
./target/release/nova
```

For local iteration:

```bash
cargo run
```

## Code Map

High-level ownership:

- `src/core/`: config, grid model, ANSI/VTE state, AI client
- `src/sys/`: PTY bridge, platform I/O, terminal parser
- `src/ui/`: iced application, components, theme

## Build The Docs Site

Docs live in `docs/` and are built with `@docmd/core`.

From the repo root:

```bash
cd docs
bun install -g @docmd/core
docmd dev
```

Build output goes to `docs/dist`:

```bash
cd docs
docmd build
```

## CI Notes

The main CI enforces formatting and linting.

Before opening a PR:

```bash
cargo fmt
cargo clippy
cargo build
```
