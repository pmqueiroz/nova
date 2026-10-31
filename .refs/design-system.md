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

ANSI colors in block output are remapped onto these tokens (red→red,
brightBlack→fg-3, white→fg-1, default→fg-2), per `NV_COLOR` in the handoff.

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
  `button.rs`: `btn(Variant)` builder (default/ghost/primary, h40, r8,
  padding 0/16, gap 8, 14 px semibold; primary bold on accent with
  on-accent text, hover brightness 1.08) and `icon_button` (transparent,
  hover surface-2, fg-2 → fg-1). Both are wrapped in `press`. CSS hover
  color transitions (120 ms) are not animated yet; tracked for the motion audit.
  `kbd.rs`: `kbd`/`keys` chips (h24, r4, surface-2, 12 px) and
  `binding_labels`, which renders a parsed keybinding with ⌘/⌃/⌥/⇧ on macOS
  and Ctrl/Alt/Shift elsewhere.
