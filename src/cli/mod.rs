pub mod commands;
pub mod constants;

use crate::cli::commands::help;

pub fn run_from_env() -> Option<i32> {
  let mut args: Vec<String> = std::env::args().collect();
  if args.len() <= 1 {
    return None;
  }

  args.remove(0);

  match args.first().map(|s| s.as_str()) {
    Some("help") | Some("--help") | Some("-h") => {
      eprint!("{}", help::usage());
      Some(0)
    }
    Some("--version") | Some("-v") => {
      eprintln!("nova {}", env!("CARGO_PKG_VERSION"));
      Some(0)
    }
    Some("ask") => Some(commands::ask::run(args.get(1..).unwrap_or_default())),
    Some("config") => Some(commands::config::run(args.get(1..).unwrap_or_default())),
    Some("explain") => Some(commands::explain::run(args.get(1..).unwrap_or_default())),
    Some(cmd) => {
      eprintln!("error: unknown command '{cmd}'\n\n{}", help::usage());
      Some(2)
    }
    None => None,
  }
}

#[cfg(windows)]
pub fn detach_console() {
  use windows_sys::Win32::System::Console::{FreeConsole, GetConsoleProcessList, GetConsoleWindow};
  use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, ShowWindow};
  unsafe {
    let mut pids = [0u32; 2];
    let count = GetConsoleProcessList(pids.as_mut_ptr(), 2);
    if count == 1 {
      let hwnd = GetConsoleWindow();
      if !hwnd.is_null() {
        ShowWindow(hwnd, SW_HIDE);
      }
    }
    let _ = FreeConsole();
  }
}
