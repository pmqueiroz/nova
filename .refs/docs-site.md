# Docs site (`docs/`)

Static Astro site served at `nova.pmqueiroz.dev`: landing page at `/`, docs
under `/docs/`. Replaced docmd because docmd could not render a custom landing
page or the design's docs chrome.

## Design source

Claude Design project `019df3c6-3f35-775a-a557-c65b0361a221`:
`Nova Site.html` (landing), `Nova Docs.html` (docs template), `nova-site.css`,
`nova-site.js`, plus the brief in `uploads/Nova - Website Design Brief.md`.
They are ported 1:1, with comments stripped and inline styles moved to classes:

| Design file | Port |
| --- | --- |
| `nova-site.css` | `src/styles/site.css` (tokens, header, buttons, cmdline, code, callouts, tables, search sheet) |
| `<style>` in `Nova Site.html` | `src/styles/landing.css` |
| `<style>` in `Nova Docs.html` | `src/styles/docs.css` (+ `.no-toc` grid when a page has no TOC) |
| `nova-site.js` | `src/scripts/site.js` (theme, OS switch, copy, TOML/shell highlight, search, drawer, TOC scrollspy) |
| hero typing script | `src/scripts/typed.js` |

Deliberate deviation: winget is shown as available (the design still says
"pending approval").

## Layout

- `src/layouts/Base.astro` — head (meta, OG, favicon, GA `G-X9WTDL262N`,
  pre-paint theme from `?theme=` / localStorage), loads `site.js`. Props:
  `title`, `description`, optional `type` (`website` | `article`) and
  `jsonLd` (see SEO below).
- `src/components/` — `Header` (`variant="landing" | "docs"`), `Footer`,
  `Logo`, `Icon` (all SVGs live in `src/lib/icons.ts`), `Keys`, `CopyButton`,
  `Cmdline`, `OsSeg`, `SearchModal`, `DocsSidebar`, `Toc`, `Pager`.
- `src/pages/index.astro` — landing. `src/pages/docs/[...slug].astro` — one
  page per Markdown file. `index.md` → `/docs/`.
- `src/lib/release.ts` — current version from `.release-please-manifest.json`.
- `src/lib/nav.ts` — **single source** for sidebar groups, page order, titles,
  crumbs, prev/next and the search index. Add a new page here or it is built
  but not linked.
- `src/content/docs/*.md` — docs content (`title`, `description` frontmatter;
  description becomes the lead paragraph).

## SEO

- Every page: canonical, `robots`, sitemap link, `theme-color` (dark/light
  `--bg` as hex), Open Graph + Twitter tags with `og.png` (1280×640, a copy
  of `.github/assets/nova-og-1280x640.png`; keep the width/height meta in
  sync if it changes), and a JSON-LD array that always starts with `WebSite`.
- `src/lib/seo.ts` builds the extra JSON-LD: landing passes
  `SoftwareApplication` (version from `release.ts`); docs pages pass
  `TechArticle` + `BreadcrumbList` (Nova › Docs › page) with `type="article"`.
- The frontmatter `description` is the meta/OG description, so keep it a real
  sentence.
- `public/robots.txt` points to `sitemap-index.xml` (from `@astrojs/sitemap`;
  the redirect stubs are not in it). `public/apple-touch-icon.png` is
  `assets/icons/macos/nova-macos-1024.png` cropped to the squircle, corners
  filled with its gradient, resized to 180×180.

## Markdown pipeline

Astro 7 uses Sätteri by default; remark/rehype plugins need
`@astrojs/markdown-remark` installed. `syntaxHighlight` is off: code is
highlighted client-side by `site.js` exactly like the design.
Plugins in `src/lib/markdown.mjs`:

- fenced code → `.code` block with header (lang, or `title="…"` from the meta)
  and copy button; `toml`, `sh`/`bash`/`zsh`/`powershell` get token colors.
- `:::note`, `:::tip`, `:::warning[Title]` → `.callout`.
- OS tabs (code inside renders as `.cmdline`):

  ~~~md
  ::::os-tabs
  :::mac
  ```sh
  brew install --cask pmqueiroz/tap/nova
  ```
  :::
  :::linux
  …
  :::
  ::::
  ~~~

- `:kbd[Mod+K]` → key caps; `Mod` shows ⌘ on macOS and Ctrl elsewhere.
- Unknown `:name` text directives are turned back into plain text, so prose
  like `a:b` survives remark-directive.
- Tables are wrapped in `.table-wrap`; h2/h3 get slug ids and a `#` anchor.
  Astro's heading text then starts with `#`, which the page strips for the TOC
  and search.
- Link docs pages as `/docs/<slug>/`. Old root URLs (`/installation`, …) are
  kept as Astro `redirects` in `astro.config.mjs`.

## Gotchas

- `compressHTML: false` is required: the terminal mocks use `white-space:
  pre-wrap` and would lose their newlines and indentation otherwise.
- "Updated with release X" reads `../.release-please-manifest.json` at build
  time, so the build runs from `docs/` with the repo checked out.
- Copy as Markdown copies the raw file body (`entry.body`).
