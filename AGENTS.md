# Agent Instructions

## .refs — feature & architecture notes

Working references for how this project is built. Code is the truth; these
explain the *why* and the gotchas.

- [conventions.md](./.refs/conventions.md) — authoring rules: **no comments in
  code** (docs go in `.refs/`), reusable-component discipline. Read first.
- [architecture.md](./.refs/architecture.md) — module layout (cli/core/sys/ui),
  PTY → vte → grid → iced data flow, shared state, components and tokens.
- [design-system.md](./.refs/design-system.md) — v2 design handoff source,
  binding size/motion constraints, tokens, iced capability fallbacks.
- [brand.md](./.refs/brand.md) — brand-v2 asset layout, which file each
  platform/surface uses, colors.
- [updates.md](./.refs/updates.md) — update check and install: install-kind
  detection, delegate vs. self-swap, staging, restart, startup cleanup.
- [docs-site.md](./.refs/docs-site.md) — Astro site in `docs/`: landing page,
  `/docs/` template, Markdown directives (callouts, OS tabs, key caps), nav order.
- [ci-cd.md](./.refs/ci-cd.md) — CI (fmt, clippy matrix), release-please,
  tag-driven packaging and package-manager publishing.
- [docs/src/content/docs/](./docs/src/content/docs/) — user-facing docs site (config,
  keybindings, CLI, OSC internals). Update alongside `.refs/` when behaviour
  users see changes.
- [incidents/](./.refs/incidents/README.md) — postmortems + standing rules;
  **read before any migration or destructive op**.

Add more `.refs/` files as the project grows a subsystem worth explaining
(e.g. `observability.md`, `economy.md`, `legal.md`) — one file per concern,
linked from this list.

## Operating Protocol

*The Self-Maintaining Loop:* If you create a new feature, make changes to an
existing one, or modify app lifecycle process, *your final task before
completing the job is to update the corresponding markdown file in the
`.refs/` directory*. The documentation must perfectly reflect the code in the
app. Write every ref update with the `ref-write` skill
(`.agents/skills/ref-write/SKILL.md`).

# Agent Skills
Custom skills and operational workflows are stored in `.agents/skills/`.
When handling tasks matching the definitions inside `.agents/skills/<skill-name>/SKILL.md`, read the corresponding skill file and adhere to its instructions.

CLAUDE Only: Everytime loading the skills create a mirror in .claude/skills with each skill pointing to the original one. Also add a .gitignore file inside .claude/skills with "*"
