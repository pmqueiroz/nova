use std::{fs, path::Path};

use async_channel::Sender;
use vte::Parser;

use crate::core::grid::Grid;
use crate::sys::pty::PtyCommand;

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
}

impl Tab {
  pub fn new(id: usize, cols: usize, rows: usize, shell_cmd: String) -> Self {
    let shell = shell_display_name(&shell_cmd);
    Self {
      id,
      grid: Grid::new(cols, rows),
      pty_tx: None,
      pty_alive: true,
      ansi_parser: Parser::new(),
      shell,
      shell_cmd,
      pwd: String::from("~"),
      git_branch: None,
      pending_command: None,
      scroll_offset: 0,
    }
  }

}

fn shell_display_name(cmd: &str) -> String {
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

impl Tab {
  pub fn update_git_status(&mut self) {
    let head_path = Path::new(&self.pwd).join(".git").join("HEAD");

    if let Ok(content) = fs::read_to_string(head_path) {
      let content = content.trim();

      if let Some(branch) = content.strip_prefix("ref: refs/heads/") {
        return self.git_branch = Some(branch.trim().to_string());
      } else {
        return self.git_branch = Some(content.chars().take(7).collect());
      }
    }
    self.git_branch = None;
  }
}
