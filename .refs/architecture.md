# Architecture

Nova is a terminal emulator: one Rust binary on `iced` 0.14 that drives one
PTY per pane, parses its output with `vte` into a `Grid`, and renders it. The
same binary is a CLI (`nova ask`, `nova config`, …) when given arguments.

## Layout

- `src/main.rs` — `cli::run_from_env()` runs first; if it returns an exit
  code, no window opens. Otherwise: detach console (Windows),
  `config::init()`, notifications, runtime theme, `ui::start()`.
- `src/cli/` — subcommands. Those that need the running app talk to it over
  private OSC 777 (see `docs/src/content/docs/internals-osc.md`).
- `src/core/` — platform-free logic: config, grid, block/session, byte
  router, completion, search, selection, url, glyph, ai.
- `src/sys/` — OS edges: `pty` (`PtyBridge`, shell bootstrap), `parser`
  (`AnsiExecutor`, the `vte::Perform` impl), bell, notification.
- `src/ui/` — the iced app. `app_state/` holds `Nova`, `Message`, `update/`
  (split by concern), `view.rs`, `subscription.rs`; `tab.rs` holds
  `Tab`/`SplitPane`; `components/` widgets; `theme/` tokens, motion, fonts.
- `assets/` — embedded fonts, icons, brand, sounds, default `settings.toml`
  (separate macOS variant). See [brand.md](./brand.md).
- `docs/` marketing site + user docs ([docs-site.md](./docs-site.md)) ·
  `packaging/` NSIS template, AUR PKGBUILD, ConPTY fetch.

## Data flow

1. A `PtyKey` subscription per pane spawns a PTY worker; bytes arrive as
   `Message::PtyOutputReceived(tab_id, bytes)`. `pty_worker` concatenates
   every chunk already queued (up to `MAX_OUTPUT_BATCH`) into one message, so
   an output flood costs one `update` + `view` per batch, not per read.
2. `core::router::route` feeds them through the pane's `vte::Parser` into
   `AnsiExecutor`, which mutates the target `Grid` and emits `ControlCommand`s
   (OSC 7 cwd, OSC 8 links, OSC 777).
3. `view` renders blocks (block mode) or the grid via `components::term`
   (classic / raw takeover).
4. Keys are encoded in `subscription.rs` (incl. kitty keyboard protocol) and
   sent as `PtyCommand::Input`.

**PTY spawn failure.** `initial-command` is spawned directly, not via the
shell, so a missing program fails `PtyBridge::new`. The worker sends
`spawn_error_message` as output and returns, so the pane gets `PtyExited`
instead of a dead window. portable-pty's `CreateProcessW` error text contains
NULs, which are stripped. The worker drops its own `tx_out` clone after a
successful spawn, otherwise `rx_out` never closes when the shell exits.

**Sideloaded ConPTY (Windows).** portable-pty loads `conpty.dll` from beside
the exe before kernel32's ConPTY; that dll runs the `OpenConsole.exe` next to
it. Nova ships both (`packaging/conpty/fetch.ps1`). The inbox conhost swallows
`?1000h/1002h/1003h/1006h`, so `Grid::mouse_mode` stays `None` and the wheel
falls back to arrow keys; OpenConsole forwards them. Both emit `?9001h`
(win32-input-mode), which Nova ignores. Bun apps receive SGR wheel reports;
Node (libuv) apps drop them.

## Terminal emulation (`sys/parser.rs`, `core/grid.rs`)

- `Cell.c` is a `SmolStr` so printing, erasing and `Cell::default()` never
  allocate. Use `c.as_str()`; `as_ref()` is ambiguous on it.
- **Screen storage is a ring of rows.** `Grid.cells` is private; logical row
  `r` lives at physical row `(r + origin) % rows`. A full-screen scroll just
  moves `origin` and clears one row; partial scroll regions (`CSI r`,
  `IL`/`DL`) swap rows. Access cells only via
  `row`/`row_mut`/`cell`/`cell_mut`/`erase_cols`/`erase_rows`. Alt-screen
  save/restore and resize call `linearize()` first, so saved buffers and
  reflow see `origin == 0`.
- Cell and pen colors are `Option<Rgb>`, not `Option<Color>`, to keep `Cell`
  small. Every color reaching a cell is 8-bit, so the round trip is exact
  (`cell_colors_round_trip_exactly`); use `fg_color()`/`bg_color()`. Alpha
  (dimming) is applied at render time.
- Once scrollback is at its limit, `scroll_up` reuses the evicted row's `Vec`.
- Tabs are fixed 8-column stops; no `HTS`/`TBC`. SGR 8/28 (hidden) are
  ignored — `CellAttrs` has no hidden attribute.
- **BCE:** erases and scrolled-in lines use `Grid::blank_cell()` with the
  current background. ConPTY paints full-width backgrounds as `SGR bg` + `EL`
  rather than spaces, so without BCE diff lines and TUI status bars vanish.
- Truecolor accepts `38;2;r;g;b` and the colon forms (`colon_rgb`).
- **SGR with an intermediate is ignored.** `CSI > 4;2 m` (modifyOtherKeys,
  sent by Claude Code) would otherwise read as underline + dim for all
  following text. Underline color `58;2;r;g;b` / `58;5;n` consumes its
  arguments (else `r`=4 underlines) and isn't rendered; `4:0` clears
  underline.
- `Grid::resize` on the alt screen also reflows the saved primary buffer,
  otherwise leaving the alt screen restores a buffer of the old size. Reflow
  never paints with the pen background and drops trailing blank cells per
  logical line, so padding from a wider width doesn't become empty rows.
- Block elements U+2580–259F are drawn as rectangles (`core::glyph`,
  `TermView`), not from the font: font glyphs don't fill the cell, and
  explicit cell backgrounds (Claude Code's logo paints `48;2;0;0;0`) show
  through the gaps as black bars. Box drawing still comes from the font.
  Block cards do the same in `output_lines.rs` (`without_block_glyphs` +
  `draw_block_glyphs` over the full `LINE_HEIGHT`). `block_glyph` runs per
  cell per frame, so `BlockGlyph` stores its rects inline and never
  allocates; `is_block_glyph` is the cheap range test.

## Block mode

Each command is shown as a block instead of a scrolling grid.

### Session and blocks (`core/block.rs`)

- `Session` (one per pane) has `blocks`, deduplicated `history`, `cwd`, PTY
  size and `phase`: `Prompt` (input bar owns keys), `Running(id)` (output
  goes to block `id`), `Raw(id)` (block is on the alt screen or took over;
  raw grid shown full-size, keys forwarded).
- `Block` owns an `output: Grid` sized like the PTY so in-place redraws work.
  Visible lines = scrollback + screen rows up to the last non-blank row or the
  cursor. Resolving that range scans the whole screen, so take
  `Block::lines()` once rather than `line(i)` in a loop. `view` calls
  `lines()` for every block, so a finished block's range is frozen
  (`freeze_lines`) in `Session::finish` and after `Session::resize`. The
  frozen range is used only while the block is `Done` and the grid's
  cols/rows/scrollback length still match, so a missed re-freeze falls back
  to a scan, never a stale range.
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
  latter) and maps `CommandNotFoundException` to 127, others to 1.
- **ConPTY ordering:** ConPTY passes OSC through immediately but renders text
  on its own schedule, so OSC marks can overtake their output. Under ConPTY
  the marks are private-use characters (`MARK_PROMPT`,
  `MARK_PROMPT_PREEXEC`, `MARK_COMMAND_START` in `core/grid.rs`) that
  `AnsiExecutor::print` swallows.
  - ConPTY also replays its screen buffer, so the readline wrapper clears the
    console before writing `command_start` (via `$Host.UI.WriteLine`;
    `[Console]::Write` turns it into `?`). A replayed `command_start` is a
    no-op; the block ends at the next `prompt_ready`.
  - Git Bash: `NOVA_TEXT_MARKS=1` makes the POSIX hook print text marks, and
    `PS0` clears the screen like the PowerShell wrapper.
  - cmd: `PROMPT=<U+F8F0>$P$G`. No exit codes or preexec — the mark ends the
    block with 0 and the echoed line is skipped.
- Shells that never send `prompt_ready` (fish, WSL where env doesn't cross
  `wsl.exe`, custom shells) fall back to classic mode.

### Byte router (`core/router.rs`)

`route()` must re-pick the target grid right after any byte that changes
routing, so one chunk carrying echo + `command_start` + output +
`command_complete` + prompt is split correctly — without feeding vte byte by
byte, which skips its bulk UTF-8 path:

- `segment_len` cuts the input right after each ConPTY text mark
  (`EF A3 B0..B2`) and, while `awaiting_echo`, after each `\n`. vte
  dispatches a plain-text run whole without checking for termination, so
  these must be split beforehand.
- Within a segment, `advance_until_terminated` stops after any escape
  sequence that queues a control or flips the alt screen / bracketed paste
  (`AnsiExecutor::terminated`); `route_step` then re-targets and continues.
- A mark split across reads completes in vte's partial-UTF-8 step, which is
  followed by a termination check, so it still lands on a boundary.
- `whole_and_bytewise_feeds_agree` pins that both feeds produce the same
  blocks.

Routing rules:

- Target grid: `Prompt`, a `Pending` block and non-block modes go to the
  pane's screen grid (keeps prompt and echo hidden); `Running`/`Raw` go to the
  block grid.
- **cwd (OSC 7)** also emits `ControlCommand::Cwd`, which the router applies
  to the screen grid's `pwd` whatever grid parsed it. Tab title, prompt and
  `Session.cwd` read the screen grid, and the OSC often lands while the block
  is still `Running` (ConPTY reorders it too); without this `cd` updates late
  or never.
- `Integration`: `Detecting` until the first `prompt_ready` → `Blocks`; after
  `INTEGRATION_TIMEOUT` without one (`expire`, driven by the 1 s `Tick`,
  subscribed only while some pane is `Detecting`) → `Classic`. A late
  `prompt_ready` still upgrades an idle classic pane.
- Shells without preexec: `on_submit` sets `awaiting_echo`; the first `\n`
  after submit starts the block.
- ConPTY positions its cursor absolutely, so a block grid starts at the
  screen cursor row; `Block::first_line` trims the blank lines above.
- Alt screen or bracketed paste inside a block → `Phase::Raw` (see
  [Raw takeover](#raw-takeover)).
- **Leaving the alt screen** keeps the last frame (`keep_last_frame`):
  `Grid::leave_alt_screen` snapshots visible rows, then `Grid::keep_frame`
  moves the primary screen's used rows and the frame into scrollback and
  clears the screen. It must not write the frame to the screen: right after
  `?1049l` ConPTY repaints its primary buffer and would wipe it.
- Responses (DA, …) and non-block controls come back in `Routed` and are
  pushed into the screen grid's queues, so existing handlers are unchanged.

### Pagers

- Pagers don't enable bracketed paste, so for them raw takeover means the
  alt screen. Git exports `LESS=FRX` when `LESS` is unset, and `X` skips the
  alt screen, so `nova_env` sets `LESS=FR` unless the user has `LESS`.
- PowerShell's `help` pipes to `more.com`, which blocks without an alt
  screen. The integration script redefines `global:help` to pipe
  `Get-Help @args` through `less` when available, else print directly.

### Input bar and submit (`update/session.rs`, `tab.rs`)

- Each pane owns an `InputBox`. `PaneMut`/`PaneRef` abstract over `Tab` and
  `SplitPane` so session code is written once.
- Up/Down history recall and Tab completion replace `InputBox.text` from
  `update`, which leaves iced's `text_input` cursor where it was, so both
  return `move_cursor_to_end` on the pane's input id.
- `clear`/`cls` clear blocks locally; anything else goes through
  `Session::submit` → `Integration::on_submit` → PTY.
- **Readline shells** (`router::is_readline_shell`) get a Ctrl+U prefix on
  every submit: readline can swallow a byte while starting up, dropping the
  first character of the first command in Git Bash.
- **PowerShell** (`router::is_powershell`) gets a leading space instead:
  under the sideloaded OpenConsole, PSReadLine drops the first character of
  the first line. A Shift-only key event doesn't absorb it; the space does.
- PSReadLine's ding is `Console.Beep`, not BEL, so Nova never sees it. The
  integration script sets `-BellStyle` from `general.bell`, read once at
  spawn.
- While a command runs, `INPUT_OWNS_KEYS` is off and keys go through the
  legacy encoder to the PTY (passwords, `Read-Host`, REPLs, Ctrl+C).
- The running state lives in the input bar, not the block: the prompt turns
  into `running_label` and the `text_input` is swapped for `stop_hint`. A
  disabled `text_input` would look typable while keys go to the PTY.
  `Nova::update` calls `sync_running_motion` after every message, so any
  phase change (submit, finish, respawn) flips the animation;
  `pane_animating` keeps frames on meanwhile.
- Because the `text_input` leaves the tree, `PtyOutputReceived` refocuses
  the input when `running_panes()` drops.
- History: `Session.history` is the source of truth, persisted to
  `command_history.bin` after every submit, loaded into every pane at startup
  and copied into new tabs/splits. Classic panes still use
  `Grid.command_history` for the inline ghost suggestion. Entries are unique
  (`push_command`, `set_command_history` merge), so `find_best_suggestion`
  — run on every echo while typing — is one allocation-free scan.

### Tab completion (`core/completion.rs`)

The shell never sees the input bar's text, so Nova completes locally.

- Always completes the **last word** — iced's `text_input` doesn't expose the
  cursor position.
- Command position: history first words + `PATH` executables. Otherwise:
  entries of the word's directory, resolved against `Session.cwd`; in
  readline shells on Windows `/c/...` maps to `C:\...`.
- `\` is a separator only on Windows outside readline shells (escape in
  bash). Matching is case-insensitive only on Windows.
- Several matches: insert the common prefix if longer, else cycle
  (`Completion`, `step(±1)`). Any edit, history walk or submit drops it.
- Runs synchronously in `update`, so a slow `PATH` entry stalls the UI.
  `PATH_CACHE` keeps listings per directory while its mtime is unchanged, so
  a Tab press costs one `stat` per `PATH` entry.

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

- Output cards report pointer moves through `components::cell_area`, not
  `mouse_area::on_move` (which publishes every pixel, and every message
  rebuilds `view`). It publishes `BlockPointer` only on entering a new
  `(line, col)` — computed with the same padding and cell size as
  `handle_block_pointer` — or while a button is held, so edge auto-scroll
  during a drag still advances.
- Selection is scoped to one block (`BlockSelection`). Ctrl+click opens the
  URL under the pointer. In the input bar, Ctrl+C copies when a block
  selection exists, otherwise clears the input.
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
- **A finished takeover keeps its fullscreen width** (`Block::keeps_width`):
  `Session::resize` and `needs_size` skip it, since reflowing an absolutely
  positioned frame to card width wraps its dividers and right-aligned text.
- The card clips the wider frame without a scrollbar. `Block::scroll_col`
  shifts drawing inside a clip layer and is added to the hit column in
  `handle_block_pointer`. It only changes while dragging a selection on the
  first/last visible column (`Block::scroll_to_col`).

**Grid rendering (`components::term`).** One `TermView` widget per grid, not
a widget per cell. `term()` resolves every cell's look (selection, search,
URL hover, cursor, reverse/dim) into a flat `DrawCell` list, dropping cells
with nothing to draw; `draw` paints background quads, block-glyph rects,
underline/strike rules and one cached `fill_text` per glyph at its cell
origin. Per-cell placement is deliberate: the font's own advance differs from
the cell width the pointer mapping assumes.

In `Phase::Raw`, `Nova::pane_body` swaps the log and input bar for
`components::term` showing the block's own grid; `PaneRef::input_grid` makes
kitty keyboard flags come from that grid. Wheel: a mouse report when the
program enabled mouse reporting, otherwise 3 arrow keys per notch.

**Pointer code must go through `Nova::pointer` / `Nova::selection_grid`**
(`update/mouse.rs`), never `tab.grid` directly. They pick the raw block grid
(`raw_pixel_to_cell`, scroll offset 0) when the active pane is raw, else the
classic grid (`pixel_to_cell`, `tab.scroll_offset`). In a raw pane,
clicks/drags become mouse reports when the program enabled mouse reporting;
otherwise (or with Shift) they drive the grid selection.

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
- Classic panes keep the legacy `calc_grid*` math; `pixel_to_cell` assumes
  the legacy layout below the title bar.
- Every resize (window, font, split divider release) goes through
  `resize_panes`. Sizing a split with classic math alone resized only the
  screen grid, so a raw takeover (Claude Code) got a PTY width different
  from its block grid and its redraws garbled.

### Initial window size

The window opens at `INITIAL_WINDOW_SIZE`, then `fit_window` resizes it
relative to the monitor in logical px (`fit_window_size`) and recenters, so
scaled screens don't overflow. On Windows the window starts hidden until the
fit is applied.

## App behavior

- **Global shortcuts** (palette, Ask AI, settings) are checked before any
  sheet captures keys: the same shortcut closes its sheet, a different one
  switches. The last tab with a live shell can't be closed
  (`is_last_live_tab`).
- **AI** (`update/ai.rs`): Generate (typed query), Diagnose (last failed
  block), Explain (last finished command); the block's output is the
  context. `split_answer` takes the first fenced code block's first line as
  the runnable command.
- **Inline failure hint** (`diagnostic_banner`): `command_failure;<code>`
  queues a diagnosis. The router has already finished the block, so blocks
  mode sends `session.last_failure()`; classic sends `extract_last_output`
  with no command. `core::ai::should_diagnose` skips locally for empty output
  or a no-match exit (code 1 from `grep`/`rg`/`diff`/… unless piped/chained).
  `diagnose_question` asks the model for `{"skip": true}` when it can only
  restate the error, else `{"message", "command"}`. Nothing renders while
  loading (`ai_diagnostic_inflight`); input clears it, so a late or skipped
  response leaves the banner empty. `parse_diagnosis` tolerates code fences,
  the legacy `command_to_solve` key, and non-JSON replies.
- **Drag a tab into a split** (`update/tabs.rs`, `split_drop_side`):
  `TabPressed` records the previously active tab as `drag_host_tab` by id,
  since reordering moves indices. Dragging below the title bar near a side
  edge sets `split_drop`; the view shows the host tab (`shown_tab_index`)
  with `split_drop_preview`. The host is the previously active tab, or the
  dragged tab's neighbour when that's the one being dragged; neither may
  already have a split. On release `merge_tab_into_split` turns the dragged
  `Tab` into a `SplitPane`; tab-only fields (`initial_command`,
  `wait_after_exit`) are dropped. Outside the title bar `tab_ghost` follows
  the cursor — a skeleton, not a render of real content — positioned from
  the window origin (`ghost_origin`) because it's stacked over the whole app.
  iced's `clip` is rectangular, so the ghost header's top corners are rounded
  to `R8 - border` or they leak past the rounded border.
- **PTY subscription identity:** iced keys `Subscription::run_with` by the
  data *and* the builder fn pointer, so tab and split panes must share the
  same `pty_stream` fn. With separate closures, moving a pane between tab
  and split restarts its shell.
- **Update check** (`core/update.rs` pure, `sys/update/` network): GitHub
  `releases/latest` via blocking `ureq` on `spawn_blocking`, compared with
  `semver` (drafts/prereleases ignored). Runs on `WindowOpened` and every
  `CHECK_INTERVAL` while `general.check-updates`. Failures show only in
  About. Install flow: [updates.md](./updates.md).
- **Shell detection** (`config::detect_shells`) spawns `where pwsh` and
  `wsl --list --quiet` on Windows, which can take seconds on a cold WSL.
  Startup uses `quick_shells()` (no subprocesses, same default shell);
  `WindowOpened` runs the full scan on `spawn_blocking` →
  `Message::ShellsDetected`, which replaces the list unless `general.shells`
  is configured. Memoized in `DETECTED_SHELLS`.
- **Audio bell** (`sys::bell::play`): one bell at a time (`PLAYING`); BELs
  during playback are dropped, so a BEL flood can't open a stream per BEL.
- **Git branch** (`git_branch`, walks up to `.git/HEAD`) is refreshed only
  when the cwd changes or a command completes, never per PTY message: it's
  filesystem I/O on the UI thread.
- `til_home` runs every frame, so the home dir is cached in `HOME_DIR`. The
  tab's hidden close × renders a `Space`, not a transparent icon — iced's svg
  tint drops alpha, so a transparent icon draws black.

## Shared state

- Persistence never blocks `update`: `config::save_in_background` and
  history saves serialize on the UI thread and hand bytes to
  `sys::persist::write_in_background`, one writer thread that keeps only the
  newest bytes per path (settings fields save per keystroke).
- Config: `OnceLock<Config>` (`config::get()`) + mutex-guarded keybindings,
  saved to `dirs::config_dir()/nova/settings.toml`. **An unparsable file is
  overwritten with defaults on startup**, so removed sections (e.g. v1's
  `[status-bar]`) must stay parseable — serde ignores unknown sections
  (`legacy_status_bar_section_is_ignored`).
- Runtime theme: mutex-guarded `RuntimeTheme` in `ui::theme::color`.
- Overlay atomics in `app_state::nova` (`SETTINGS_OPEN`, `INPUT_OWNS_KEYS`,
  `ACTIVE_KITTY_FLAGS`, …): the keyboard subscription can't borrow `Nova`.
- **Pointer moves:** every message rebuilds `view`, so the `listen_with` fn
  records the position in `POINTER` and emits `CursorMoved` only if
  `TRACK_POINTER` is set (`needs_pointer_moves`) or the pointer crossed a
  window-resize edge zone (`POINTER_ZONE`). `update` copies
  `latest_pointer()` into `cursor_position` before any other message.
  Anything new that reads `cursor_position` live in `view` must be added to
  one of those two conditions.
- Animation: `window::frames()` is subscribed only while
  `Nova::is_animating()`, so an idle window renders nothing. The
  `CursorBlinkTick` runs only while `needs_cursor_blink()` (blink on and a
  grid cursor shown, or the cursor currently hidden so it ticks back).
  Animations interpolate against `Nova.now`.

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
