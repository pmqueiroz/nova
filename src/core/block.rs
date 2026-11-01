use std::collections::VecDeque;

use chrono::{DateTime, Local};

use crate::core::grid::{Cell, CommandEntry, DEFAULT_SCROLLBACK_LIMIT, Grid};

pub type BlockId = u64;

pub const HISTORY_LIMIT: usize = 2000;
pub const SIGPIPE_EXIT: i32 = 141;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockState {
  Pending,
  Running,
  Done,
}

pub struct Block {
  pub id: BlockId,
  pub command: String,
  pub cwd: String,
  pub started_at: DateTime<Local>,
  pub finished_at: Option<DateTime<Local>>,
  pub exit_code: Option<i32>,
  pub output: Grid,
  pub state: BlockState,
  pub born: std::time::Instant,
}

impl Block {
  pub fn new(id: BlockId, command: String, cwd: String, cols: usize, rows: usize) -> Self {
    let mut output = Grid::new(cols.max(1), rows.max(1));
    output.pwd = cwd.clone();
    Self {
      id,
      command,
      cwd,
      started_at: Local::now(),
      finished_at: None,
      exit_code: None,
      output,
      state: BlockState::Pending,
      born: std::time::Instant::now(),
    }
  }

  pub fn is_error(&self) -> bool {
    matches!(self.exit_code, Some(c) if c != 0 && c != SIGPIPE_EXIT)
  }

  pub fn is_finished(&self) -> bool {
    self.state == BlockState::Done
  }

  pub fn program(&self) -> &str {
    self.command.split_whitespace().next().unwrap_or("")
  }

  pub fn split_command(&self) -> (&str, &str) {
    match self.command.find(' ') {
      Some(i) => (&self.command[..i], &self.command[i..]),
      None => (self.command.as_str(), ""),
    }
  }

  fn raw_line(&self, i: usize) -> Option<&[Cell]> {
    let sb = self.output.scrollback.len();
    if i < sb {
      self.output.scrollback.get(i).map(|(r, _)| r.as_slice())
    } else if i - sb < self.output.rows {
      Some(self.output.row(i - sb))
    } else {
      None
    }
  }

  fn raw_end(&self) -> usize {
    let sb = self.output.scrollback.len();
    if let Some(r) = last_used_row(&self.output) {
      return sb + r + 1;
    }
    (0..sb)
      .rev()
      .find(|&i| self.raw_line(i).is_some_and(|l| !row_is_blank(l)))
      .map_or(0, |i| i + 1)
  }

  pub fn first_line(&self) -> usize {
    let end = self.raw_end();
    (0..end)
      .find(|&i| self.raw_line(i).is_some_and(|l| !row_is_blank(l)))
      .unwrap_or(end)
  }

  pub fn line_count(&self) -> usize {
    self.raw_end().saturating_sub(self.first_line())
  }

  pub fn has_output(&self) -> bool {
    self.line_count() > 0
  }

  pub fn line(&self, i: usize) -> Option<&[Cell]> {
    if i >= self.line_count() {
      return None;
    }
    self.raw_line(self.first_line() + i)
  }

  pub fn line_text(&self, i: usize) -> String {
    self
      .line(i)
      .map(|cells| {
        cells
          .iter()
          .filter(|c| c.width != 0)
          .map(|c| c.c.as_ref())
          .collect::<String>()
          .trim_end()
          .to_string()
      })
      .unwrap_or_default()
  }

  pub fn text(&self) -> String {
    (0..self.line_count())
      .map(|i| self.line_text(i))
      .collect::<Vec<_>>()
      .join("\n")
  }
}

fn row_is_blank(cells: &[Cell]) -> bool {
  cells
    .iter()
    .all(|c| c.c.as_ref().trim().is_empty() && c.bg.is_none())
}

pub fn last_used_row(grid: &Grid) -> Option<usize> {
  let cursor_row = (grid.cursor_x > 0).then_some(grid.cursor_y);
  let content_row = (0..grid.rows).rev().find(|&r| !row_is_blank(grid.row(r)));
  match (cursor_row, content_row) {
    (Some(a), Some(b)) => Some(a.max(b)),
    (a, b) => a.or(b),
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
  Prompt,
  Running(BlockId),
  Raw(BlockId),
}

pub struct Session {
  pub blocks: Vec<Block>,
  pub history: VecDeque<CommandEntry>,
  pub phase: Phase,
  pub cwd: String,
  pub cols: usize,
  pub rows: usize,
  pub scrollback_limit: usize,
  next_id: BlockId,
}

impl Session {
  pub fn new(cwd: String, cols: usize, rows: usize) -> Self {
    Self {
      blocks: Vec::new(),
      history: VecDeque::new(),
      phase: Phase::Prompt,
      cwd,
      cols,
      rows,
      scrollback_limit: DEFAULT_SCROLLBACK_LIMIT,
      next_id: 1,
    }
  }

  pub fn is_running(&self) -> bool {
    !matches!(self.phase, Phase::Prompt)
  }

  pub fn is_raw(&self) -> bool {
    matches!(self.phase, Phase::Raw(_))
  }

  pub fn active_id(&self) -> Option<BlockId> {
    match self.phase {
      Phase::Running(id) | Phase::Raw(id) => Some(id),
      Phase::Prompt => None,
    }
  }

  pub fn block(&self, id: BlockId) -> Option<&Block> {
    self.blocks.iter().find(|b| b.id == id)
  }

  pub fn block_mut(&mut self, id: BlockId) -> Option<&mut Block> {
    self.blocks.iter_mut().find(|b| b.id == id)
  }

  pub fn active(&self) -> Option<&Block> {
    self.active_id().and_then(|id| self.block(id))
  }

  pub fn active_mut(&mut self) -> Option<&mut Block> {
    let id = self.active_id()?;
    self.block_mut(id)
  }

  pub fn submit(&mut self, command: &str) -> Option<BlockId> {
    let command = command.trim();
    if command.is_empty() || self.is_running() {
      return None;
    }
    self.push_history(command);
    let id = self.next_id;
    self.next_id += 1;
    let mut block = Block::new(
      id,
      command.to_string(),
      self.cwd.clone(),
      self.cols,
      self.rows,
    );
    block.output.scrollback_limit = self.scrollback_limit;
    self.blocks.push(block);
    self.phase = Phase::Running(id);
    Some(id)
  }

  pub fn mark_started(&mut self) {
    if let Some(b) = self.active_mut()
      && b.state == BlockState::Pending
    {
      b.state = BlockState::Running;
    }
  }

  pub fn finish(&mut self, exit_code: i32) -> Option<BlockId> {
    let id = self.active_id()?;
    if let Some(b) = self.block_mut(id) {
      b.state = BlockState::Done;
      b.exit_code = Some(exit_code);
      b.finished_at = Some(Local::now());
    }
    self.phase = Phase::Prompt;
    Some(id)
  }

  pub fn enter_raw(&mut self) {
    if let Phase::Running(id) = self.phase {
      self.phase = Phase::Raw(id);
    }
  }

  pub fn leave_raw(&mut self) {
    if let Phase::Raw(id) = self.phase {
      self.phase = Phase::Running(id);
    }
  }

  pub fn clear(&mut self) {
    let active = self.active_id();
    self.blocks.retain(|b| Some(b.id) == active);
  }

  pub fn last_finished(&self) -> Option<&Block> {
    self.blocks.iter().rev().find(|b| b.is_finished())
  }

  pub fn last_failure(&self) -> Option<&Block> {
    self.last_finished().filter(|b| b.is_error())
  }

  pub fn resize(&mut self, cols: usize, rows: usize) {
    self.cols = cols.max(1);
    self.rows = rows.max(1);
    for b in self.blocks.iter_mut() {
      if b.output.cols != self.cols || b.output.rows != self.rows {
        b.output.resize(self.cols, self.rows);
      }
    }
  }

  pub fn push_history(&mut self, text: &str) {
    let text = text.trim();
    if text.is_empty() {
      return;
    }
    if let Some(pos) = self.history.iter().position(|e| e.text == text) {
      let mut entry = self.history.remove(pos).unwrap();
      entry.count += 1;
      self.history.push_back(entry);
    } else {
      self.history.push_back(CommandEntry {
        text: text.to_string(),
        count: 1,
      });
    }
    while self.history.len() > HISTORY_LIMIT {
      self.history.pop_front();
    }
  }

  pub fn history_back(&self, steps_back: usize) -> Option<&str> {
    let len = self.history.len();
    (steps_back < len).then(|| self.history[len - 1 - steps_back].text.as_str())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::sys::parser::AnsiExecutor;
  use vte::Parser;

  fn feed(b: &mut Block, s: &str) {
    let mut p = Parser::new();
    let mut exec = AnsiExecutor {
      grid: &mut b.output,
      bell_pending: false,
    };
    p.advance(&mut exec, s.as_bytes());
  }

  #[test]
  fn submit_creates_pending_block_and_runs() {
    let mut s = Session::new("~".into(), 80, 24);
    let id = s.submit("  ls -la ").unwrap();
    assert_eq!(s.phase, Phase::Running(id));
    let b = s.block(id).unwrap();
    assert_eq!(b.command, "ls -la");
    assert_eq!(b.state, BlockState::Pending);
    assert_eq!(b.split_command(), ("ls", " -la"));
  }

  #[test]
  fn cannot_submit_while_running_or_empty() {
    let mut s = Session::new("~".into(), 80, 24);
    assert!(s.submit("   ").is_none());
    s.submit("sleep 1").unwrap();
    assert!(s.submit("ls").is_none());
  }

  #[test]
  fn finish_records_exit_and_returns_to_prompt() {
    let mut s = Session::new("~".into(), 80, 24);
    let id = s.submit("clsd").unwrap();
    s.mark_started();
    assert_eq!(s.block(id).unwrap().state, BlockState::Running);
    s.finish(127);
    assert_eq!(s.phase, Phase::Prompt);
    let b = s.block(id).unwrap();
    assert!(b.is_error());
    assert_eq!(s.last_failure().map(|b| b.id), Some(id));
  }

  #[test]
  fn closed_pager_pipe_is_not_a_failure() {
    let mut s = Session::new("~".into(), 80, 24);
    let id = s.submit("git log").unwrap();
    s.mark_started();
    s.finish(SIGPIPE_EXIT);
    assert!(!s.block(id).unwrap().is_error());
    assert!(s.last_failure().is_none());
  }

  #[test]
  fn raw_phase_round_trips() {
    let mut s = Session::new("~".into(), 80, 24);
    let id = s.submit("vim").unwrap();
    s.enter_raw();
    assert_eq!(s.phase, Phase::Raw(id));
    s.leave_raw();
    assert_eq!(s.phase, Phase::Running(id));
  }

  #[test]
  fn clear_keeps_running_block() {
    let mut s = Session::new("~".into(), 80, 24);
    s.submit("a").unwrap();
    s.finish(0);
    let id = s.submit("b").unwrap();
    s.clear();
    assert_eq!(s.blocks.len(), 1);
    assert_eq!(s.blocks[0].id, id);
  }

  #[test]
  fn output_lines_skip_trailing_blank_rows() {
    let mut s = Session::new("~".into(), 20, 5);
    let id = s.submit("echo").unwrap();
    let b = s.block_mut(id).unwrap();
    assert!(!b.has_output());
    feed(b, "hello\r\nworld\r\n");
    assert_eq!(b.line_count(), 2);
    assert_eq!(b.text(), "hello\nworld");
  }

  #[test]
  fn leading_blank_rows_are_trimmed() {
    let mut s = Session::new("~".into(), 20, 6);
    let id = s.submit("x").unwrap();
    let b = s.block_mut(id).unwrap();
    feed(
      b,
      "[4;1Hhi
",
    );
    assert_eq!(b.line_count(), 1);
    assert_eq!(b.text(), "hi");
  }

  #[test]
  fn narrowing_resize_keeps_output_tight() {
    let mut s = Session::new("~".into(), 40, 8);
    let id = s.submit("echo").unwrap();
    feed(
      s.block_mut(id).unwrap(),
      "left
",
    );
    s.resize(20, 8);
    let b = s.block(id).unwrap();
    assert_eq!(b.text(), "left");
  }

  #[test]
  fn output_scrolls_into_scrollback() {
    let mut s = Session::new("~".into(), 10, 3);
    let id = s.submit("seq").unwrap();
    let b = s.block_mut(id).unwrap();
    feed(b, "1\r\n2\r\n3\r\n4\r\n5");
    assert_eq!(b.line_count(), 5);
    assert_eq!(b.line_text(0), "1");
    assert_eq!(b.line_text(4), "5");
  }

  #[test]
  fn history_moves_repeats_to_end_and_counts() {
    let mut s = Session::new("~".into(), 80, 24);
    s.push_history("ls");
    s.push_history("pwd");
    s.push_history("ls");
    assert_eq!(s.history_back(0), Some("ls"));
    assert_eq!(s.history_back(1), Some("pwd"));
    assert_eq!(s.history_back(2), None);
    assert_eq!(s.history.back().unwrap().count, 2);
  }
}
