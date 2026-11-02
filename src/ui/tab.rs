use std::{
  fs,
  path::Path,
  time::{Duration, Instant},
};

use async_channel::Sender;
use iced::animation::Animation;
use vte::Parser;

use crate::core::block::Session;
use crate::core::completion::Completion;
use crate::core::config;
use crate::core::grid::{DEFAULT_SCROLLBACK_LIMIT, Grid};
use crate::core::router::Integration;
use crate::sys::pty::PtyCommand;
use crate::ui::theme::motion;

pub const BLOCKS_ENABLED: bool = true;

#[derive(Default, Debug, Clone)]
pub struct InputBox {
  pub text: String,
  pub history_cursor: Option<usize>,
  pub draft: String,
  pub submitted: u64,
  pub scrolled_up: bool,
  pub pulsed_at: Option<Instant>,
  pub completion: Option<Completion>,
}

impl InputBox {
  pub fn take(&mut self) -> String {
    self.history_cursor = None;
    self.completion = None;
    self.draft.clear();
    self.submitted += 1;
    self.pulsed_at = Some(Instant::now());
    std::mem::take(&mut self.text)
  }

  pub fn set(&mut self, text: String) {
    self.text = text;
    self.history_cursor = None;
    self.completion = None;
  }

  pub fn history_up(&mut self, session: &Session) {
    self.completion = None;
    let next = self.history_cursor.map(|c| c + 1).unwrap_or(0);
    if let Some(entry) = session.history_back(next) {
      if self.history_cursor.is_none() {
        self.draft = self.text.clone();
      }
      self.history_cursor = Some(next);
      self.text = entry.to_string();
    }
  }

  pub fn history_down(&mut self, session: &Session) {
    self.completion = None;
    match self.history_cursor {
      None => {}
      Some(0) => {
        self.history_cursor = None;
        self.text = std::mem::take(&mut self.draft);
      }
      Some(c) => {
        self.history_cursor = Some(c - 1);
        if let Some(entry) = session.history_back(c - 1) {
          self.text = entry.to_string();
        }
      }
    }
  }
}

pub struct PaneMut<'a> {
  pub grid: &'a mut Grid,
  pub session: &'a mut Session,
  pub integration: &'a mut Integration,
  pub input: &'a mut InputBox,
  pub pty_tx: &'a Option<Sender<PtyCommand>>,
}

impl<'a> PaneRef<'a> {
  pub fn raw_grid(&self) -> Option<&'a Grid> {
    if self.integration.mode == crate::core::router::Mode::Classic {
      return None;
    }
    match self.session.phase {
      crate::core::block::Phase::Raw(id) => self.session.block(id).map(|b| &b.output),
      _ => None,
    }
  }

  pub fn input_grid(&self) -> &'a Grid {
    self.raw_grid().unwrap_or(self.grid)
  }
}

impl PaneMut<'_> {
  pub fn write(&self, bytes: Vec<u8>) {
    if let Some(tx) = self.pty_tx {
      let _ = tx.send_blocking(PtyCommand::Input(bytes));
    }
  }
}

pub struct PaneRef<'a> {
  pub grid: &'a Grid,
  pub id: usize,
  pub session: &'a Session,
  pub integration: &'a Integration,
  pub input: &'a InputBox,
}

pub struct SplitPane {
  pub id: usize,
  pub grid: Grid,
  pub pty_tx: Option<Sender<PtyCommand>>,
  pub pty_alive: bool,
  pub ansi_parser: Parser,
  pub session: Session,
  pub integration: Integration,
  pub input: InputBox,
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
  pub session: Session,
  pub integration: Integration,
  pub input: InputBox,
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
  pub appear: Animation<bool>,
  pub last_command_elapsed: Option<Duration>,
  pub last_pty_output: Option<Instant>,
  pub waiting_after_exit: bool,
  pub initial_command: Option<Vec<String>>,
  pub wait_after_exit: bool,
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
      session: Session::new(initial_cwd.clone(), cols, rows),
      integration: Integration::for_shell(BLOCKS_ENABLED, &shell_cmd),
      input: InputBox::default(),
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
      appear: motion::entrance(Instant::now()),
      last_command_elapsed: None,
      last_pty_output: None,
      waiting_after_exit: false,
      initial_command: config::initial_argv(),
      wait_after_exit: config::get().general.wait_after_command,
    }
  }

  pub fn for_command(
    id: usize,
    cols: usize,
    rows: usize,
    title: &str,
    argv: Vec<String>,
    initial_cwd: String,
  ) -> Self {
    let mut tab = Self::new(id, cols, rows, title.to_string(), initial_cwd);
    tab.shell = title.to_string();
    tab.initial_command = Some(argv);
    tab.wait_after_exit = true;
    tab.integration = Integration::new(false);
    tab
  }
}

impl Tab {
  pub fn pane_mut(&mut self, split: bool) -> PaneMut<'_> {
    if split && let Some(sp) = self.split.as_mut() {
      return PaneMut {
        grid: &mut sp.grid,
        session: &mut sp.session,
        integration: &mut sp.integration,
        input: &mut sp.input,
        pty_tx: &sp.pty_tx,
      };
    }
    PaneMut {
      grid: &mut self.grid,
      session: &mut self.session,
      integration: &mut self.integration,
      input: &mut self.input,
      pty_tx: &self.pty_tx,
    }
  }

  pub fn pane(&self, split: bool) -> PaneRef<'_> {
    if split && let Some(sp) = self.split.as_ref() {
      return PaneRef {
        id: sp.id,
        grid: &sp.grid,
        session: &sp.session,
        integration: &sp.integration,
        input: &sp.input,
      };
    }
    PaneRef {
      id: self.id,
      grid: &self.grid,
      session: &self.session,
      integration: &self.integration,
      input: &self.input,
    }
  }

  pub fn active_pane_mut(&mut self) -> PaneMut<'_> {
    let split = self.active_pane_is_split;
    self.pane_mut(split)
  }

  pub fn active_pane(&self) -> PaneRef<'_> {
    self.pane(self.active_pane_is_split)
  }

  pub fn pane_by_id_mut(&mut self, id: usize) -> Option<PaneMut<'_>> {
    if self.id == id {
      return Some(self.pane_mut(false));
    }
    if self.split.as_ref().is_some_and(|s| s.id == id) {
      return Some(self.pane_mut(true));
    }
    None
  }

  pub fn panes_mut(&mut self) -> Vec<PaneMut<'_>> {
    let mut out = Vec::new();
    let Tab {
      grid,
      session,
      integration,
      input,
      pty_tx,
      split,
      ..
    } = self;
    out.push(PaneMut {
      grid,
      session,
      integration,
      input,
      pty_tx,
    });
    if let Some(sp) = split.as_mut() {
      out.push(PaneMut {
        grid: &mut sp.grid,
        session: &mut sp.session,
        integration: &mut sp.integration,
        input: &mut sp.input,
        pty_tx: &sp.pty_tx,
      });
    }
    out
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

#[cfg(test)]
mod tests {
  use super::*;

  fn session_with(history: &[&str]) -> Session {
    let mut s = Session::new("~".into(), 80, 24);
    for h in history {
      s.push_history(h);
    }
    s
  }

  #[test]
  fn history_walks_back_and_restores_draft() {
    let s = session_with(&["ls", "pwd"]);
    let mut i = InputBox {
      text: "gi".into(),
      ..InputBox::default()
    };
    i.history_up(&s);
    assert_eq!(i.text, "pwd");
    i.history_up(&s);
    assert_eq!(i.text, "ls");
    i.history_up(&s);
    assert_eq!(i.text, "ls");
    i.history_down(&s);
    assert_eq!(i.text, "pwd");
    i.history_down(&s);
    assert_eq!(i.text, "gi");
    assert_eq!(i.history_cursor, None);
  }

  #[test]
  fn take_clears_and_bumps_pulse() {
    let mut i = InputBox {
      text: "echo".into(),
      ..InputBox::default()
    };
    assert_eq!(i.take(), "echo");
    assert!(i.text.is_empty());
    assert_eq!(i.submitted, 1);
  }
}
