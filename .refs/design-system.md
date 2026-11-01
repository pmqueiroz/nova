# Design system (Nova v2)

Source of truth: Claude Design project `019df3c6-3f35-775a-a557-c65b0361a221`
(`Terminal v2.html`, `nova-ds.css`, `nova-session.jsx`, `nova-overlays.jsx`).
When this file and the handoff disagree, the handoff wins and this file gets
fixed. Exact sizes and colors live in code, not here.

## Binding constraints

- Every size is a multiple of 2. Spacing 2/4/8/12/16/24/32/48/64, radius
  4/8/12/16, text 12/14/16/18/24, control heights 24/32/40/48/56/64.
- Motion 120/200/400 ms. `ease` = cubic-bezier(.2,.8,.2,1), `spring` =
  cubic-bezier(.34,1.56,.64,1).
- Fira Code Nerd Font everywhere — a deliberate deviation from the handoff's
  IBM Plex. Sizes and weights still follow the handoff.
- Shortcuts show `⌘` on macOS, `Ctrl` elsewhere.
- Components read `ui/theme/tokens.rs` only — no raw numbers or hex in
  component code. Handoff oklch colors were converted to sRGB there.

## Code map

- `ui/theme/tokens.rs` — every DS value and color, `ACCENTS`, `with_alpha`/`mix`.
- `ui/theme/motion.rs` — curves as `Easing::Custom`, duration builders,
  `blink_on`, `dot_phase`.
- `ui/icons.rs` — the handoff's stroke icons as SVG, handles cached so each
  rasterizes once.
- `ui/components/ds/` — primitives, one file per handoff CSS class (button,
  kbd, seg, toggle, stepper, swatch, row, list_item, sheet, transform,
  focus_within).
- `ui/components/` — screens and app widgets (block, session, input_bar,
  palette, ai_sheet, settings, tab_bar, title_bar, …).

## iced 0.14 gaps and workarounds

| Design feature | Approach |
|---|---|
| CSS transforms (scale, translateY) | `ds/transform.rs`: wrapper using `Renderer::with_transformation`, layout untouched. `widget::float` can't be used: it only transforms when scale > 1 and draws as an overlay layer. Needs iced `advanced`. |
| Opacity fades | No global alpha: style fns take an alpha factor and scale their colors. |
| Box-shadow rings (swatch) | Drawn outside the widget bounds so layout never shifts. |
| Focus glow | `Shadow` with blur 4; inset 1 px lines become borders. |
| Backdrop blur | Not available; scrim color only. |
| `prefers-reduced-motion` | Not exposed; omitted. |
| Hover color transitions | Not animated: iced styles switch per status; animating needs a stateful wrapper per widget. |

## Deviations and gotchas

- **Sheets drop the `pop` scale (.96→1).** iced_wgpu passes the scale to
  glyphon, so every frame at a new scale re-rasterizes every glyph; settings
  dropped frames. The translateY is snapped to whole px for the same reason
  (fractional offsets miss the glyph cache's subpixel bins).
- Sheets are top-aligned at `min(16vh, 128)` and keep the same gap at the
  bottom; tall bodies scroll inside `sheet::max_height`. The panel is wrapped
  in `opaque` so clicks don't reach the scrim.
- ANSI colors are remapped to DS tokens (`sys/parser.rs::ansi_color`, per
  `NV_COLOR` in the handoff); normal and bright share a token. Background
  black maps to win-bot so dark fills stay dark. 256-color and truecolor pass
  through.
- `focus_within` runs a focusable operation over its subtree to detect focus
  — iced has no `:focus-within`.
- Settings' font list scan (font-kit) is prewarmed on a background thread at
  startup so the first open doesn't block a frame.
- Settings' background/foreground/border color editors were removed (the DS
  owns those) but the fields stay in the config for compatibility.
- About uses a raster lockup PNG (no SVG in the brand handoff).
- In block mode at the prompt, pasted text goes into the input bar with
  lines joined by `; `; otherwise to the PTY (bracketed when enabled).

## Compact layout (≤ 640 px, `tokens::block::is_compact`)

Matches the handoff's `@media (max-width:640px)`: tighter log and card
padding, "Ask AI" label hidden, title bar cwd hidden. `calc_block_grid` and
the pointer mapping use the same compact paddings so the PTY width matches
the card.

## Motion gaps

Every handoff keyframe is implemented except:

- Hover background/color transitions (buttons, tabs, seg, list items) — instant.
- Tab close × opacity — instant.
- Settings row fade on tab switch — not animated.
