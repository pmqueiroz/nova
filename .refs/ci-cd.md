# CI/CD

GitHub Actions, three workflows in `.github/workflows/`. Toolchain is
`dtolnay/rust-toolchain@stable` everywhere (no pin file); Rust cache via
`Swatinem/rust-cache` keyed per target.

## `ci.yml` — push to `main`, PRs into `main`

- **`fmt`** — `cargo fmt --check` on ubuntu.
- **`clippy`** — `cargo clippy -- -D warnings` on a matrix: windows
  (x86_64-msvc), macos-latest (aarch64), ubuntu-22.04 (x86_64-gnu; installs
  X11/Wayland/GL/ALSA dev libs). Warnings fail the build.

Locally, `lefthook.yml` runs `cargo clippy --fix` and `cargo fmt` on staged
`.rs` files pre-commit.

## `cd.yml` — push to `main`

- **`release-please`** — maintains the release PR (`release-type: rust`,
  untagged component) using `RELEASE_PLEASE_TOKEN`. Merging it bumps
  `Cargo.toml`, `CHANGELOG.md`, `.release-please-manifest.json` and pushes a
  `vX.Y.Z` tag. Changelog is driven by Conventional Commit types.
- **`docs`** — `bun install --frozen-lockfile && bun run build` in `docs/`
  (Astro, see [docs-site.md](./docs-site.md)) and deploys `docs/dist` to
  `gh-pages`.

## `release.yml` — tag `v*.*.*`

- **`package`** — matrix (windows, macos x86_64 + aarch64, ubuntu-22.04):
  `cargo build --release` then `cargo packager` → NSIS + portable ZIP, DMG,
  deb/AppImage, RPM. Uploads artifacts.
  - Windows first runs `packaging/conpty/fetch.ps1 -Dest target/release`:
    downloads the pinned `Microsoft.Windows.Console.ConPTY` nupkg, checks its
    SHA-256, and copies `conpty.dll` + `OpenConsole.exe` (x64) next to
    `nova.exe`. To bump, change `$Version` and `$Sha256` together.
  - The NSIS template derives their source paths from `MAINBINARYSRCPATH`
    via `!searchreplace` (cargo-packager `resources` would also land in the
    macOS/Linux bundles), renames any existing copies to `.old` before
    writing (a lingering `OpenConsole.exe` may still run), and deletes them
    on uninstall. A missing file fails `makensis`.
  - The portable ZIP holds all three at its root; Scoop extracts it as is.
- **`release`** — collects artifacts, generates checksums, publishes the
  GitHub release.
- After `release`: **`update-readme`** (commits new download URLs to
  README), **`publish-homebrew`**, **`publish-winget`**, **`publish-scoop`**,
  **`publish-aur`**, **`publish-apt`**, **`publish-rpm`** — each pushes to
  its own external repo.
- **`publish-scoop` writes the whole manifest inline.** Its `shortcuts`
  entry is the only Start Menu entry Scoop installs get. Scoop points the
  shortcut at `apps\nova\current\`, so a taskbar pin made from it survives
  `scoop update`. A pin made from a running versioned exe does not.

**Asset names are a contract.** `core::install::asset_name` builds the
filenames the updater downloads (`nova_V_x64-setup.exe`,
`nova_V_x64_portable.zip`, `nova_V_{x64,aarch64}.dmg`,
`nova_V_x86_64.AppImage`, `nova_V_amd64.deb`, `nova-V-1.x86_64.rpm`), and it
verifies them against `checksums.txt`. The portable ZIP must keep `nova.exe` at
its root. Renaming an artifact here breaks in-app updates; the
`asset_names_match_release_files` test pins the list. See
[updates.md](./updates.md).

## Not set up yet

- No test job: unit tests (`cargo test`) exist but CI does not run them.
- No code signing / notarization for Windows or macOS artifacts.

## Release profile

`[profile.release]` uses fat LTO and `codegen-units = 1`: slower release
builds for a faster binary (cross-crate inlining into iced/vte/wgpu). No
`panic = "abort"` — a panic in a PTY reader thread must only end that thread.
