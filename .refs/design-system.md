# Design system (Nova v2)

Source of truth: Claude Design project `019df3c6-3f35-775a-a557-c65b0361a221`,
files `Terminal v2.html`, `nova-ds.css`, `nova-session.jsx`, `nova-overlays.jsx`.
The UI must match that handoff. When this file and the handoff disagree, the
handoff wins and this file gets fixed.

## Binding constraints

- Every size is a multiple of 2. Spacing 2/4/8/12/16/24/32/48/64, radius
  4/8/12/16, text 12/14/16/18/24, control heights 24/32/40/48/56/64.
- Motion durations 120/200/400 ms. Curves: `ease` = cubic-bezier(.2,.8,.2,1),
  `spring` = cubic-bezier(.34,1.56,.64,1).
- Fonts: Fira Code Nerd Font for everything (deliberate deviation from the
  handoff's IBM Plex Mono/Sans). Sizes and weights still follow the handoff.
- Shortcuts are shown as `⌘` on macOS and `Ctrl` elsewhere.

## iced 0.14 capabilities and fallbacks

| Design feature | iced 0.14 | Approach |
|---|---|---|
| Keyframe/transition timing | `iced::Animation<T>` (lilt), `Easing::Custom(fn(f32) -> f32)` | Curves implemented as cubic-bezier solvers in `theme/motion.rs`, passed as `Easing::Custom`. |
| Frame ticks | `iced::window::frames()` | Subscribed only while any animation `is_animating`. |
| Scale (press .96, pop .96, pulse .8, hover 1.08/1.16) | `widget::float` only transforms when scale > 1 or translated, and then draws as an overlay layer | Custom wrapper widget using `Renderer::with_transformation` (scale about the center, plus translate). Layout is unaffected, as with CSS transforms. |
| translateY (`in`, `pop`, `dot`) | same wrapper | `Transformation::translate`. |
| Opacity fades | no global alpha | Style fns take an alpha factor and scale the colors they emit. |
| Window gradient | `Background::Gradient(Linear)` | 180° from win-top to win-bot. |
| Shadows, focus glow | `Shadow { color, offset, blur_radius }` on container/button styles | The 4 px accent glow becomes a shadow with blur 4. Inset 1 px lines become a border. |
| Backdrop blur (scrim) | none | Scrim color only (`#04060990`). |
| `prefers-reduced-motion` | not exposed | Omitted. |

## Tokens

Colors converted from the handoff's oklch values to sRGB:

| Token | Value |
|---|---|
| win-top / win-bot | `#2A3139` / `#15181F` |
| surface-1/2/3 | white α .04 / .08 / .12 |
| line / line-2 | white α .08 / .14 |
| fg-1 / fg-2 / fg-3 | `#F0F2F5` / `#CDD1D8` / `#9399A2` |
| green / red / yellow | `#88E48C` / `#FD736D` / `#F2CD64` |
| blue / magenta / cyan | `#82BAFF` / `#D695E4` / `#72DEE4` |
| on-accent | `#11161F` |
| accents | Blue `#82BAFF`, Mint `#65DFAF`, Amber `#FCBE62`, Rose `#FC899D` |
| traffic lights | close `#F75E54`, min `#FAC131`, max `#61CC69` |
| scrim | `#04060990` |

ANSI colors are remapped onto these tokens in `sys/parser.rs::ansi_color`
(normal and bright share a token: black→fg-3, red, green, yellow, blue,
magenta, cyan, white→fg-1), per `NV_COLOR` in the handoff. Background black
maps to win-bot so dark fills stay dark. Unstyled output is fg-2 (block
renderer default). 256-color and truecolor values pass through.

## Code map

- `src/ui/theme/tokens.rs` — every DS value: `space`, `radius`, `text`,
  `height`, `duration`, `font_size` (14–20, step 2, default 16) modules and
  the color constants above, plus `ACCENTS` presets and `with_alpha`/`mix`.
  Components read tokens only; no raw numbers or hex in component code.
- `src/ui/theme/motion.rs` — `ease`/`spring` cubic-bezier curves wrapped as
  `EASE`/`SPRING` (`Easing::Custom`), `fast`/`base`/`slow` animation builders
  (120/200/400 ms), `blink_on` (1 s step blink) and `dot_phase` (the
  `nv-dot` keyframe: returns opacity and Y offset for a given delay).
- `src/ui/icons.rs` — the handoff's 14 stroke icons (`Icon`), rendered as
  24×24 stroke-2 round-cap SVG and tinted through `svg::Style::color`.
  Handles are cached per icon so iced rasterizes each one once.
- `src/ui/components/ds/` — DS primitives, one file per handoff CSS class.
  `transform.rs` is the scale/translate wrapper (`Transform`, `press`):
  it scales about the content's center through `Renderer::with_transformation`
  without touching layout, and `.press()` eases to `PRESS_SCALE` (.96) while
  the left button is held, matching `:active{transform:scale(.96)}`. Needs the
  iced `advanced` feature.
  `.hover(scale)` adds a hover scale and `.spring()` switches both to the
  spring curve (traffic lights: hover 1.16, press .84).
  `button.rs`: `btn(Variant)` builder (default/ghost/primary, h40, r8,
  padding 0/16, gap 8, 14 px semibold; primary bold on accent with
  on-accent text, hover brightness 1.08) and `icon_button(icon, IconSize, on_press)` (transparent,
  hover surface-2, fg-2 → fg-1; `IconSize::Md` = 40 box/20 icon,
  `IconSize::Sm` = 32 box/16 icon — title bar, search bar, stepper, closes). Both are wrapped in `press`. CSS hover
  color transitions (120 ms) are not animated yet; tracked for the motion audit.
  `kbd.rs`: `kbd`/`keys` chips (h24, r4, surface-2, 12 px) and
  `binding_labels`, which renders a parsed keybinding with ⌘/⌃/⌥/⇧ on macOS
  and Ctrl/Alt/Shift elsewhere.
  `seg.rs`: segmented control (surface-1 r12 track, padding 4, h32 r8
  buttons, selected surface-3/fg-1, idle fg-3). `track_style` is shared with
  the stepper.
  `toggle.rs`: 48×28 switch, custom widget. The track eases surface-3 → accent
  (200 ms) and the 24 px knob springs 20 px (200 ms `spring`), fg-1 → on-accent,
  with a 0/2/4 black .24 shadow; press scales .96.
  `stepper.rs`: −/+ icon buttons (32) around a 48-wide value in a seg track.
  `swatch.rs`: 32 px color dot, custom widget. Selected rings (2 px win-bot,
  then 2 px fg-1) are drawn outside its bounds like CSS box-shadow, so layout
  never shifts. Hover springs to 1.08, press .96, 2 px black .16 inner edge.
  `row.rs`: settings row (min-h 64, padding 12/0, gap 24, 16 px label fg-1,
  14 px hint fg-3) with a 1 px line divider unless it is the last row.
  `list_item.rs`: sheet list item (h48, padding 0/16, gap 16, r8, 20 px icon).
  Selected = surface-2 + fg-1 + accent icon; `Tone::Error` paints icon and
  hint red. Hovering publishes `on_hover` so the keyboard selection follows the
  mouse, as in `useNvList`.
  `sheet.rs`: modal sheet (`.nv-scrim` + `.nv-sheet`). Scrim fills the window
  and closes on press; the panel (576 wide, 640 for settings, r16, gradient,
  1 px line-2 edge, 0/32/64 black .48 shadow) is top-aligned and centered
  horizontally only: it sits `min(16vh, 128)` from the top and keeps the same
  gap at the bottom (`max_height` = window − 2 × `top_offset`; tall bodies
  scroll inside that cap instead of running to the window edge). It is
  wrapped in `opaque` so clicks inside never reach the scrim.
  `progress` (0→1, driven by the caller's spring animation) maps to the `pop`
  keyframe: translateY 8→0 (`pop_offset`, snapped to whole px), scrim/shadow
  alpha 0→1. **Deviation:** the handoff's scale .96→1 is dropped for sheets.
  iced_wgpu passes the transformation scale to glyphon, so every frame at a
  new scale re-rasterizes every glyph in the sheet (text-heavy sheets like
  settings dropped frames); fractional translation likewise misses the glyph
  cache's subpixel bins, hence the rounding. `head`
  (h64, padding 0/16/0/24) and `foot` (min-h 56, padding 8/16/8/24) add the
  1 px line separators.
- `src/ui/components/block.rs` — one command block (`.nv-entry`). Command
  line in fg-1 (program name semibold, rest regular, wraps anywhere). Output
  card: surface-1, r8, padding 16/24, output font (`tokens::block::font_size`
  = size − 2, line height 1.5), default text fg-2, or red when the exit code is
  non-zero. Lines are rendered as one `rich_text` per grid row, with runs of
  identically styled cells merged into a single span (reverse swaps fg/bg,
  dim halves alpha, trailing blanks trimmed). Pending or not-yet-printing
  blocks show three 6 px accent dots (`motion::dot_phase`, delays
  0/120/240 ms) + "Running" (14 px fg-3). Optional 48 px `HH:MM` timestamp
  column (12 px fg-3, gap 16). The entry eases in over 200 ms (translateY 8→0,
  alpha 0→1); the output follows 60 ms later. `is_animating` keeps frames
  coming while dots are visible or an entry is still easing in.
- `src/ui/components/session.rs` — the block log (`.nv-scroll`/`.nv-log`).
  Scrollable column of block entries, gap 8, padding 16/32/24 (compact
  8/16/16). Empty session: centered "Type *help* to see what Nova can do"
  (16 px fg-3, `help` italic fg-1), fading in over 400 ms. Pinning: each
  pane records `InputBox.scrolled_up` from `on_scroll` (more than 32 px from
  the bottom = scrolled up); new PTY output snaps the log to the end only
  while pinned (`Nova::follow_output`). Clicking the log focuses the pane's
  input (`FocusInput`). Widget ids per pane: `scroll_id`, `input_id`.
- `src/ui/components/input_bar.rs` — Warp-style input (`.nv-inputbar`): h56,
  margin 0/16/16, padding 0/8/0/16, r12, surface-2 + 1 px line; focused (via
  `ds/focus_within.rs`, which runs a focusable operation over its subtree)
  white α .10, accent α .48 border and accent α .12 glow. Prompt = short cwd
  fg-3 + semibold accent `$`; transparent `text_input`; ghost "Ask AI" button
  (sparkle + label + ⌘K/Ctrl K kbd, label hidden when compact); enter icon
  button that springs .8→1 on every submit (`InputBox.pulsed_at`). While block
  mode is at the prompt `INPUT_OWNS_KEYS` is set: ↑/↓ walk history, Ctrl+C
  clears the input, Ctrl+L clears blocks; nothing is forwarded to the PTY.
  Block mode is on (`tab::BLOCKS_ENABLED`).
- `src/ui/components/command_palette.rs` — palette sheet (`NvPalette`). Head:
  20 px search icon + 18 px "Search commands" input. Items in design order
  (New tab ⌘T, Close tab ⌘W, Clear screen ⌃L, Ask AI ⌘K, Larger text ⌘+,
  Smaller text ⌘−, Toggle timestamps, Settings ⌘,) plus Split pane and Find;
  Ctrl replaces ⌘ off macOS. Case-insensitive substring filter; no match shows
  "Nothing matches “q”". ↑/↓ clamp (no wrap), ↩ runs, esc / scrim click
  close, hovering selects. Pressing the palette shortcut again closes it.
  The sheet pops with `Nova::sheet_progress` (200 ms spring from
  `sheet_opened_at`).
- `src/ui/components/ai_sheet.rs` — AI sheet (`NvAI`). List view: accent
  sparkle + 18 px "Describe what you want to do"; items `Suggest a command
  for “q”` (only with a query), `Fix “cmd”` with a red `exit N` hint (only
  when the active pane's last block failed) and `Explain last command`.
  Answer view: back button, title (Suggested command / Diagnosis / Explain)
  and sparkle; "Thinking" dots while the request runs; the reply types in at
  2 chars per 16 ms with a blinking accent caret; backtick spans render as
  code chips (surface-2); errors are red. When done, a suggestion card
  (surface-1, r12, line edge) shows `$ cmd` + primary Run. Footer hints ↩
  Select/Run, esc Close/Back. Run closes the sheet and submits the command in
  the active session 120 ms later (`Message::RunCommand`).
- Inline AI suggestion (`ai_sheet::inline_suggestion`): the automatic
  diagnosis for a failed command (when `[ai] diagnostic-banner` is on) is no
  longer a bottom banner. In block mode it renders as a card directly under the
  failed block (surface-1, r8, padding 16/24): accent sparkle + "Nova AI"
  label, the message (14 px, backtick code chips) and, when the model returned
  a fix, the `.nv-suggest` row with Run, which submits it in the pane. Classic
  panes show the same card above the bottom edge.
- `src/ui/components/paste_warning.rs` — paste confirmation sheet for
  multi-line or risky clipboard text (`is_dangerous_paste`). Head: red alert
  icon + "Paste multiple lines?" / "Paste this command?"; body: reason (14 px
  fg-3) and a surface-1 r8 preview (first 8 lines, "+N more lines"); foot:
  Cancel (ghost) and Paste (primary). esc cancels, ↩ pastes. In block mode at
  the prompt, pasted text goes into the input bar with lines joined by `; `;
  otherwise it is written to the PTY (bracketed paste when enabled).
- `src/ui/components/shell_picker.rs` — "New tab with shell" list sheet
  (terminal icon per shell, `default` hint on the first). Opens from
  right-click on the title bar `+` or the palette item "New tab with shell…"
  (only when more than one shell is available; otherwise it opens the
  default directly). ↑/↓ clamp, ↩ opens, esc / scrim close.
- `src/ui/components/settings.rs` — settings sheet (`NvSettings`, 640 wide).
  Head: "Settings" + seg [Appearance, Shell, Keys, AI, About]. Body: DS rows
  in a scrollable capped to `sheet::max_height` minus head/foot. Foot: config path in mono (click opens
  the read-only raw view) + primary Done; esc / scrim close.
  - Appearance: Text size stepper (14–20, step 2, `step_font`), Timestamps
    toggle (`theme.timestamps`), Accent swatches (Blue/Mint/Amber/Rose, which
    set accent and cursor), Font pick list, Cursor seg, Cursor blink. The
    pick list's system font scan (`font::installed_fonts`, font-kit) is
    prewarmed on a background thread at startup (`prewarm_installed_fonts`)
    so the first open doesn't block a frame on it.
  - Shell: Editor seg nano/vim/code, Bell seg Off/Flash/Sound, Window controls
    seg, shell list with remove buttons + add field, Initial command, Keep
    finished tabs.
  - Keys: one row per binding with kbd chips; click to re-record ("Press
    keys…" in accent), click again or esc to cancel.
  - AI: provider seg, model, API key (secure), base URL, auto-diagnose toggle.
  - About: brand lockup (`assets/brand/lockup/nova-lockup-for-dark-256h.png`,
    raster `image`, 240 wide), version, "Reset to defaults".
  Text fields use `field_style` (surface-1, r8, line edge, accent α .48 when
  focused). The background/foreground/border color editors are gone (the DS
  owns those); the fields stay in the config file for compatibility.

## Motion audit (handoff keyframes → implementation)

| Handoff | Where | Status |
|---|---|---|
| `pop` 400 ms ease (window) | `components/app.rs`, `Nova.window_pop` | done |
| `pop` 200 ms spring (sheets) | `ds/sheet.rs`, `Nova::sheet_progress` | done (opacity → scrim/shadow alpha; scale dropped for perf, see `sheet.rs`) |
| `pop` 200 ms ease (new tab) | `tab_bar.rs`, `Tab.appear` | done |
| `in` 200 ms (entry, output +60 ms) | `components/block.rs` | done (opacity via color alpha) |
| `fade` 400 ms (empty state) | `components/session.rs` | done |
| `fade` 200 ms (scrim) | `ds/sheet.rs` | done |
| `fade` 200 ms (settings rows on tab switch) | `components/settings.rs` | not animated (gap) |
| `dot` 1 s loop, 0/120/240 ms | `block::dots`, `motion::dot_phase` | done |
| `blink` 1 s steps (AI caret) | `ai_sheet.rs`, `motion::blink_on` | done |
| `pulse` 200 ms spring (enter icon) | `input_bar.rs`, `InputBox.pulsed_at` | done |
| `:active` scale .96 | buttons, icon buttons, list items, tabs, seg, swatch, toggle (`ds/transform.rs` `press`) | done |
| lights hover 1.16 / active .84 spring | `title_bar::light` | done |
| swatch hover 1.08 spring | `ds/swatch.rs` | done |
| toggle knob 200 ms spring, track 200 ms ease | `ds/toggle.rs` | done |
| hover background/color transitions (120–200 ms) | buttons, tabs, seg, list items | instant (gap): iced styles switch per status; animating them needs a stateful wrapper per widget |
| tab close × opacity 120 ms | `tab_bar.rs` | instant (gap) |
| input bar focus ring 200 ms | `input_bar.rs` / `ds/focus_within.rs` (`blend`) | done |

## Compact layout (≤ 640 px window, `tokens::block::is_compact`)

Matches the handoff's `@media (max-width:640px)`: the block log padding drops
to 8/16/16, output cards to 12/16, the input bar hides the "Ask AI" label
(icon only) and the title bar hides the cwd. `calc_block_grid` and the
pointer-to-cell mapping use the same compact paddings, so the PTY width
always matches the visible card.
