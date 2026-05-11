use std::io::Write;

use base64::Engine;

use crate::cli::constants;

pub fn run(args: &[String]) -> i32 {
  if std::env::var(constants::ENV_IN_NOVA).ok().as_deref() != Some("1") {
    eprintln!("error: 'nova ask' must be run inside Nova terminal");
    return 1;
  }

  let preset = args.get(0..).unwrap_or_default().join(" ");

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
  0
}
