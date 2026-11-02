use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::update::{Asset, Release};

pub const REPO_URL: &str = "https://github.com/pmqueiroz/nova";
pub const AUR_URL: &str = "https://aur.archlinux.org/packages/nova-bin";
pub const CHECKSUMS_ASSET: &str = "checksums.txt";
pub const FILE_ARG: &str = "{file}";
pub const WINDOWS_SIDECARS: &[&str] = &["conpty.dll", "OpenConsole.exe"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Os {
  Windows,
  Mac,
  Linux,
  #[default]
  Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Arch {
  X86_64,
  Aarch64,
  #[default]
  Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PkgOwner {
  Dpkg,
  Rpm,
  Pacman,
}

#[derive(Debug, Clone, Default)]
pub struct Probe {
  pub os: Os,
  pub arch: Arch,
  pub exe: PathBuf,
  pub appimage: Option<PathBuf>,
  pub has_uninstaller: bool,
  pub program_files: Vec<PathBuf>,
  pub cask_installed: bool,
  pub pkg_owner: Option<PkgOwner>,
  pub aur_helper: Option<String>,
  pub has_dnf: bool,
  pub cargo_bin: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Install {
  Nsis { exe: PathBuf, per_machine: bool },
  Portable { exe: PathBuf },
  Scoop,
  MacApp { bundle: PathBuf },
  BrewCask,
  BrewFormula,
  AppImage { path: PathBuf },
  Aur { helper: Option<String> },
  Deb,
  Rpm { dnf: bool },
  Cargo,
  Unmanaged,
}

impl std::fmt::Display for Install {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      Install::Nsis { .. } => "the Windows installer",
      Install::Portable { .. } => "the portable ZIP",
      Install::Scoop => "Scoop",
      Install::MacApp { .. } => "the DMG",
      Install::BrewCask | Install::BrewFormula => "Homebrew",
      Install::AppImage { .. } => "the AppImage",
      Install::Aur { .. } => "the AUR",
      Install::Deb => "a .deb package",
      Install::Rpm { .. } => "an .rpm package",
      Install::Cargo => "cargo",
      Install::Unmanaged => "an unmanaged build",
    };
    f.write_str(s)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdatePlan {
  Stage {
    asset: Asset,
    checksums: Asset,
  },
  Delegate {
    argv: Vec<String>,
    download: Option<(Asset, Asset)>,
  },
  OpenPage(String),
}

impl UpdatePlan {
  pub fn command_line(&self) -> Option<String> {
    match self {
      UpdatePlan::Delegate { argv, .. } => Some(argv.join(" ")),
      _ => None,
    }
  }
}

pub fn norm(p: &Path) -> String {
  let s = p.to_string_lossy().replace('\\', "/");
  let s = s.strip_prefix("//?/").unwrap_or(&s).to_string();
  s.to_lowercase()
}

pub fn target_path(probe: &Probe) -> PathBuf {
  match (&probe.os, &probe.appimage) {
    (Os::Linux, Some(p)) => p.clone(),
    _ => probe.exe.clone(),
  }
}

fn is_dev_build(path: &str) -> bool {
  path.contains("/target/debug/") || path.contains("/target/release/")
}

fn in_cargo_bin(probe: &Probe, target: &Path) -> bool {
  probe
    .cargo_bin
    .as_ref()
    .is_some_and(|bin| target.parent().is_some_and(|p| norm(p) == norm(bin)))
}

pub fn app_bundle(exe: &Path) -> Option<PathBuf> {
  exe
    .ancestors()
    .find(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("app")))
    .map(Path::to_path_buf)
}

pub fn detect(probe: &Probe) -> Install {
  let target = target_path(probe);
  let path = norm(&target);
  if is_dev_build(&path) {
    return Install::Unmanaged;
  }
  match probe.os {
    Os::Windows => {
      if path.contains("/scoop/apps/nova/") {
        Install::Scoop
      } else if in_cargo_bin(probe, &target) {
        Install::Cargo
      } else if probe.has_uninstaller {
        let per_machine = probe
          .program_files
          .iter()
          .any(|pf| path.starts_with(&format!("{}/", norm(pf).trim_end_matches('/'))));
        Install::Nsis {
          exe: target,
          per_machine,
        }
      } else {
        Install::Portable { exe: target }
      }
    }
    Os::Mac => match app_bundle(&target) {
      Some(_) if probe.cask_installed => Install::BrewCask,
      Some(bundle) => Install::MacApp { bundle },
      None if in_cargo_bin(probe, &target) => Install::Cargo,
      None => Install::Unmanaged,
    },
    Os::Linux => {
      if path.contains("/cellar/nova/") {
        Install::BrewFormula
      } else if probe.pkg_owner == Some(PkgOwner::Pacman) {
        Install::Aur {
          helper: probe.aur_helper.clone(),
        }
      } else if probe.pkg_owner == Some(PkgOwner::Dpkg) {
        Install::Deb
      } else if probe.pkg_owner == Some(PkgOwner::Rpm) {
        Install::Rpm { dnf: probe.has_dnf }
      } else if probe.appimage.is_some() {
        Install::AppImage { path: target }
      } else if in_cargo_bin(probe, &target) {
        Install::Cargo
      } else {
        Install::Unmanaged
      }
    }
    _ => Install::Unmanaged,
  }
}

pub fn asset_name(install: &Install, version: &str, arch: Arch) -> Option<String> {
  match (install, arch) {
    (Install::Nsis { .. }, Arch::X86_64) => Some(format!("nova_{version}_x64-setup.exe")),
    (Install::Portable { .. }, Arch::X86_64) => Some(format!("nova_{version}_x64_portable.zip")),
    (Install::MacApp { .. }, Arch::X86_64) => Some(format!("nova_{version}_x64.dmg")),
    (Install::MacApp { .. }, Arch::Aarch64) => Some(format!("nova_{version}_aarch64.dmg")),
    (Install::AppImage { .. }, Arch::X86_64) => Some(format!("nova_{version}_x86_64.AppImage")),
    (Install::Deb, Arch::X86_64) => Some(format!("nova_{version}_amd64.deb")),
    (Install::Rpm { .. }, Arch::X86_64) => Some(format!("nova-{version}-1.x86_64.rpm")),
    _ => None,
  }
}

fn find_asset(release: &Release, name: &str) -> Option<Asset> {
  release.assets.iter().find(|a| a.name == name).cloned()
}

fn verified_asset(install: &Install, release: &Release, arch: Arch) -> Option<(Asset, Asset)> {
  let name = asset_name(install, &release.version, arch)?;
  Some((
    find_asset(release, &name)?,
    find_asset(release, CHECKSUMS_ASSET)?,
  ))
}

fn argv(parts: &[&str]) -> Vec<String> {
  parts.iter().map(|s| s.to_string()).collect()
}

pub fn plan(install: &Install, release: &Release, arch: Arch) -> UpdatePlan {
  let page = || UpdatePlan::OpenPage(release.url.clone());
  let delegate = |argv: Vec<String>| UpdatePlan::Delegate {
    argv,
    download: None,
  };
  match install {
    Install::Nsis { .. }
    | Install::Portable { .. }
    | Install::MacApp { .. }
    | Install::AppImage { .. } => match verified_asset(install, release, arch) {
      Some((asset, checksums)) => UpdatePlan::Stage { asset, checksums },
      None => page(),
    },
    Install::Deb | Install::Rpm { .. } => {
      let Some(download) = verified_asset(install, release, arch) else {
        return page();
      };
      let argv = match install {
        Install::Rpm { dnf: false } => argv(&["sudo", "rpm", "-U", FILE_ARG]),
        Install::Rpm { dnf: true } => argv(&["sudo", "dnf", "install", FILE_ARG]),
        _ => argv(&["sudo", "apt", "install", FILE_ARG]),
      };
      UpdatePlan::Delegate {
        argv,
        download: Some(download),
      }
    }
    Install::Scoop => delegate(argv(&[
      "powershell",
      "-NoProfile",
      "-Command",
      "scoop update nova",
    ])),
    Install::BrewCask => delegate(argv(&["brew", "upgrade", "--cask", "nova"])),
    Install::BrewFormula => delegate(argv(&["brew", "upgrade", "nova"])),
    Install::Aur { helper: Some(h) } => delegate(vec![h.clone(), "-S".into(), "nova-bin".into()]),
    Install::Aur { helper: None } => UpdatePlan::OpenPage(AUR_URL.into()),
    Install::Cargo => delegate(vec![
      "cargo".into(),
      "install".into(),
      "--git".into(),
      REPO_URL.into(),
      "--tag".into(),
      format!("v{}", release.version),
      "--locked".into(),
    ]),
    Install::Unmanaged => page(),
  }
}

pub fn fill_file(argv: &[String], file: &Path) -> Vec<String> {
  argv
    .iter()
    .map(|a| {
      if a == FILE_ARG {
        file.to_string_lossy().into_owned()
      } else {
        a.clone()
      }
    })
    .collect()
}

pub fn parse_checksums(text: &str) -> HashMap<String, String> {
  text
    .lines()
    .filter_map(|l| {
      let mut parts = l.split_whitespace();
      let hash = parts.next()?;
      let name = parts.next()?.trim_start_matches('*');
      Some((name.to_string(), hash.to_ascii_lowercase()))
    })
    .collect()
}

fn ps_quote(p: &Path) -> String {
  format!("'{}'", p.to_string_lossy().replace('\'', "''"))
}

pub fn nsis_helper_script(setup: &Path, exe: &Path, per_machine: bool) -> String {
  let (mode, elevate) = if per_machine {
    ("/ALLUSERS", " -Verb RunAs")
  } else {
    ("/CURRENTUSER", "")
  };
  format!(
    "Start-Process -FilePath {} -ArgumentList '/S','{mode}'{elevate} -Wait; Start-Process -FilePath {}",
    ps_quote(setup),
    ps_quote(exe)
  )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingInstaller {
  pub version: String,
  pub setup: PathBuf,
  pub exe: PathBuf,
  pub per_machine: bool,
}

impl PendingInstaller {
  pub fn to_text(&self) -> String {
    format!(
      "{}\n{}\n{}\n{}\n",
      self.version,
      self.setup.display(),
      self.exe.display(),
      self.per_machine
    )
  }

  pub fn parse(text: &str) -> Option<Self> {
    let mut lines = text.lines();
    Some(Self {
      version: lines.next()?.to_string(),
      setup: PathBuf::from(lines.next()?),
      exe: PathBuf::from(lines.next()?),
      per_machine: lines.next()?.parse().ok()?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn probe(os: Os, exe: &str) -> Probe {
    Probe {
      os,
      arch: Arch::X86_64,
      exe: PathBuf::from(exe),
      ..Probe::default()
    }
  }

  fn release(names: &[&str]) -> Release {
    Release {
      version: "1.2.0".into(),
      url: "https://example.com/v1.2.0".into(),
      assets: names
        .iter()
        .map(|n| Asset {
          name: n.to_string(),
          url: format!("https://example.com/{n}"),
        })
        .collect(),
    }
  }

  fn all_assets() -> Release {
    release(&[
      "checksums.txt",
      "nova_1.2.0_x64-setup.exe",
      "nova_1.2.0_x64_portable.zip",
      "nova_1.2.0_x64.dmg",
      "nova_1.2.0_aarch64.dmg",
      "nova_1.2.0_x86_64.AppImage",
      "nova_1.2.0_amd64.deb",
      "nova-1.2.0-1.x86_64.rpm",
    ])
  }

  #[test]
  fn windows_nsis_per_user_and_per_machine() {
    let mut p = probe(Os::Windows, r"C:\Users\me\AppData\Local\Nova\nova.exe");
    p.has_uninstaller = true;
    p.program_files = vec![PathBuf::from(r"C:\Program Files")];
    assert!(matches!(
      detect(&p),
      Install::Nsis {
        per_machine: false,
        ..
      }
    ));
    p.exe = PathBuf::from(r"C:\Program Files\Nova\nova.exe");
    assert!(matches!(
      detect(&p),
      Install::Nsis {
        per_machine: true,
        ..
      }
    ));
  }

  #[test]
  fn windows_portable_scoop_and_cargo() {
    let p = probe(Os::Windows, r"D:\tools\nova\nova.exe");
    assert!(matches!(detect(&p), Install::Portable { .. }));
    let p = probe(Os::Windows, r"C:\Users\me\scoop\apps\nova\current\nova.exe");
    assert_eq!(detect(&p), Install::Scoop);
    let mut p = probe(Os::Windows, r"C:\Users\me\.cargo\bin\nova.exe");
    p.cargo_bin = Some(PathBuf::from(r"C:\Users\me\.cargo\bin"));
    assert_eq!(detect(&p), Install::Cargo);
  }

  #[test]
  fn verbatim_prefix_is_ignored() {
    let p = probe(
      Os::Windows,
      r"\\?\C:\Users\me\scoop\apps\nova\1.0.0\nova.exe",
    );
    assert_eq!(detect(&p), Install::Scoop);
  }

  #[test]
  fn dev_builds_are_unmanaged() {
    let p = probe(Os::Windows, r"C:\src\nova\target\release\nova.exe");
    assert_eq!(detect(&p), Install::Unmanaged);
  }

  #[test]
  fn macos_dmg_and_cask() {
    let mut p = probe(Os::Mac, "/Applications/Nova.app/Contents/MacOS/nova");
    assert_eq!(
      detect(&p),
      Install::MacApp {
        bundle: PathBuf::from("/Applications/Nova.app")
      }
    );
    p.cask_installed = true;
    assert_eq!(detect(&p), Install::BrewCask);
  }

  #[test]
  fn linux_kinds() {
    let mut p = probe(Os::Linux, "/tmp/.mount_novaXY/usr/bin/nova");
    p.appimage = Some(PathBuf::from("/home/me/Apps/nova.AppImage"));
    assert_eq!(
      detect(&p),
      Install::AppImage {
        path: PathBuf::from("/home/me/Apps/nova.AppImage")
      }
    );

    p.appimage = Some(PathBuf::from(
      "/home/linuxbrew/.linuxbrew/Cellar/nova/1.0.0/bin/nova",
    ));
    assert_eq!(detect(&p), Install::BrewFormula);

    p.appimage = Some(PathBuf::from("/usr/bin/nova"));
    p.pkg_owner = Some(PkgOwner::Pacman);
    p.aur_helper = Some("paru".into());
    assert_eq!(
      detect(&p),
      Install::Aur {
        helper: Some("paru".into())
      }
    );

    let mut p = probe(Os::Linux, "/usr/bin/nova");
    p.pkg_owner = Some(PkgOwner::Dpkg);
    assert_eq!(detect(&p), Install::Deb);
    p.pkg_owner = Some(PkgOwner::Rpm);
    p.has_dnf = true;
    assert_eq!(detect(&p), Install::Rpm { dnf: true });
    p.pkg_owner = None;
    assert_eq!(detect(&p), Install::Unmanaged);
  }

  #[test]
  fn asset_names_match_release_files() {
    let r = all_assets();
    let cases = [
      (
        Install::Nsis {
          exe: PathBuf::new(),
          per_machine: false,
        },
        Arch::X86_64,
      ),
      (
        Install::Portable {
          exe: PathBuf::new(),
        },
        Arch::X86_64,
      ),
      (
        Install::MacApp {
          bundle: PathBuf::new(),
        },
        Arch::X86_64,
      ),
      (
        Install::MacApp {
          bundle: PathBuf::new(),
        },
        Arch::Aarch64,
      ),
      (
        Install::AppImage {
          path: PathBuf::new(),
        },
        Arch::X86_64,
      ),
      (Install::Deb, Arch::X86_64),
      (Install::Rpm { dnf: true }, Arch::X86_64),
    ];
    for (install, arch) in cases {
      let name = asset_name(&install, "1.2.0", arch).unwrap();
      assert!(r.assets.iter().any(|a| a.name == name), "{name}");
    }
  }

  #[test]
  fn unsupported_arch_opens_page() {
    let install = Install::Portable {
      exe: PathBuf::new(),
    };
    assert_eq!(
      plan(&install, &all_assets(), Arch::Aarch64),
      UpdatePlan::OpenPage("https://example.com/v1.2.0".into())
    );
  }

  #[test]
  fn missing_checksums_opens_page() {
    let install = Install::Portable {
      exe: PathBuf::new(),
    };
    let r = release(&["nova_1.2.0_x64_portable.zip"]);
    assert!(matches!(
      plan(&install, &r, Arch::X86_64),
      UpdatePlan::OpenPage(_)
    ));
  }

  #[test]
  fn self_managed_installs_stage() {
    let install = Install::Portable {
      exe: PathBuf::new(),
    };
    match plan(&install, &all_assets(), Arch::X86_64) {
      UpdatePlan::Stage { asset, checksums } => {
        assert_eq!(asset.name, "nova_1.2.0_x64_portable.zip");
        assert_eq!(checksums.name, CHECKSUMS_ASSET);
      }
      other => panic!("{other:?}"),
    }
  }

  #[test]
  fn package_managers_delegate() {
    let r = all_assets();
    let line = |i: Install| plan(&i, &r, Arch::X86_64).command_line();
    assert_eq!(
      line(Install::Scoop).as_deref(),
      Some("powershell -NoProfile -Command scoop update nova")
    );
    assert_eq!(
      line(Install::BrewCask).as_deref(),
      Some("brew upgrade --cask nova")
    );
    assert_eq!(
      line(Install::BrewFormula).as_deref(),
      Some("brew upgrade nova")
    );
    assert_eq!(
      line(Install::Aur {
        helper: Some("yay".into())
      })
      .as_deref(),
      Some("yay -S nova-bin")
    );
    assert_eq!(
      line(Install::Cargo).as_deref(),
      Some("cargo install --git https://github.com/pmqueiroz/nova --tag v1.2.0 --locked")
    );
    assert_eq!(
      plan(&Install::Aur { helper: None }, &r, Arch::X86_64),
      UpdatePlan::OpenPage(AUR_URL.into())
    );
  }

  #[test]
  fn deb_and_rpm_download_then_delegate() {
    let r = all_assets();
    match plan(&Install::Deb, &r, Arch::X86_64) {
      UpdatePlan::Delegate {
        argv,
        download: Some((asset, _)),
      } => {
        assert_eq!(asset.name, "nova_1.2.0_amd64.deb");
        let filled = fill_file(&argv, Path::new("/cache/nova.deb"));
        assert_eq!(filled.join(" "), "sudo apt install /cache/nova.deb");
      }
      other => panic!("{other:?}"),
    }
    assert_eq!(
      plan(&Install::Rpm { dnf: false }, &r, Arch::X86_64)
        .command_line()
        .as_deref(),
      Some("sudo rpm -U {file}")
    );
  }

  #[test]
  fn checksums_parse_sha256sum_output() {
    let m = parse_checksums("ABC123  nova_1.2.0_amd64.deb\ndef456 *checksums.txt\n\n");
    assert_eq!(m["nova_1.2.0_amd64.deb"], "abc123");
    assert_eq!(m["checksums.txt"], "def456");
  }

  #[test]
  fn nsis_helper_quotes_and_elevates() {
    let s = nsis_helper_script(
      Path::new(r"C:\Users\O'Neil\setup.exe"),
      Path::new(r"C:\Program Files\Nova\nova.exe"),
      true,
    );
    assert!(s.contains(r"'C:\Users\O''Neil\setup.exe'"));
    assert!(s.contains("'/S','/ALLUSERS' -Verb RunAs -Wait"));
    assert!(s.ends_with(r"Start-Process -FilePath 'C:\Program Files\Nova\nova.exe'"));
    let user = nsis_helper_script(Path::new("s.exe"), Path::new("n.exe"), false);
    assert!(user.contains("'/S','/CURRENTUSER' -Wait"));
  }

  #[test]
  fn pending_installer_round_trips() {
    let p = PendingInstaller {
      version: "1.2.0".into(),
      setup: PathBuf::from(r"C:\cache\setup.exe"),
      exe: PathBuf::from(r"C:\Program Files\Nova\nova.exe"),
      per_machine: true,
    };
    assert_eq!(PendingInstaller::parse(&p.to_text()), Some(p));
    assert_eq!(PendingInstaller::parse("1.0.0\n"), None);
  }
}
