use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::install::{self, Install, PendingInstaller, UpdatePlan};
use crate::core::update::{self, Offer};

const PENDING_FILE: &str = "pending-installer";

pub fn updates_dir() -> Result<PathBuf, String> {
  dirs::cache_dir()
    .map(|d| d.join("nova").join("updates"))
    .ok_or_else(|| "cannot determine cache directory".to_string())
}

pub fn version_dir(version: &str) -> Result<PathBuf, String> {
  Ok(updates_dir()?.join(version))
}

fn io(e: std::io::Error) -> String {
  e.to_string()
}

fn sibling(target: &Path, suffix: &str) -> PathBuf {
  let mut name = target.file_name().unwrap_or_default().to_os_string();
  name.push(suffix);
  target.with_file_name(name)
}

fn free_old_slot(target: &Path) -> PathBuf {
  let old = sibling(target, ".old");
  if !old.exists() || std::fs::remove_file(&old).is_ok() {
    return old;
  }
  let stamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_secs())
    .unwrap_or_default();
  sibling(target, &format!(".old.{stamp}"))
}

fn swap_file(new: &Path, target: &Path) -> Result<(), String> {
  let old = free_old_slot(target);
  std::fs::rename(target, &old).map_err(|e| format!("cannot move {}: {e}", target.display()))?;
  if let Err(e) = std::fs::rename(new, target) {
    let _ = std::fs::rename(&old, target);
    return Err(format!("cannot replace {}: {e}", target.display()));
  }
  Ok(())
}

fn extract_and_swap(
  archive: &mut zip::ZipArchive<std::fs::File>,
  name: &str,
  target: &Path,
) -> Result<(), String> {
  let mut entry = archive
    .by_name(name)
    .map_err(|_| format!("{name} not found in the portable ZIP"))?;
  let new = sibling(target, ".new");
  let mut out = std::fs::File::create(&new)
    .map_err(|e| format!("cannot write next to {}: {e}", target.display()))?;
  std::io::copy(&mut entry, &mut out).map_err(io)?;
  drop(out);
  if target.exists() {
    swap_file(&new, target)
  } else {
    std::fs::rename(&new, target).map_err(|e| format!("cannot write {}: {e}", target.display()))
  }
}

fn replace_exe_from_zip(zip_path: &Path, exe: &Path) -> Result<(), String> {
  let file = std::fs::File::open(zip_path).map_err(io)?;
  let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
  for name in install::WINDOWS_SIDECARS {
    if archive.index_for_name(name).is_some() {
      extract_and_swap(&mut archive, name, &exe.with_file_name(name))?;
    }
  }
  extract_and_swap(&mut archive, "nova.exe", exe)
}

fn replace_appimage(file: &Path, target: &Path) -> Result<(), String> {
  let new = sibling(target, ".new");
  std::fs::copy(file, &new)
    .map_err(|e| format!("cannot write next to {}: {e}", target.display()))?;
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&new, std::fs::Permissions::from_mode(0o755)).map_err(io)?;
  }
  swap_file(&new, target)
}

fn run(program: &str, args: &[&std::ffi::OsStr]) -> Result<(), String> {
  let status = Command::new(program)
    .args(args)
    .status()
    .map_err(|e| format!("{program}: {e}"))?;
  if status.success() {
    Ok(())
  } else {
    Err(format!("{program} exited with {status}"))
  }
}

fn bundle_old(bundle: &Path) -> PathBuf {
  let name = bundle.file_name().unwrap_or_default().to_string_lossy();
  bundle.with_file_name(format!(".{name}.old"))
}

fn replace_bundle(dmg: &Path, bundle: &Path) -> Result<(), String> {
  let mnt = dmg.with_extension("mnt");
  let _ = std::fs::create_dir_all(&mnt);
  run(
    "hdiutil",
    &[
      "attach".as_ref(),
      "-nobrowse".as_ref(),
      "-readonly".as_ref(),
      "-noautoopen".as_ref(),
      "-mountpoint".as_ref(),
      mnt.as_os_str(),
      dmg.as_os_str(),
    ],
  )?;
  let result = copy_bundle(&mnt, bundle);
  let _ = run(
    "hdiutil",
    &["detach".as_ref(), mnt.as_os_str(), "-force".as_ref()],
  );
  result
}

fn copy_bundle(mnt: &Path, bundle: &Path) -> Result<(), String> {
  let src = std::fs::read_dir(mnt)
    .map_err(io)?
    .filter_map(Result::ok)
    .map(|e| e.path())
    .find(|p| p.extension().is_some_and(|e| e == "app"))
    .ok_or_else(|| "no .app found in the disk image".to_string())?;
  let name = bundle.file_name().unwrap_or_default().to_string_lossy();
  let staged = bundle.with_file_name(format!(".{name}.new"));
  let _ = std::fs::remove_dir_all(&staged);
  run("ditto", &[src.as_os_str(), staged.as_os_str()])?;
  let old = bundle_old(bundle);
  let _ = std::fs::remove_dir_all(&old);
  std::fs::rename(bundle, &old).map_err(|e| format!("cannot move {}: {e}", bundle.display()))?;
  if let Err(e) = std::fs::rename(&staged, bundle) {
    let _ = std::fs::rename(&old, bundle);
    return Err(format!("cannot replace {}: {e}", bundle.display()));
  }
  Ok(())
}

fn pending_path() -> Result<PathBuf, String> {
  Ok(updates_dir()?.join(PENDING_FILE))
}

pub fn pending_for(offer: &Offer, setup: &Path) -> Option<PendingInstaller> {
  match &offer.install {
    Install::Nsis { exe, per_machine } => Some(PendingInstaller {
      version: offer.release.version.clone(),
      setup: setup.to_path_buf(),
      exe: exe.clone(),
      per_machine: *per_machine,
    }),
    _ => None,
  }
}

pub fn stage(offer: &Offer, file: &Path) -> Result<(), String> {
  match &offer.install {
    Install::Portable { exe } => replace_exe_from_zip(file, exe),
    Install::AppImage { path } => replace_appimage(file, path),
    Install::MacApp { bundle } => replace_bundle(file, bundle),
    Install::Nsis { .. } => {
      let pending = pending_for(offer, file).ok_or("not an installer install")?;
      std::fs::write(pending_path()?, pending.to_text()).map_err(io)
    }
    _ => Ok(()),
  }
}

fn spawn_detached(cmd: &mut Command) -> Result<(), String> {
  #[cfg(windows)]
  {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
  }
  if let Some(home) = dirs::home_dir() {
    cmd.current_dir(home);
  }
  cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

fn run_hidden_powershell(script: String) -> Result<(), String> {
  spawn_detached(
    Command::new("powershell")
      .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command"])
      .arg(script),
  )
}

fn run_installer(p: &PendingInstaller) -> Result<(), String> {
  run_hidden_powershell(install::nsis_helper_script(&p.setup, &p.exe, p.per_machine))
}

pub fn restart(offer: &Offer) -> Result<(), String> {
  match &offer.install {
    Install::Nsis { .. } => {
      let UpdatePlan::Stage { asset, .. } = &offer.plan else {
        return Err("nothing staged".into());
      };
      let setup = version_dir(&offer.release.version)?.join(&asset.name);
      let pending = pending_for(offer, &setup).ok_or("not an installer install")?;
      let _ = std::fs::remove_file(pending_path()?);
      run_installer(&pending)
    }
    Install::Scoop => {
      let exe = std::env::current_exe().map_err(io)?;
      let app_dir = install::scoop_app_dir(&exe).ok_or("cannot find the Scoop app folder")?;
      run_hidden_powershell(install::scoop_helper_script(std::process::id(), &app_dir))
    }
    Install::MacApp { bundle } => spawn_detached(Command::new("open").arg("-n").arg(bundle)),
    Install::AppImage { path } => spawn_detached(&mut Command::new(path)),
    Install::Portable { exe } => spawn_detached(&mut Command::new(exe)),
    _ => {
      let exe = std::env::current_exe().map_err(io)?;
      spawn_detached(&mut Command::new(exe))
    }
  }
}

fn remove_old_siblings(target: &Path) {
  let (Some(dir), Some(name)) = (target.parent(), target.file_name()) else {
    return;
  };
  let prefix = format!("{}.old", name.to_string_lossy());
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for e in entries.filter_map(Result::ok) {
    if e.file_name().to_string_lossy().starts_with(&prefix) {
      let _ = std::fs::remove_file(e.path());
    }
  }
}

fn remove_stale_downloads(current: &str) {
  let Ok(dir) = updates_dir() else {
    return;
  };
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for e in entries.filter_map(Result::ok) {
    let name = e.file_name().to_string_lossy().into_owned();
    if e.path().is_dir() && !update::is_newer(&name, current) {
      let _ = std::fs::remove_dir_all(e.path());
    }
  }
}

pub fn cleanup() {
  let probe = super::probe::light();
  let target = install::target_path(&probe);
  remove_old_siblings(&target);
  for name in install::WINDOWS_SIDECARS {
    remove_old_siblings(&target.with_file_name(name));
  }
  if let Some(bundle) = install::app_bundle(&target) {
    let _ = std::fs::remove_dir_all(bundle_old(&bundle));
  }
  remove_stale_downloads(update::current_version());
}

pub fn apply_pending_on_startup() -> bool {
  cleanup();
  let Ok(path) = pending_path() else {
    return false;
  };
  let Some(pending) = std::fs::read_to_string(&path)
    .ok()
    .and_then(|t| PendingInstaller::parse(&t))
  else {
    return false;
  };
  let _ = std::fs::remove_file(&path);
  if !update::is_newer(&pending.version, update::current_version()) || !pending.setup.is_file() {
    return false;
  }
  run_installer(&pending).is_ok()
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;

  fn write_zip(path: &Path, files: &[(&str, &[u8])]) {
    let mut zip = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    for (name, data) in files {
      zip
        .start_file(*name, zip::write::SimpleFileOptions::default())
        .unwrap();
      zip.write_all(data).unwrap();
    }
    zip.finish().unwrap();
  }

  fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nova-apply-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
  }

  #[test]
  fn portable_zip_swaps_exe_and_sidecars() {
    let dir = scratch("sidecars");
    let exe = dir.join("nova.exe");
    std::fs::write(&exe, b"old exe").unwrap();
    std::fs::write(dir.join("conpty.dll"), b"old dll").unwrap();
    let zip = dir.join("p.zip");
    write_zip(
      &zip,
      &[
        ("nova.exe", b"new exe"),
        ("conpty.dll", b"new dll"),
        ("OpenConsole.exe", b"new host"),
      ],
    );
    replace_exe_from_zip(&zip, &exe).unwrap();
    assert_eq!(std::fs::read(&exe).unwrap(), b"new exe");
    assert_eq!(std::fs::read(dir.join("conpty.dll")).unwrap(), b"new dll");
    assert_eq!(
      std::fs::read(dir.join("OpenConsole.exe")).unwrap(),
      b"new host"
    );
    assert_eq!(
      std::fs::read(dir.join("conpty.dll.old")).unwrap(),
      b"old dll"
    );
    remove_old_siblings(&dir.join("conpty.dll"));
    assert!(!dir.join("conpty.dll.old").exists());
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn portable_zip_without_sidecars_still_updates() {
    let dir = scratch("plain");
    let exe = dir.join("nova.exe");
    std::fs::write(&exe, b"old exe").unwrap();
    let zip = dir.join("p.zip");
    write_zip(&zip, &[("nova.exe", b"new exe")]);
    replace_exe_from_zip(&zip, &exe).unwrap();
    assert_eq!(std::fs::read(&exe).unwrap(), b"new exe");
    assert!(!dir.join("conpty.dll").exists());
    let _ = std::fs::remove_dir_all(&dir);
  }
}
