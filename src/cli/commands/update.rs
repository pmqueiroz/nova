use std::io::Write;
use std::path::Path;

use crate::core::install::{self, Install, UpdatePlan};
use crate::core::update;
use crate::sys::update as sys_update;

pub fn run(args: &[String]) -> i32 {
  let install = match args.first().map(String::as_str) {
    None => false,
    Some("--install") | Some("-i") => true,
    Some("help") | Some("--help") | Some("-h") => {
      eprint!("{}", usage());
      return 0;
    }
    Some(arg) => {
      eprintln!("error: unexpected argument '{arg}'\n\n{}", usage());
      return 2;
    }
  };

  let current = update::current_version();
  let offer = match sys_update::check_blocking() {
    Ok(Some(offer)) => offer,
    Ok(None) => {
      println!("nova {current} is up to date");
      return 0;
    }
    Err(e) => {
      eprintln!("error: update check failed: {e}");
      return 1;
    }
  };

  println!(
    "nova {} is available (current {current}), installed via {}",
    offer.release.version, offer.install
  );
  if !install {
    match &offer.plan {
      UpdatePlan::OpenPage(url) => println!("{url}"),
      _ => println!("run 'nova update --install' to update"),
    }
    return 0;
  }

  let mut last = u32::MAX;
  let file = match &offer.plan {
    UpdatePlan::OpenPage(url) => {
      println!("this install can't be updated automatically, download it from:\n{url}");
      return 0;
    }
    _ => sys_update::prepare_blocking(&offer, &mut |p| {
      let pct = (p * 100.0) as u32;
      if pct != last {
        last = pct;
        eprint!("\rdownloading {pct}%");
        let _ = std::io::stderr().flush();
      }
    }),
  };
  if last != u32::MAX {
    eprintln!();
  }
  let file = match file {
    Ok(f) => f,
    Err(e) => {
      eprintln!("error: {e}");
      return 1;
    }
  };

  match &offer.plan {
    UpdatePlan::Delegate { argv, .. } => run_delegate(&install::fill_file(argv, Path::new(&file))),
    _ => {
      match offer.install {
        Install::Nsis { .. } => println!(
          "nova {} is staged; the installer runs the next time Nova starts",
          offer.release.version
        ),
        _ => println!(
          "updated to nova {}; restart Nova to use it",
          offer.release.version
        ),
      }
      0
    }
  }
}

fn run_delegate(argv: &[String]) -> i32 {
  let Some((program, args)) = argv.split_first() else {
    return 1;
  };
  println!("running: {}", argv.join(" "));
  match std::process::Command::new(program).args(args).status() {
    Ok(status) => status.code().unwrap_or(1),
    Err(e) => {
      eprintln!("error: {program}: {e}");
      1
    }
  }
}

fn usage() -> &'static str {
  "nova update [--install]

Checks GitHub for a newer Nova release.

  --install, -i  Install it. Package-manager installs (Scoop, Homebrew,
                 apt, dnf, AUR, cargo) run the manager's own upgrade
                 command; installer, portable, DMG and AppImage installs
                 are downloaded, checksum-verified and swapped in place.
"
}
