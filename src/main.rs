#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod cli;
mod core;
mod sys;
mod ui;

use core::config;

pub fn main() -> ui::Result {
  if let Some(code) = cli::run_from_env() {
    std::process::exit(code);
  }

  config::init().expect("failed to load config");
  sys::notification::register();

  let c = &config::get().theme.colors;
  ui::theme::color::init_runtime(ui::theme::color::RuntimeTheme {
    background: config::parse_hex_color(&c.background).expect("invalid theme.colors.background"),
    foreground: config::parse_hex_color(&c.foreground).expect("invalid theme.colors.foreground"),
    accent: config::parse_hex_color(&c.accent).expect("invalid theme.colors.accent"),
    foreground_muted: config::parse_hex_color(&c.foreground_muted)
      .expect("invalid theme.colors.foreground-muted"),
    border: config::parse_hex_color(&c.border).expect("invalid theme.colors.border"),
    cursor: config::parse_hex_color(&c.cursor).expect("invalid theme.colors.cursor"),
  });

  ui::start()
}
