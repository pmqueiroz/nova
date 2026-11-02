# Architecture

Nova is a terminal emulator: one Rust binary on `iced` 0.14 that drives one
PTY per pane, parses its output with `vte` into a `Grid`, and renders it. The
same binary is a CLI (`nova ask`, `nova config`, …) when given arguments.

## Layout

- `src/main.rs` — `cli::run_from_env()` runs first; if it returns an exit
  code, no window opens. Otherwise: detach console (Windows),
  `config::init()`, notifications, runtime theme, `ui::start()`.
- `src/cli/` — subcommands. Those that need the running app talk to it over
  private OSC 777 (see `docs/content/internals-osc.md`).
- `src/core/` — platform-free logic: config, grid, block/session, byte
  router, completion, search, selection, url, glyph, ai.
- `src/sys/` — OS edges: `pty` (`PtyBridge`, shell bootstrap), `parser`
  (`AnsiExecutor`, the `vte::Perform` impl), bell, notification.
- `src/ui/` — the iced app. `app_state/` holds `Nova`, `Message`, `update/`
  (split by concern), `view.rs`, `subscription.rs`; `tab.rs` holds
  `Tab`/`SplitPane`; `components/` widgets; `theme/` tokens, motion, fonts.
- `assets/` — embedded fonts, icons, brand, sounds, default `settings.toml`
  (separate macOS variant). See [brand.md](./brand.md).
- `docs/` marketing site + user docs (Astro, see [docs-site.md](./docs-site.md)) · `packaging/` NSIS template, AUR PKGBUILD.

## Data flow

1. A `PtyKey` subscription per pane spawns a PTY worker; bytes arrive as
   `Message::PtyOutputReceived(tab_id, bytes)`.
2. `core::router::route` feeds them through the pane's `vte::Parser` into
   `AnsiExecutor`, which mutates the target `Grid` and emits `ControlCommand`s
   (OSC 7 cwd, OSC 8 links, OSC 777).
3. `view` renders blocks (block mode) or the grid via `components::term`
   (classic / raw takeover).
4. Keys are encoded in `subscription.rs` (incl. kitty keyboard protocol) and
   sent as `PtyCommand::Input`.

**PTY spawn failure.** `initial-command` is spawned directly, not via the
shell, so a missing program fails `PtyBridge::new`. The worker then sends
`spawn_error_message` as output and returns, so the pane gets `PtyExited`
instead of a panicked thread behind a dead window. portable-pty's
`CreateProcessW` error text contains NULs, which are stripped. The worker
drops its own `tx_out` clone after a successful spawn, otherwise `rx_out`
never closes when the shell exits.

**Sideloaded ConPTY (Windows).** portable-pty loads `conpty.dll` from beside
the exe before falling back to kernel32's ConPTY; that dll runs the
`OpenConsole.exe` next to it. Nova ships both (pinned NuGet
`Microsoft.Windows.Console.ConPTY`, see `packaging/conpty/fetch.ps1`). The
inbox conhost swallows `?1000h/1002h/1003h/1006h`, so `Grid::mouse_mode`
stays `None` and the wheel falls back to arrow keys (Claude Code then warns
"Scroll wheel is sending arrow keys"). OpenConsole forwards them. Both hosts
emit `?9001h` (win32-input-mode), which Nova ignores; plain VT input still
works. Bun apps receive SGR wheel reports verbatim; Node (libuv) apps drop
them.

## Terminal emulation (`sys/parser.rs`, `core/grid.rs`)

- Tabs are fixed 8-column stops; no `HTS`/`TBC`. SGR 8/28 (hidden) are
  ignored — `CellAttrs` has no hidden attribute.
- **BCE:** erases and scrolled-in lines use `Grid::blank_cell()` with the
  current background. ConPTY paints full-width backgrounds as `SGR bg` + `EL`
  rather than spaces, so without BCE diff lines and TUI status bars vanish.
- Truecolor accepts `38;2;r;g;b` and the colon forms (`colon_rgb`).
- `Grid::resize` on the alt screen also reflows the saved primary buffer,
  otherwise leaving the alt screen restores a buffer of the old size. Reflow
  never paints with the pen background and drops trailing blank cells per
  logical line, so padding from a wider width doesn't become empty rows.
- Block elements U+2580–259F are drawn as rectangles (`core::glyph`,
  `term::block_cell`), not from the font: font glyphs don't fill the cell, so
  pixel art showed gaps. Box drawing still comes from the font. Block cards
  do the same in `output_lines.rs`: `shape` blanks them to spaces
  (`without_block_glyphs`) and `draw_block_glyphs` fills their rects over
  the full `LINE_HEIGHT`. Otherwise explicit cell backgrounds (Claude Code's
  logo paints `48;2;0;0;0`) show through the gaps as black bars.

## Block mode

Each command is shown as a block instead of a scrolling grid.

### Session and blocks (`core/block.rs`)

- `Session` (one per pane) has `blocks`, deduplicated `history` (capped
  2000), `cwd`, PTY size and `phase`: `Prompt` (input bar owns keys),
  `Running(id)` (output goes to block `id`), `Raw(id)` (block is on the alt
  screen; raw grid shown full-size, keys forwarded).
- `Block` owns an `output: Grid` sized like the PTY so in-place redraws work.
  Visible lines = scrollback + screen rows up to the last non-blank row or the
  cursor. Resolving that range scans the whole screen, so callers walking many
  lines must take `Block::lines()` once rather than `line(i)` in a loop.
- Exit 141 (SIGPIPE) is not an error: Git exits with it when the pager is
  quit early.

### Shell integration marks (`sys/pty.rs`)

Nova owns the prompt, so shells report command boundaries via private
OSC 777, parsed into `ControlCommand`s:

| Mark | Meaning | Emitted by |
|---|---|---|
| `prompt_ready[;preexec]` | At a prompt. `preexec` = will also send `command_start`. | Every prompt hook. |
| `command_start` | Line read; following output belongs to the command. | zsh `preexec`; bash ≥ 4.4 `PS0`; PowerShell `PSConsoleHostReadLine` wrapper (needs PSReadLine). |
| `command_complete;<code>` | Previous command's exit code. Not on first prompt. | All hooks. |
| `command_failure;<code>` | Non-zero exit (drives AI diagnosis). | All hooks. |

- One POSIX hook (`POSIX_PROMPT_HOOK`) serves bash, zsh, git-bash and WSL;
  it detects zsh / bash ≥ 4.4 at runtime to decide `preexec`.
- PowerShell reads `$?` before `$LASTEXITCODE` (cmdlet errors never set the
  latter) and maps `CommandNotFoundException` to 127, others to 1, to match
  POSIX codes.
- **ConPTY ordering:** ConPTY passes OSC through immediately but renders text
  on its own schedule, so OSC marks can overtake their output. Under ConPTY
  the marks are therefore private-use characters (U+F8F0 `prompt_ready`,
  U+F8F1 `prompt_ready;preexec`, U+F8F2 `command_start`) that
  `AnsiExecutor::print` swallows.
  - ConPTY also replays its screen buffer, so the readline wrapper clears the
    console before writing `command_start` (via `$Host.UI.WriteLine`;
    `[Console]::Write` turns it into `?`). A replayed `command_start` is a
    no-op; the block ends at the next `prompt_ready`.
  - Git Bash: `NOVA_TEXT_MARKS=1` makes the POSIX hook print text marks, and
    `PS0` clears the screen as the PowerShell wrapper does.
  - cmd: `PROMPT=<U+F8F0>$P$G`. No exit codes or preexec — the mark ends the
    block with 0 and the echoed line is skipped.
- Shells that never send `prompt_ready` (fish, WSL where env doesn't cross
  `wsl.exe`, custom shells) fall back to classic mode.

### Byte router (`core/router.rs`)

`route()` feeds the parser byte by byte and checks for a mark after each, so
one chunk carrying echo + `command_start` + output + `command_complete` +
prompt is split correctly.

- Target grid: `Prompt`, a `Pending` block and non-block modes go to the
  pane's screen grid (keeps prompt and echo hidden); `Running`/`Raw` go to the
  block grid.
- **cwd (OSC 7)** also emits `ControlCommand::Cwd`, which the router applies
  to the screen grid's `pwd` whatever grid parsed it. The tab title, prompt and
  `Session.cwd` read the screen grid, and the hooks print OSC 7 before the
  prompt mark (ConPTY can reorder them too), so the OSC often lands while the
  block is still `Running`. Without this, `cd` showed up late or not at all.
- `Integration`: `Detecting` until the first `prompt_ready` → `Blocks`; after
  8 s without one (`expire`, driven by the 1 s `Tick`) → `Classic`. A late
  `prompt_ready` still upgrades an idle classic pane.
- Shells without preexec: `on_submit` sets `awaiting_echo`; the first `\n`
  after submit starts the block.
- ConPTY positions its cursor absolutely, so a block grid starts at the
  screen cursor row; `Block::first_line` trims the blank lines above.
- Alt screen or bracketed paste inside a block → `Phase::Raw` (see
  [Raw takeover](#raw-takeover)).
- **Leaving the alt screen** keeps the last frame (`keep_last_frame`):
  `Grid::leave_alt_screen` snapshots visible rows (trailing blanks trimmed),
  then `Grid::keep_frame` moves the primary screen's used rows and the frame
  into scrollback and clears the screen. It must not write the frame to the
  screen: right after `?1049l` ConPTY repaints its primary buffer and would
  wipe it.
- Responses (DA, …) and non-block controls come back in `Routed` and are
  pushed into the screen grid's queues, so existing handlers are unchanged.

### Pagers

- Pagers don't enable bracketed paste, so for them raw takeover means the
  alt screen. Git exports `LESS=FRX` when
  `LESS` is unset, and `X` skips the alt screen, so `nova_env` sets `LESS=FR`
  unless the user has `LESS`. `F` keeps short output inline.
- PowerShell's `help` pipes to `more.com`, which blocks without an alt
  screen. The integration script redefines `global:help` to pipe
  `Get-Help @args` through `less` when available, else print directly.

### Input bar and submit (`update/session.rs`, `tab.rs`)

- Each pane owns an `InputBox` (text, history cursor + draft, completion
  state, submit pulse). `PaneMut`/`PaneRef` abstract over `Tab` and
  `SplitPane` so session code is written once.
- `clear`/`cls` clear blocks locally; anything else goes through
  `Session::submit` → `Integration::on_submit` → PTY.
- Readline shells (`router::is_readline_shell`) get a Ctrl+U prefix on every
  submit: readline can swallow a byte while starting up, which lost the first
  character of the first command in Git Bash.
- While a command runs, `INPUT_OWNS_KEYS` is off and keys go through the
  legacy encoder to the PTY (passwords, `Read-Host`, REPLs, Ctrl+C).
- History: `Session.history` is the source of truth, persisted to
  `command_history.bin` (bincode) after every submit, loaded into every pane
  at startup and copied into new tabs/splits. Classic panes still use
  `Grid.command_history` for the inline ghost suggestion.

### Tab completion (`core/completion.rs`)

The shell never sees the input bar's text, so Nova completes locally.

- Always completes the **last word** — iced's `text_input` doesn't expose the
  cursor position.
- Command position: history first words + `PATH` executables (`PATHEXT` on
  Windows, exec bit on Unix). Otherwise: entries of the word's directory,
  resolved against `Session.cwd`; `~` is home, and in readline shells on
  Windows `/c/...` maps to `C:\...`.
- `\` is a separator only on Windows outside readline shells (escape in
  bash). Matching is case-insensitive only on Windows.
- Several matches: insert the common prefix if longer, else cycle
  (`Completion`, `step(±1)`). Any edit, history walk or submit drops it.
- Runs synchronously in `update`; a slow network drive in `PATH` stalls the
  UI.

### Block log rendering (`components/output_lines.rs`)

Every wheel tick and pointer move re-runs `view`, so a block's output is one
`OutputLines` widget rather than a `rich_text` per line:

- Layout is O(1) (`lines × size × LINE_HEIGHT`), the same fixed line height
  the pointer-to-cell mapping assumes.
- `draw` shapes only lines in the clipped viewport and re-shapes a line only
  when its runs differ from the cached paragraph.
- Selection/URL/search marks come from a per-block `MarksFn` built in
  `output_card`.

### Selection, links, search

- Output cards report pointer moves relative to the card, mapped to
  `(line, col)` with card padding and `tokens::block` cell size. Selection is
  scoped to one block (`BlockSelection`). Ctrl+click opens the URL under the
  pointer. In the input bar, Ctrl+C copies when a block selection exists,
  otherwise clears the input.
- Search runs `core::search::search_session` over the active pane's blocks.
  Jumping to a match uses `relative_position`, an approximation by line
  count. Classic panes use the legacy grid search.

### Raw takeover

Triggered by the alt screen (`?1049`/`?47`/`?1047`) or by the command
enabling bracketed paste (`?2004h`). Inline TUIs such as Claude Code (Ink)
never use the alt screen; under ConPTY they only send `?2004h`, `?1004h`,
`?2026`, `?25l` on the primary buffer.

- Bracketed paste sets the sticky `Block::takeover`: the block stays `Raw`
  until the command finishes, because REPLs toggle `?2004` per input line
  and would flicker. The block grid only sees modes set after
  `command_start`, so the shell's own prompt-time `?2004h` never counts.
- A finished takeover (`Block::keeps_width`) keeps its fullscreen width:
  `Session::resize` and `needs_size` skip it, since reflowing an absolutely
  positioned frame to card width wraps its dividers and right-aligned text.
  Every other block that doesn't match the card size is resized.
- The card clips the wider frame without a scrollbar. `Block::scroll_col`
  shifts drawing inside a clip layer (so nothing paints into the card
  padding) and is added to the hit column in `handle_block_pointer`. It only
  changes while dragging a selection on the first/last visible column
  (`Block::scroll_to_col`, clamped to the frame width).

In `Phase::Raw`, `Nova::pane_body` swaps the log and input bar for
`components::term` showing the block's own grid; `PaneRef::input_grid` makes
kitty keyboard flags come from that grid. Wheel: a mouse report when the
program enabled mouse reporting, otherwise 3 arrow keys per notch (alternate
scroll, as xterm does). Clicks and drags are not forwarded.

### Sizing (`helpers::calc_block_grid`, `update/window.rs`)

`Nova::resize_panes` sizes each pane per mode:

- Block panes: PTY width equals the output card's text width, so programs
  wrap exactly at the card edge. `calc_block_grid` and the pointer mapping
  must use the same paddings (incl. compact).
- Raw takeover: `calc_raw_grid` uses `term`'s own metrics and the padding
  passed to `term()` — nothing else may wrap it, or sizing and rendering
  disagree. Only the running block's grid and the PTY resize. `update`
  compares `raw_panes()` before/after routing to resize on takeover
  enter/leave.
- Classic panes keep the legacy `calc_grid*` math; their pixel mapping
  (`helpers::pixel_to_cell`) assumes the legacy layout below the title bar.

### Initial window size

The window opens at `INITIAL_WINDOW_SIZE`, then `fit_window` resizes to 16:10
at 65% of the monitor width (capped at 80% of its height), in logical px, and
recenters. A fixed size overflowed scaled screens (1080p @150% is 1280×720
logical). On Windows the window starts hidden until the fit is applied.

## App behavior

- **Global shortcuts** (palette, Ask AI, settings) are checked before any
  sheet captures keys: the same shortcut closes its sheet, a different one
  switches. The last tab with a live shell can't be closed by shortcut or ×
  (`is_last_live_tab`).
- **AI** (`update/ai.rs`): one prompt per kind — Generate (typed query),
  Diagnose (last failed block), Explain (last finished command). The block's
  output is the context. `split_answer` takes the first fenced code block's
  first line as the runnable command.
- **Inline failure hint** (`ai.diagnostic_banner`): `command_failure;<code>`
  (1–127, API key set) queues a diagnosis. The router has already finished
  the block, so blocks mode sends `session.last_failure()`'s command + text;
  classic sends `extract_last_output` with no command. `core::ai::should_diagnose`
  skips the request locally for empty output or a no-match exit (code 1 from
  `grep`/`rg`/`diff`/`test`/`which`/… unless piped/chained).
  `diagnose_question` sends the tail of the output (60 lines / 4000 chars) and
  tells the model to answer `{"skip": true}` for expected exits, self-explanatory
  errors, or when it can only restate the exit code; otherwise
  `{"message", "command"}` naming the concrete cause. Nothing renders while
  loading: `ai_diagnostic_inflight` holds the code, input clears it, and a
  late or skipped response leaves `diagnostic_banner` empty.
  `parse_diagnosis` tolerates code fences, the legacy `command_to_solve` key,
  and falls back to `split_answer` for non-JSON replies.
- **Split panes** are independent sessions; the focused input ring marks the
  active pane (no border).
- **Drag a tab into a split** (`update/tabs.rs`, `split_drop_side` in
  `helpers.rs`): `TabPressed` records the previously active tab as
  `drag_host_tab` (by id, since reordering moves indices). While dragging,
  the cursor below the title bar in the outer 25% of the width sets
  `split_drop`; the view then shows the host tab (`shown_tab_index`) with
  `components::split_drop_preview` over the target half. The host is the
  previously active tab, or the dragged tab's neighbour when that's the one
  being dragged; neither may already have a split (one split per tab). On
  release `merge_tab_into_split` turns the dragged `Tab` into a `SplitPane`
  (`From<Tab>`) on the right, or for the left side makes it the primary and
  the host the split; ratio resets to 0.5 and grids resize. Tab-only fields
  (`initial_command`, `wait_after_exit`) are dropped.
  Once the cursor leaves the title bar, `components::tab_ghost` follows it:
  a skeleton mini-window (header with the last segment of the tab's cwd,
  `helpers::dir_name`, and placeholder bars),
  not a render of the real content. It is stacked over the whole app, so it
  is positioned with padding from the window origin (`ghost_origin`,
  clamped inside the window), and its border turns accent while over a
  drop zone.
  iced's `clip` is rectangular, so the header's top corners are rounded
  to `R8 - border` and the card is padded by the border width; otherwise
  the header's square corners leak past the rounded border.
- **PTY subscription identity:** iced keys `Subscription::run_with` by the
  data *and* the builder fn pointer, so tab and split panes must share the
  same `pty_stream` fn. With separate closures, moving a pane between tab
  and split (tab drag-to-split, closing the left pane) restarts its shell.
- **Update check** (`core/update.rs` pure, `sys/update/` network): GET
  GitHub's `releases/latest` via blocking `ureq` on `spawn_blocking`, 10 s
  timeout, compared with `CARGO_PKG_VERSION` using `semver` (so `1.10 > 1.9`;
  drafts/prereleases ignored). Runs on `WindowOpened` and every
  `CHECK_INTERVAL` (24 h, a subscription only while `general.check-updates`).
  `Nova.update_status` drives an orange-tint (`Variant::Orange`) "Update" button after
  the settings icon in the title bar actions, and the
  About tab; failures show only in About. Installing the update (delegate to
  the package manager, or verify + swap the binary) is in
  [updates.md](./updates.md). `nova update [--install]` is the CLI side.
- **Tabs** are labeled with the last segment of their cwd
  (`helpers::dir_name`), not the full path. The hidden close × renders a `Space`, not a transparent icon —
  iced's svg tint drops alpha, so a transparent icon draws black.

## Shared state

- Config: `OnceLock<Config>` (`config::get()`) + mutex-guarded keybindings.
  Saved to `dirs::config_dir()/nova/settings.toml`. **An unparsable file is
  overwritten with defaults on startup**, so removed sections (e.g. v1's
  `[status-bar]`) must stay parseable — serde ignores unknown sections
  (`legacy_status_bar_section_is_ignored`).
- Runtime theme: mutex-guarded `RuntimeTheme` in `ui::theme::color`.
- Overlay atomics in `app_state::nova` (`SETTINGS_OPEN`, `INPUT_OWNS_KEYS`,
  `ACTIVE_KITTY_FLAGS`, …): the keyboard subscription can't borrow `Nova`.
- Animation: `window::frames()` is subscribed only while
  `Nova::is_animating()`, so an idle window renders nothing. Animations
  interpolate against `Nova.now`.

## Platform specifics

- Window decorations are off everywhere (custom title bar). Windows: opaque
  with rounded corners. macOS/Linux: transparent background.
- Windows: start in home dir when launched from the install dir; the NSIS
  installer handles PATH.

## Credits

- `assets/sounds/bell.flac`: "Notification Sound" by thehorriblejoke,
  <https://freesound.org/s/583554/>, CC0.
- `assets/fonts/`: Fira Code Nerd Font.

## Known gaps

- CI runs no tests; UI verification is manual.
- Block mode has only been exercised on Windows (PowerShell, cmd, Git Bash).
- cmd: no exit codes; a window resize during a long command can end its block
  early (ConPTY replays the prompt mark). cmd never reports cwd, so
  completion resolves against the starting directory.
- Under ConPTY, output printed before a program went fullscreen can be
  repainted once more below the kept frame.
- Completion is local only (no git branches, cmdlets, flags), ignores the
  cursor position, and has no candidate list UI.
- Raw takeover forwards only the wheel, not clicks/drags.
- Hover color transitions are instant (see `design-system.md`).
