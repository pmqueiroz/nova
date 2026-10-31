# Architecture

Nova is a GPU-accelerated terminal emulator: a single Rust binary built on
`iced` 0.14 (Elm-style update/view/subscription) that drives one PTY per
pane, parses its output with `vte` into a `Grid`, and renders that grid as
an iced widget. The same binary doubles as a CLI (`nova ask`, `nova
config`, …) when invoked with arguments.

Code is the truth. This file explains the *why* behind the shape it takes.

## Layout

- `src/main.rs` — entry point. `cli::run_from_env()` runs first; if it
  returns an exit code the process exits without opening a window. Otherwise:
  detach the console (Windows), `config::init()`, register notifications,
  seed the runtime theme from config, `ui::start()`.
- `src/cli/` — argument dispatch (`mod.rs`) and one file per subcommand in
  `commands/`. CLI commands that need the running app talk to it over the
  private OSC 777 channel (see `docs/content/internals-osc.md`).
- `src/core/` — platform-free logic: `config` (TOML settings, keybinding
  parsing, shell detection), `grid` (cell buffer, scrollback, alt screen,
  command history), `ai` (Anthropic/OpenAI queries), `url` (link detection).
- `src/sys/` — OS edges: `pty` (`PtyBridge` over `portable-pty`, shell
  bootstrap incl. prompt-command injection), `parser` (`AnsiExecutor`, the
  `vte::Perform` impl that mutates a `Grid`), `bell`, `notification`.
- `src/ui/` — the iced app. `mod.rs` builds window settings and fonts;
  `app_state/` holds the `Nova` state, `Message`, `update/` (split by
  concern: input, mouse, tabs, settings, palette, search, ai, window),
  `view.rs`, `subscription.rs`; `tab.rs` holds `Tab`/`SplitPane`;
  `components/` the widgets; `theme/` colors and fonts; `typography.rs`
  text styling.
- `assets/` — embedded fonts (Fira Code Nerd Font), icons, sounds, and the
  default `settings.toml` (separate macOS variant).
- `build.rs` — Windows icon/resource embedding.
- `docs/` — user docs site (docmd, Bun), deployed to GitHub Pages.
- `packaging/` — NSIS installer template, AUR PKGBUILD.

## Data flow

1. A `PtyKey` subscription per pane (`subscription.rs`) spawns a PTY worker;
   bytes arrive as `Message::PtyOutputReceived(tab_id, bytes)`.
2. `update` feeds the bytes through the pane's `vte::Parser` into
   `AnsiExecutor`, which mutates that pane's `Grid` (and handles OSC 7 cwd,
   OSC 8 links, OSC 777 Nova commands).
3. `view` renders the grid via `components::term`.
4. Keyboard input is encoded in `subscription.rs` (`handle_key_pressed`,
   including kitty keyboard protocol) and sent back as `PtyCommand::Input`.

## Terminal emulation notes (`sys/parser.rs`, `core/grid.rs`)

- C0 controls in `execute`: `0x0D` CR (column 0, clears pending wrap),
  `0x08`/`0x7F` backspace, `0x09` tab (fixed 8-column stops, clamped to the
  last column; no `HTS`/`TBC` support), `0x07` BEL (sets `bell_pending`).
- SGR 8 (hidden) and 28 (reveal) are accepted and ignored: there is no
  hidden attribute in `CellAttrs`.
- `MouseMode` maps to DEC private modes: `Normal` = `?1000`, `Button` =
  `?1002`, `AnyEvent` = `?1003`.

## Assets and credits

- `assets/sounds/bell.flac`: "Notification Sound" by thehorriblejoke,
  <https://freesound.org/s/583554/>, CC0. Embedded by `sys/bell.rs`.
- `assets/fonts/`: Fira Code Nerd Font (regular, bold), embedded by `ui/mod.rs`.

## Block engine (`core/block.rs`)

v2 shows each command as a block instead of a raw scrolling grid.

- `Session` (one per pane): `blocks`, deduplicated `history` (a repeated
  command moves to the end and its count goes up; capped at 2000), the `cwd`,
  the PTY size and the `phase`:
  - `Prompt`: the shell is waiting; Nova's input bar owns the keyboard.
  - `Running(id)`: a command was submitted; PTY output belongs to block `id`.
  - `Raw(id)`: that command switched to the alt screen (vim, htop); the UI
    shows the raw grid full-size and forwards keys untouched.
- `Block`: command text, cwd, start/finish time (`chrono::Local`), exit code,
  `state` (`Pending` until the shell confirms the command started, then
  `Running`, then `Done`) and its own `output: Grid`, sized like the PTY
  screen so cursor movement and in-place redraws behave as in a normal
  terminal. Rendered lines = grid scrollback + screen rows up to the last
  non-blank row or the cursor row (`line_count`, `line`, `text`).
- `submit` refuses empty input and refuses while a command is running.
  `clear` drops every block except the running one.

### Shell integration marks (OSC 777, `sys/pty.rs`)

Nova owns the prompt, so it needs to know where each command starts and ends.
Every supported shell emits private OSC 777 marks (parsed in
`sys/parser.rs` into `ControlCommand`s):

| Mark | Meaning | Emitted by |
|---|---|---|
| `prompt_ready[;preexec]` | Shell is at a prompt, ready for input. `preexec` = this shell will also send `command_start`. | End of every prompt hook. |
| `command_start` | The submitted line was read; output that follows belongs to the command. | zsh `preexec` hook; bash ≥ 4.4 `PS0`; PowerShell `PSConsoleHostReadLine` wrapper (only if PSReadLine is present). |
| `command_complete;<code>` | Previous command finished with exit `code`. Not sent on the first prompt. | bash/zsh/git-bash hook, PowerShell prompt. |
| `command_failure;<code>` | Non-zero exit (drives the AI diagnosis). | Same. |

- The POSIX hook lives once in `POSIX_PROMPT_HOOK` and is used by bash, zsh
  (via `NOVA_PROMPT_COMMAND` + precmd), git-bash (`nova_prompt.sh`) and the
  WSL env. It detects zsh / bash ≥ 4.4 at runtime to decide `preexec`.
- cmd: `PROMPT=$E]777;prompt_ready$E\$P$G` (ST-terminated). No exit codes,
  no preexec: a `prompt_ready` while a command runs ends it with code 0, and the
  echoed command line is skipped.
- Shells that never send `prompt_ready` (fish, WSL distros where the env
  does not cross `wsl.exe`, custom shells) fall back to the classic raw terminal.

## Shared state

- Config: `core::config` keeps a `OnceLock<Config>` (`config::get()`) plus a
  mutex-guarded parsed keybinding set. Saving from the settings UI writes
  `settings.toml` under `dirs::config_dir()/nova/`. An unparsable file is
  overwritten with defaults on startup.
- Runtime theme: `ui::theme::color` holds a mutex-guarded `RuntimeTheme`
  seeded in `main` and updated by the settings UI.
- Overlay flags: `app_state::nova` exposes atomics (`SETTINGS_OPEN`,
  `PALETTE_OPEN`, `AI_OPEN`, `SEARCH_OPEN`, `KB_RECORDING`,
  `ACTIVE_KITTY_FLAGS`) so the keyboard subscription, which can't borrow
  `Nova`, knows how to route keys.
- Animation clock: `Nova.now` is advanced by `Message::Frame(Instant)`.
  `subscription` adds `window::frames()` only while `Nova::is_animating()`
  is true, so an idle window renders no frames. Every animation
  (`iced::Animation`, curves from `theme/motion.rs`) is interpolated against
  `Nova.now` in `view`. `window_pop` plays the `pop` keyframe (400 ms ease,
  scale .96→1, translateY 8→0) when the window opens.
- Everything else lives on `Nova` (tabs, selection, window state) with
  fields scoped `pub(in crate::ui::app_state)`.

## Design system / shared UI

Widgets live in `src/ui/components/`, re-exported from `components/mod.rs`.
Settings screens are composed from `setting_row`, `section_label`,
`color_row` rather than raw iced widgets.

### Tokens

Colors are `Hue` constants in `ui/theme/color.rs` (`BG`, `BG_DEEP`, `FG`,
`ACCENT`, …); user-themable colors go through `theme::color::runtime()`.
Text goes through `ui::typography::Typography`, fonts through
`ui::theme::font`. Don't hardcode colors or fonts in components.

## Window chrome

- `components/title_bar.rs`: 56 px bar, three columns. Traffic-light
  layout (macOS default): lights left (12 px, gap 8, padding-left 24), title
  centered, actions right (`+` new tab, sliders = settings; 40 px icon
  buttons, 20 px icons). `System` window controls (Windows/Linux default):
  actions move left and 48×56 min/max/close buttons sit flush right (close
  hovers `LIGHT_CLOSE`). Title = "Nova" (14 semibold fg-2) + cwd (fg-3).
  The visual bell flashes "Nova" in the accent color.
- Tabs (`components/tab_bar.rs`) replace the centered title whenever more than
  one tab is open: surface-1 r12 pill track (max `min(560, 50% window)`,
  horizontal scroll, hidden scrollbar), tabs h32, min-w 96, width computed from
  the monospace label (`tab_width`), active = surface-2/fg-1, close × (24 px)
  shown only on the active or hovered tab. New tabs play the `pop` keyframe
  through `Tab.appear`. Drag-reorder is position-free: `TabPressed` arms a
  drag, the 6 px move threshold in `mouse.rs` starts it, and `TabHovered`
  over another tab swaps them. Middle-click closes. Right-click on the title
  bar `+` opens the shell picker.
- The terminal area's pixel origin (`helpers::pixel_to_cell`, `calc_grid`
  padding) still assumes the legacy layout below the 56 px bar. It goes away
  with the block UI.

## Platform specifics

- Windows: opaque window with rounded corners, console detached at start,
  PATH registry handling in the NSIS installer, start in home dir when
  launched from the install dir.
- macOS/Linux: transparent window background; custom title bar and traffic
  lights (window decorations are off on every platform).

## Removed in v2

- Status bar (shell, git branch, command timer, date/time) and its
  `[status-bar]` config section. Old config files that still contain the
  section keep loading: serde ignores unknown sections, which matters because
  `config::init` overwrites any file that fails to parse
  (`legacy_status_bar_section_is_ignored` test).

## Known gaps

- Unit tests only (`cargo test`, e.g. `sys/parser.rs`); UI verification is manual (`cargo run`).
