#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod core;
mod sys;
mod ui;

pub fn main() -> ui::Result {
  ui::start()
}
