use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::core::install::{self, Arch, Os, PkgOwner, Probe};

pub fn os() -> Os {
  if cfg!(target_os = "windows") {
    Os::Windows
  } else if cfg!(target_os = "macos") {
    Os::Mac
  } else if cfg!(target_os = "linux") {
    Os::Linux
  } else {
    Os::Other
  }
}

pub fn arch() -> Arch {
  match std::env::consts::ARCH {
    "x86_64" => Arch::X86_64,
    "aarch64" => Arch::Aarch64,
    _ => Arch::Other,
  }
}

pub fn light() -> Probe {
  let exe = std::env::current_exe()
    .map(|p| p.canonicalize().unwrap_or(p))
    .unwrap_or_default();
  Probe {
    os: os(),
    arch: arch(),
    exe,
    appimage: std::env::var_os("APPIMAGE").map(PathBuf::from),
    ..Probe::default()
  }
}

pub fn full() -> Probe {
  let mut p = light();
  let target = install::target_path(&p);
  p.has_uninstaller = target
    .parent()
    .is_some_and(|d| d.join("uninstall.exe").is_file());
  p.program_files = ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"]
    .iter()
    .filter_map(std::env::var_os)
    .map(PathBuf::from)
    .collect();
  p.cask_installed = p.os == Os::Mac
    && ["/opt/homebrew/Caskroom/nova", "/usr/local/Caskroom/nova"]
      .iter()
      .any(|d| Path::new(d).is_dir());
  p.cargo_bin = std::env::var_os("CARGO_HOME")
    .map(PathBuf::from)
    .or_else(|| dirs::home_dir().map(|h| h.join(".cargo")))
    .map(|h| h.join("bin"));
  if p.os == Os::Linux && install::norm(&target).starts_with("/usr/") {
    p.pkg_owner = owner(&target);
    if p.pkg_owner == Some(PkgOwner::Pacman) {
      p.aur_helper = ["paru", "yay"]
        .iter()
        .find(|h| on_path(h))
        .map(|h| h.to_string());
    }
    p.has_dnf = on_path("dnf");
  }
  p
}

fn succeeds(program: &str, args: &[&std::ffi::OsStr]) -> bool {
  Command::new(program)
    .args(args)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .is_ok_and(|s| s.success())
}

fn owner(target: &Path) -> Option<PkgOwner> {
  let t = target.as_os_str();
  if succeeds("dpkg", &["-S".as_ref(), t]) {
    Some(PkgOwner::Dpkg)
  } else if succeeds("pacman", &["-Qo".as_ref(), t]) {
    Some(PkgOwner::Pacman)
  } else if succeeds("rpm", &["-qf".as_ref(), t]) {
    Some(PkgOwner::Rpm)
  } else {
    None
  }
}

fn on_path(name: &str) -> bool {
  std::env::var_os("PATH")
    .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
}
