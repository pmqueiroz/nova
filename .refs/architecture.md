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
- PowerShell reports `$LASTEXITCODE`, which cmdlet/command-not-found errors
  never set; the prompt reads `$?` first and maps `CommandNotFoundException`
  to 127 (other failures to 1) so the AI Fix flow sees the same codes as POSIX
  shells.
- ConPTY (Windows) passes OSC sequences through immediately but renders text
  on its own schedule, so OSC marks can overtake the output they follow. On
  Windows PowerShell the boundary marks are therefore text: private-use
  characters U+F8F0 (`prompt_ready`), U+F8F1 (`prompt_ready;preexec`) and
  U+F8F2 (`command_start`), which `AnsiExecutor::print` swallows into
  `ControlCommand`s (they never render, in any mode). Because ConPTY replays
  its screen buffer, the readline wrapper clears the console
  (`[Console]::Clear()`) before writing `command_start` with
  `$Host.UI.WriteLine` (Unicode-safe, `[Console]::Write` would turn it into
  `?`), so marks from earlier prompts can never be replayed mid-command.
  `command_complete` only records the exit code; the block ends at the next
  `prompt_ready`. A replayed `command_start` is a no-op (`begin_output` only
  moves `Pending` blocks).
- cmd: `PROMPT=<U+F8F0>$P$G` (text mark, for the same ConPTY ordering reason).
  No exit codes and no preexec: the mark ends the running block with code 0,
  and the echoed command line is skipped. ConPTY replays its buffer on a full
  repaint (window resize), so resizing during a long cmd command can end its
  block early (known gap).
- Git Bash runs under ConPTY too: Nova sets `NOVA_TEXT_MARKS=1`, which makes
  the POSIX hook print the UTF-8 bytes of the text marks instead of OSC, and
  `PS0` clear the console (`\e[H\e[2J\e[3J`) before `command_start`, the
  bash counterpart of the PowerShell `Clear()`.
- Shells that never send `prompt_ready` (fish, WSL distros where the env
  does not cross `wsl.exe`, custom shells) fall back to the classic raw terminal.

### Byte router (`core/router.rs`)

`route()` replaces the plain parse loop in `handle_pty_output` for tabs and
split panes. It feeds the vte parser byte by byte (one parser per pane) and,
after every byte, checks whether a mark was just parsed, so a single PTY
chunk carrying echo + `command_start` + output + `command_complete` + the
next prompt is split correctly.

- Target grid: `Prompt` phase, a still-`Pending` block, and any non-block
  mode go to the pane's screen grid (prompt drawing and echo stay hidden in
  block mode). `Running`/`Raw` go to the block's grid.
- `Integration` per pane: `Detecting` until the first `prompt_ready` switches
  it to `Blocks`; `expire()` after 8 s without one switches to `Classic`, the
  old raw-terminal behavior. A later `prompt_ready` (slow shell start) still
  upgrades an idle classic pane to blocks. `enabled = false` (`tab::BLOCKS_ENABLED`) keeps
  every pane `Classic`; it is off until the block UI lands.
- Shells without preexec: `on_submit` sets `awaiting_echo`, and the first
  `
` after submit starts the block (skips the echoed line).
- ConPTY positions its cursor absolutely, so a block's grid starts at the
  screen grid's cursor row and `Block::first_line` trims the blank lines
  above the first output.
- Alt screen inside a block → `Phase::Raw` (the block grid itself is shown
  full-size); leaving it → back to `Running`.
- Responses (DA, etc.) and non-block controls are returned in `Routed` and
  pushed back into the screen grid's queues, so the existing handlers
  (clipboard, AI triggers, notifications) are unchanged.

### Submitting (`update/session.rs`, `tab.rs`)

- Each pane (`Tab` or `SplitPane`) owns an `InputBox`: the input text, a
  history cursor with the unsent draft, and a `submitted` counter that drives
  the enter-button pulse. `Tab::pane_mut` / `active_pane_mut` / `panes_mut`
  give one `PaneMut` view over either pane type, so session code is written
  once.
- `InputSubmit` in block mode: `clear`/`cls` clear the blocks locally (the
  design's `clear`); anything else goes through `Session::submit`, then
  `Integration::on_submit`, then writes `command + 
` to the PTY. Submits while
  a command is running, or outside block mode, only pulse.
- Readline shells (bash, zsh, git-bash, sh; `router::is_readline_shell`)
  get a Ctrl+U prefix on every submit (`Integration::submit_bytes`): a no-op
  on an empty line that absorbs the byte readline can swallow while starting
  up (seen as the first command losing its first character in Git Bash).
- `InputHistory(-1/+1)` walks the deduplicated session history; the draft
  comes back when walking past the newest entry.
- While a block-mode command runs (`Phase::Running`), `INPUT_OWNS_KEYS` is
  off: the input bar shows as disabled and every key goes through the legacy
  encoder in `subscription.rs` to the PTY. That covers passwords, `Read-Host`,
  REPLs and Ctrl+C (``). If the platform reports Ctrl+letter as the control
  character itself, it is forwarded unchanged.
- `Tick` (1 s) calls `Integration::expire` on every pane.
- History: `Session.history` is the block-mode source of truth. It is
  loaded from `command_history.bin` (bincode, `helpers::command_history_path`)
  into every pane at startup, copied from the active pane into new tabs and
  splits, and saved after every submit. Classic panes keep using
  `Grid.command_history` for the legacy inline ghost suggestion.

### Selection and links in blocks (`core/selection.rs`)

- Each output card is a `mouse_area`; `on_move` reports the pointer relative to
  the card, mapped to `(line, col)` with the card padding (24/16) and the
  output font's cell size (`tokens::block`). State lives on `Nova`:
  `block_pointer`, `block_selection` (`BlockSelection`, scoped to one block of
  one pane) and `block_url`.
- Press starts a selection, moving with the button held extends it, release
  ends it (also on a global release). A click without a drag clears the
  selection and focuses the input. Selected cells render fg-1 on accent α .32;
  full middle lines stop at the last visible character.
- The URL under the pointer (`url_at` → `core::url::detect_urls`) is
  underlined with a pointer cursor; Ctrl+click opens it.
- Copy: the copy keybinding copies the block selection. In the input bar,
  Ctrl+C copies when a selection exists (`HAS_BLOCK_SELECTION`), otherwise it
  clears the input.

### Search

Ctrl/⌘+F opens `components::search_bar`: a floating 384×48 DS bar (window
gradient, r12, line-2 edge, drop shadow) top-right of the body, with a
search icon, `Find` input (red when nothing matches), `n / N` counter,
chevron prev/next and close. In block mode `recompute_search` runs
`core::search::search_session` over the active pane's blocks
(case-insensitive, per output line) into `Nova.block_matches`; matches render
on yellow α .32 and the current one solid yellow with on-accent text.
Next/prev snaps the log to the current match's relative position
(`relative_position`, an approximation by line count). Classic panes keep
the legacy grid search.

### Split panes

Each pane of a split is its own session (log + input bar). Panes are
separated by a 1 px `LINE` divider (8 px drag handle) and each has a DS close
icon button (32 px, top-right). There is no active-pane border: the focused
input bar's ring marks the active pane. Clicking a pane's log sends
`FocusPane(id)`, which makes it active and focuses its input.

### AI (`update/ai.rs`, `core/ai.rs`)

`start_ai(kind)` builds one prompt per kind (`question_for`): Generate uses
the typed query; Diagnose uses the last failed block's command and exit code;
Explain the last finished command. The context is that block's output (classic
panes fall back to `extract_last_output`). With no previous command, Explain
answers locally. Replies go through `split_answer`, which flattens the prose
and takes the first fenced code block's first line as the runnable command.
`nova ask "q"` (OSC 777 `ask_ai`) opens the sheet and runs Generate;
`nova explain` runs Diagnose when there is a failure, else Explain.

### Raw takeover

When the running block enters the alt screen (`Phase::Raw`),
`Nova::pane_body` swaps the block log and input bar for the legacy
`components::term` widget showing that block's own grid at the output font
size, inside the log padding. `PaneRef::input_grid` makes the kitty keyboard
flags come from that grid; keys reach the program through the running-command
path. Leaving the alt screen restores the log (the alt-screen content is not
kept in the block) and refocuses the input. Mouse reporting in raw mode is
not wired yet (known gap). `term` no longer paints a background, so classic
and raw panes sit on the window gradient.

### Sizing (`helpers::calc_block_grid`, `update/window.rs`)

Panes are sized per mode by `Nova::resize_panes` (window resize, font
change, a pane entering/leaving block mode, integration timeout):

- Classic panes keep the legacy `calc_grid*` math.
- Block/detecting panes: the PTY width equals the block output column, so
  programs wrap exactly at the card edge. Width = pane width − 2×32 (scroll
  padding) − 2×24 (output card padding) − 64 if timestamps are shown; height =
  window − 56 (title bar) − 72 (input bar + margin) − scroll/card paddings.
  Cell size uses the output font (`font size − 2`) and line height 1.5.
- `apply_size` resizes the screen grid, every block grid (reflow via
  `Grid::resize`) and the PTY together.

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
