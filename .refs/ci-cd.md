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
- **`docs`** — builds `docs/` with `@docmd/core` (Bun) and deploys
  `docs/dist` to `gh-pages`.

## `release.yml` — tag `v*.*.*`

- **`package`** — matrix (windows, macos x86_64 + aarch64, ubuntu-22.04):
  `cargo build --release` then `cargo packager` → NSIS + portable ZIP, DMG,
  deb/AppImage, RPM. Uploads artifacts.
- **`release`** — collects artifacts, generates checksums, publishes the
  GitHub release.
- After `release`: **`update-readme`** (commits new download URLs to
  README), **`publish-homebrew`**, **`publish-winget`**, **`publish-scoop`**,
  **`publish-aur`**, **`publish-apt`**, **`publish-rpm`** — each pushes to
  its own external repo.

## Not set up yet

- No test job: unit tests (`cargo test`) exist but CI does not run them.
- No code signing / notarization for Windows or macOS artifacts.
