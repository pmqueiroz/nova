# Updates

Nova finds a newer GitHub release and installs it in whatever way matches how
it was installed. Package managers own their installs, so Nova runs **their**
upgrade command. Every other install has its binary swapped by Nova itself.

## Code map

- `core/update.rs`: release parsing (`newer_release`, incl. `assets`), semver
  compare (`is_newer`), `Offer { release, install, plan }`, `Status`.
- `core/install.rs`: pure. `detect(&Probe) -> Install`,
  `plan(&Install, &Release, Arch) -> UpdatePlan`, `asset_name`,
  `parse_checksums`, `nsis_helper_script`, `PendingInstaller`.
- `sys/update/`: `probe` (fills `Probe` from the OS), `download`
  (`fetch_verified`: SHA-256 against `checksums.txt`), `apply` (stage, restart,
  startup cleanup), `mod` (`check`, `prepare` → `Event` channel).
- UI: `app_state/update/updates.rs` (flow), `components/update.rs` (title-bar
  action, About line/button, restart confirm), `components/confirm.rs` (the
  confirm sheet, shared with the paste warning).
- CLI: `nova update [--install]`.

## Install kinds

| `Install` | Detected by | `UpdatePlan` |
|---|---|---|
| `Nsis { exe, per_machine }` | Windows, `uninstall.exe` next to the exe; per-machine when under a `%ProgramFiles%` dir | Stage `nova_V_x64-setup.exe` |
| `Portable { exe }` | Windows, no uninstaller | Stage `nova_V_x64_portable.zip` |
| `Scoop` | path contains `/scoop/apps/nova/` | Delegate `powershell -Command scoop update nova` |
| `MacApp { bundle }` | macOS, exe inside a `.app` | Stage `nova_V_{x64,aarch64}.dmg` |
| `BrewCask` | macOS `.app` + `{/opt/homebrew,/usr/local}/Caskroom/nova` | Delegate `brew upgrade --cask nova` |
| `BrewFormula` | Linux, path contains `/Cellar/nova/` | Delegate `brew upgrade nova` |
| `Aur { helper }` | `pacman -Qo` owns it | Delegate `paru`/`yay -S nova-bin`; no helper → AUR page |
| `Deb` | `dpkg -S` owns it | Download `.deb`, delegate `sudo apt install <file>` |
| `Rpm { dnf }` | `rpm -qf` owns it | Download `.rpm`, delegate `sudo dnf install <file>` / `sudo rpm -U <file>` |
| `AppImage { path }` | `$APPIMAGE` set, not owned by a manager | Stage `nova_V_x86_64.AppImage` |
| `Cargo` | exe in `$CARGO_HOME/bin` | Delegate `cargo install --git … --tag vV --locked` |
| `Unmanaged` | `target/{debug,release}` builds, unknown layouts | Open the release page |

- Winget installs with the same NSIS setup, so it is `Nsis`. Running the newer
  setup updates the ARP `DisplayVersion`, which is what winget reads.
- On Linux the path checked is `$APPIMAGE` when set, not `current_exe()` (that
  is inside the AppImage's FUSE mount). Brew and AUR both ship the AppImage, so
  they are recognised through it. Package-ownership queries run only for paths
  under `/usr/`.
- A missing asset, a missing `checksums.txt` or an unsupported arch (only
  x86_64 is shipped for Windows and Linux) falls back to `OpenPage`.
- Deb/rpm install the downloaded file instead of `apt upgrade`, so it works the
  same for repo and manual installs.

## Flow

1. `Status::Available(offer)` → `title_update_button` shows an orange-tint
   "Update" after the settings icon. The label is always "Update"; the press
   follows `update_action` (start, none while downloading, restart confirm
   when `Ready`). The About tab keeps the detailed labels ("Update to X",
   "Get X" for `OpenPage`, "Updating N%", "Restart to update").
2. `StartUpdate`:
   - `OpenPage` opens the URL;
   - `Delegate` without a download opens an **update tab**;
   - otherwise → `Downloading`, with `sys::update::prepare` streaming
     `Event::Progress` / `Event::Done` through `Task::run`.
3. `Done(Ok(file))`:
   - `Stage` → `Ready` ("Restart to update");
   - `Delegate` → update tab with `{file}` filled in → `Delegated`.
4. `ConfirmRestart` → confirm sheet → `apply::restart` → `process::exit(0)`.

The update tab is `Tab::for_command`: the argv runs directly (no shell, so
paths with spaces survive), classic mode, and it always waits after exit. Tab
launch commands are argv (`Tab.initial_command: Option<Vec<String>>`); the
config's `initial-command` string is split on whitespace by
`config::initial_argv`.

## Staging (`apply::stage`)

Downloads go to `dirs::cache_dir()/nova/updates/<version>/`. A file already
there with the right hash is reused. Writes go to `<name>.part`, which is
renamed after the hash matches.

- **Portable:** extract `nova.exe` to `nova.exe.new`, rename the running exe to
  `nova.exe.old` (Windows allows renaming, not overwriting, a running exe),
  then move the new one in. If `.old` is locked by a still-running older
  process, a timestamped `.old.<secs>` is used. This swap happens at stage
  time, so the next launch is already the new version. The
  `install::WINDOWS_SIDECARS` (`conpty.dll`, `OpenConsole.exe`) get the same
  swap first when the ZIP has them (a loaded dll and a running host can be
  renamed too); older ZIPs without them still update. `cleanup()` removes
  their `.old*` siblings as well.
- **AppImage:** copy to `<path>.new`, `chmod 755`, same rename swap. The
  running mount keeps the old inode.
- **MacApp:** `hdiutil attach -nobrowse -readonly` the dmg, `ditto` the `.app`
  to `.Nova.app.new` beside the bundle, detach, then rename
  `Nova.app → .Nova.app.old` and `.Nova.app.new → Nova.app`. No quarantine
  xattr gets added because the download isn't done by a browser.
- **NSIS:** only writes `updates/pending-installer` (`PendingInstaller` text:
  version, setup, exe, per_machine). The setup can't run while Nova is open.

## Restart and startup

- `apply::restart`:
  - NSIS: removes the marker and spawns hidden PowerShell running
    `nsis_helper_script`: `Start-Process setup -ArgumentList '/S','/CURRENTUSER'|'/ALLUSERS' [-Verb RunAs] -Wait; Start-Process nova.exe`;
  - macOS: `open -n` the bundle;
  - the rest spawn the (now replaced) binary.

  Children are spawned with `CREATE_NO_WINDOW` on Windows.
- The silent NSIS setup kills **every** running `nova.exe`
  (`CheckIfAppIsRunning`), and doesn't relaunch: the finish page is skipped in
  silent mode, which is why the helper starts Nova itself.
- `main` calls `apply::apply_pending_on_startup()` before any window:
  - `cleanup()` removes `<exe>.old*` siblings, `.Nova.app.old` and cache
    version dirs not newer than the running version;
  - then, if a marker for a newer version exists, it removes the marker
    (so a failing installer can't loop), spawns the helper and exits. That is
    how an NSIS update the user didn't restart for applies on the next launch.

## Untested

Only the Windows portable path has been exercised end to end (0.24.0 →
1.0.0 via `nova update --install`, then startup cleanup). NSIS, macOS and
Linux swaps and all delegate commands are covered by unit tests of detection
and planning only.
