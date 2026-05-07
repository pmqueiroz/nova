use std::{fs, path::Path};

use async_channel::Sender;
use vte::Parser;

use crate::core::grid::Grid;
use crate::sys::pty::PtyCommand;

pub struct Tab {
  pub id: usize,
  pub grid: Grid,
  pub pty_tx: Option<Sender<PtyCommand>>,
  pub ansi_parser: Parser,
  pub shell: String,
  pub pwd: String,
  pub git_branch: Option<String>,
}

impl Tab {
  pub fn new(id: usize, cols: usize, rows: usize) -> Self {
    #[cfg(target_os = "windows")]
    let shell = "powershell".to_string();

    #[cfg(not(target_os = "windows"))]
    let shell = std::env::var("SHELL")
      .unwrap_or_else(|_| "bash".to_string())
      .split('/')
      .last()
      .unwrap_or("bash")
      .to_string();

    Self {
      id,
      grid: Grid::new(cols, rows),
      pty_tx: None,
      ansi_parser: Parser::new(),
      shell,
      pwd: String::from("~"),
      git_branch: None,
    }
  }

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
