use std::{
  fs,
  path::Path,
  time::{Duration, Instant},
};

use async_channel::Sender;
use vte::Parser;

use crate::core::config;
use crate::core::grid::{DEFAULT_SCROLLBACK_LIMIT, Grid};
use crate::sys::kitty_graphics::{ApcState, PendingKittyImage};
use crate::sys::pty::PtyCommand;

pub struct SplitPane {
  pub id: usize,
  pub grid: Grid,
  pub pty_tx: Option<Sender<PtyCommand>>,
  pub pty_alive: bool,
  pub ansi_parser: Parser,
  pub shell: String,
  pub shell_cmd: String,
  pub pwd: String,
  pub git_branch: Option<String>,
  pub current_input: String,
  pub command_start: Option<Instant>,
  pub last_command_elapsed: Option<Duration>,
  pub last_pty_output: Option<Instant>,
  pub scroll_offset: usize,
  pub initial_cwd: String,
  pub waiting_after_exit: bool,
  pub apc_state: ApcState,
  pub pending_kitty: Option<PendingKittyImage>,
}

impl SplitPane {
  pub fn update_git_status(&mut self) {
    self.git_branch = read_git_branch(&self.pwd);
  }
}

pub struct Tab {
  pub id: usize,
  pub grid: Grid,
  pub pty_tx: Option<Sender<PtyCommand>>,
  pub pty_alive: bool,
  pub ansi_parser: Parser,
  pub shell: String,
  pub shell_cmd: String,
  pub pwd: String,
  pub git_branch: Option<String>,
  pub pending_command: Option<Vec<u8>>,
  pub scroll_offset: usize,
  pub initial_cwd: String,
  pub current_input: String,
  pub split: Option<SplitPane>,
  pub active_pane_is_split: bool,
  pub split_ratio: f32,
  pub shell_at_prompt: bool,
  pub command_start: Option<Instant>,
  pub command_done: bool,
  pub last_command_elapsed: Option<Duration>,
  pub last_pty_output: Option<Instant>,
  pub waiting_after_exit: bool,
  pub initial_command: Option<String>,
  pub apc_state: ApcState,
  pub pending_kitty: Option<PendingKittyImage>,
}

impl Tab {
  pub fn new(id: usize, cols: usize, rows: usize, shell_cmd: String, initial_cwd: String) -> Self {
    let shell = shell_display_name(&shell_cmd);
    let scrollback_limit = config::get()
      .general
      .scrollback
      .unwrap_or(DEFAULT_SCROLLBACK_LIMIT);
    let mut grid = Grid::new(cols, rows);
    grid.scrollback_limit = scrollback_limit;
    let pwd = if initial_cwd.is_empty() {
      String::from("~")
    } else {
      initial_cwd.clone()
    };
    grid.pwd = pwd.clone();
    let git_branch = read_git_branch(&pwd);
    Self {
      id,
      grid,
      pty_tx: None,
      pty_alive: true,
      ansi_parser: Parser::new(),
      shell,
      shell_cmd,
      pwd,
      git_branch,
      pending_command: None,
      scroll_offset: 0,
      initial_cwd,
      current_input: String::new(),
      split: None,
      active_pane_is_split: false,
      split_ratio: 0.5,
      shell_at_prompt: true,
      command_start: None,
      command_done: false,
      last_command_elapsed: None,
      last_pty_output: None,
      waiting_after_exit: false,
      initial_command: config::get().general.initial_command.clone(),
      apc_state: ApcState::default(),
      pending_kitty: None,
    }
  }
}

pub fn shell_display_name(cmd: &str) -> String {
  #[cfg(target_os = "windows")]
  {
    let lower = cmd.to_lowercase();
    if lower == "powershell" || lower.ends_with("powershell.exe") {
      return "powershell".to_string();
    }
    if lower == "pwsh" || lower.ends_with("pwsh.exe") {
      return "pwsh".to_string();
    }
    if lower == "cmd" || lower.ends_with("cmd.exe") {
      return "cmd".to_string();
    }
    std::path::Path::new(cmd)
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or(cmd)
      .to_string()
  }
  #[cfg(not(target_os = "windows"))]
  {
    std::path::Path::new(cmd)
      .file_name()
      .and_then(|s| s.to_str())
      .unwrap_or(cmd)
      .to_string()
  }
}

fn read_git_branch(pwd: &str) -> Option<String> {
  let mut dir = Path::new(pwd);
  loop {
    let head_path = dir.join(".git").join("HEAD");
    if let Ok(content) = fs::read_to_string(head_path) {
      let content = content.trim();
      return if let Some(branch) = content.strip_prefix("ref: refs/heads/") {
        Some(branch.trim().to_string())
      } else {
        Some(content.chars().take(7).collect())
      };
    }
    dir = dir.parent()?;
  }
}

impl Tab {
  pub fn update_git_status(&mut self) {
    self.git_branch = read_git_branch(&self.pwd);
  }
}
