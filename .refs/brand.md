# Brand assets (v2)

Source: the brand-v2 handoff export (`exports/brand-v2/`).

Colors: accent `#82baff` · accent on light `#3a81d7` · ink `#11161f` ·
white `#f0f2f5`.

## Layout

| Path | Contents | Used by |
| --- | --- | --- |
| `assets/icons/windows/nova.ico` | 16–256 multi-size ico | `build.rs` (winres exe icon) |
| `assets/icons/windows/nova-windows-*.png` | 16–256 PNGs | `src/ui/mod.rs` window icon (256) |
| `assets/icons/macos/nova.icns` | 16–1024 incl. @2x | cargo-packager `icons` (`.app`/dmg) |
| `assets/icons/macos/nova-macos-*.png` | 16–1024 PNGs | `src/ui/mod.rs` window icon (512) |
| `assets/icons/linux/nova-linux-*.png` | 16–512 PNGs + `nova-linux.svg` | packager `icons` (deb/AppImage), window icon (256) |
| `assets/icons/mark/` | mark SVG/PNG (+ black/white/blue-on-light), `nova-glyph*` | not referenced in code |
| `assets/brand/lockup/` | mark + wordmark PNGs; `for-dark` = light text | Settings → About (`for-dark-256h`) |
| `assets/brand/wordmark/` | "Nova" text only (IBM Plex Sans 600) | not referenced in code |
| `.github/assets/nova-hero-1280x400.gif` | README hero (caret blink) | `README.md` |
| `.github/assets/nova-og-*.png` | social preview | uploaded manually in GitHub repo settings |
| `docs/public/favicon.{ico,svg}`, `docs/public/og.png` | site favicon (brand-v2 `icon/web/favicon.svg`, metadata stripped) and OG image (copy of `nova-og-1280x640.png`) | `docs/src/layouts/Base.astro` |

## Gotchas

- Sizes ≤ 48 px use the simplified 4-ray glyph for legibility; the handoff
  already baked that into the small PNGs and the `.ico`/`.icns`.
- The site logo is the inline 4-ray glyph SVG + "nova" text (`Logo.astro`),
  colored by the theme accent, so it needs no per-theme raster.
- No lockup SVG exists in the handoff, so About uses a raster PNG.
- The in-app default accent (`DEFAULT_ACCENT` in `src/core/config.rs`, `tokens::BLUE`) is the same `#82BAFF`.
