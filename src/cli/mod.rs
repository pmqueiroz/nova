use std::io::Write;

use base64::Engine;

pub mod constants;

fn usage() -> &'static str {
  "nova <command>\n\nCommands:\n  ask    Open Ask AI modal (Nova terminal only)\n  help   Show this help\n"
}

pub fn run_from_env() -> Option<i32> {
  let mut args: Vec<String> = std::env::args().collect();
  if args.len() <= 1 {
    return None;
  }

  #[cfg(windows)]
  win::attach_parent_console();

  args.remove(0);

  match args.first().map(|s| s.as_str()) {
    Some("help") | Some("--help") | Some("-h") => {
      eprint!("{}", usage());
      Some(0)
    }
    Some("ask") => {
      if std::env::var(constants::ENV_IN_NOVA).ok().as_deref() != Some("1") {
        eprintln!("error: 'nova ask' must be run inside Nova terminal");
        return Some(1);
      }

      let preset = args.get(1..).unwrap_or_default().join(" ");

      let mut out = std::io::stdout().lock();
      let _ = out.write_all(constants::OSC_PREFIX);
      let _ = out.write_all(b";ask_ai");
      if !preset.trim().is_empty() {
        let b64 = base64::engine::general_purpose::STANDARD.encode(preset.as_bytes());
        let _ = out.write_all(b";");
        let _ = out.write_all(b64.as_bytes());
      }
      let _ = out.write_all(constants::OSC_SUFFIX_BEL);
      let _ = out.flush();
      Some(0)
    }
    Some(cmd) => {
      eprintln!("error: unknown command '{cmd}'\n\n{}", usage());
      Some(2)
    }
    None => None,
  }
}

#[cfg(windows)]
mod win {
  use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};

  pub fn attach_parent_console() {
    unsafe {
      let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
  }
}
