use crate::core::update;

pub fn run(args: &[String]) -> i32 {
  if let Some(arg) = args.first() {
    if matches!(arg.as_str(), "help" | "--help" | "-h") {
      eprint!("{}", usage());
      return 0;
    }
    eprintln!("error: unexpected argument '{arg}'\n\n{}", usage());
    return 2;
  }

  let current = update::current_version();
  match crate::sys::update::check_blocking() {
    Ok(Some(release)) => {
      println!("nova {} is available (current {current})", release.version);
      println!("{}", release.url);
      0
    }
    Ok(None) => {
      println!("nova {current} is up to date");
      0
    }
    Err(e) => {
      eprintln!("error: update check failed: {e}");
      1
    }
  }
}

fn usage() -> &'static str {
  "nova update

Checks GitHub for a newer Nova release and prints its download page.
Install it with the package manager or installer you used originally.
"
}
