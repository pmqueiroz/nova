# Contributing to Nova

Thanks for your interest in making Nova better. This document covers how to report bugs, propose features, set up a development environment, and get your changes reviewed.

## TL;DR

- Search existing issues before filing a new one.
- For bug fixes: open an issue, then a pull request.
- For new features: open an issue first and wait for feedback before writing code.
- Run `cargo fmt` and `cargo clippy` before pushing — the CI will reject failures.

## How Contributing Works

Nova is a small, focused project. There is no automated triage agent or formal spec process. The contribution flow is straightforward:

1. **File an issue** describing the bug or feature.
2. **Wait for feedback** on features — this avoids duplicate work on ideas that may not fit the project's direction.
3. **Open a pull request** once the direction is agreed on (or immediately for bug fixes).

## Filing a Good Issue

Search [existing issues](https://github.com/pmqueiroz/nova/issues) before filing to avoid duplicates.

### Bug reports

Include:

- A clear title and a short description of what went wrong.
- Steps to reproduce, with a minimal example where possible.
- Expected vs. actual behavior.
- Nova version (shown in the title bar or release tag), OS, and architecture.
- Screenshots or recordings when the issue is visual.

### Feature requests

Describe the problem you're trying to solve before proposing a solution. Include:

- The user need or pain point.
- What the current behavior is and why it falls short.
- A rough sketch of the desired behavior or workflow.

## Development Setup

You need [Rust](https://rustup.rs/) stable with the 2024 edition.

```sh
git clone https://github.com/pmqueiroz/nova.git
cd nova
cargo build --release
```

The binary lands at `./target/release/nova`.

For a faster iteration loop during development, use:

```sh
cargo run
```

Nova's main crates and what they own:

| Path | Responsibility |
|---|---|
| `src/core/` | Config, grid model, ANSI/VTE state, AI client |
| `src/sys/` | PTY bridge, platform I/O |
| `src/ui/` | iced application, components, theme |

## Before You Push

```sh
cargo fmt                  # format all code
cargo clippy               # lint — fix any warnings before opening a PR
cargo build                # make sure it compiles
```

There is no automated test suite today. Manual testing is required. Run the app, exercise the change end to end, and include screenshots or a short recording in your pull request when the change is visual.

## Opening a Pull Request

1. Fork the repository and create a branch from `main` with a descriptive name (e.g. `fix/pty-resize-crash` or `feat/shell-picker-keyboard-nav`).
2. Keep the PR focused on a single logical change.
3. Fill in the pull request description: what changed and why, plus any manual testing steps the reviewer should follow.
4. Link the related issue using `Closes #<number>` in the description.

Pull requests are reviewed by the maintainer. Feedback will come as review comments — push fixup commits on the same branch to address them.

## Code Style

- Follow standard Rust idioms. `cargo fmt` and `cargo clippy` are the authority.
- Prefer explicit imports over wildcard `use *` except where established by the existing module conventions.
- Keep UI components in `src/ui/components/` and pure logic out of the view layer.
- Write comments only when the *why* is non-obvious from the code itself — not to narrate what the code does.

## Commit Messages

- Use the imperative mood in the subject line: `fix: pty restart on resize`, not `fixed`.
- Keep the subject under 72 characters.
- Explain *why* in the body when the change is non-obvious.

## Licensing

By contributing, you agree that your changes will be licensed under the project's [MIT License](LICENSE).

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). All contributors are expected to uphold it.

## Reporting Security Issues

See [SECURITY.md](SECURITY.md). Do not open public issues for vulnerabilities.
