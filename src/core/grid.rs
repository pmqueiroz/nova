use bitflags::bitflags;
use iced::Color;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

const SCROLLBACK_LIMIT: usize = 10_000;

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct CellAttrs: u8 {
        const BOLD          = 0b00000001;
        const DIM           = 0b00000010;
        const ITALIC        = 0b00000100;
        const UNDERLINE     = 0b00001000;
        const BLINK         = 0b00010000;
        const REVERSE       = 0b00100000;
        const STRIKETHROUGH = 0b01000000;
    }
}

#[derive(Clone, Debug)]
pub struct Cell {
  pub c: char,
  pub fg: Option<Color>,
  pub bg: Option<Color>,
  pub attrs: CellAttrs,
  pub uri: Option<Arc<str>>,
}

impl Default for Cell {
  fn default() -> Self {
    Self {
      c: ' ',
      fg: None,
      bg: None,
      attrs: CellAttrs::empty(),
      uri: None,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
  pub text: String,
  pub count: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MouseMode {
  #[default]
  None,
  Normal,   // ?1000
  Button,   // ?1002
  AnyEvent, // ?1003
}

pub struct Grid {
  pub cells: Vec<Cell>,
  pub cursor_x: usize,
  pub cursor_y: usize,
  pub current_fg: Option<Color>,
  pub current_bg: Option<Color>,
  pub current_uri: Option<Arc<str>>,
  pub current_attrs: CellAttrs,
  pub mouse_mode: MouseMode,
  pub mouse_sgr: bool,
  pub bracketed_paste: bool,
  pub cols: usize,
  pub rows: usize,
  pub scroll_top: usize,
  pub scroll_bottom: usize,
  pub pwd: String,
  pub output_queue: Vec<Vec<u8>>,
  pub control_queue: Vec<ControlCommand>,
  pub saved_cursor: Option<(usize, usize)>,
  pub wrap_next: bool,
  pub row_continuation: Vec<bool>,
  pub scrollback: VecDeque<(Vec<Cell>, bool)>,
  pub scrollback_base: usize,
  pub zone_markers: Vec<usize>,
  pub command_history: VecDeque<CommandEntry>,
  pub suggestion: Option<String>,
  pub input_start_col: Option<usize>,
  pub input_start_row: Option<usize>,
  alt_cells: Option<Vec<Cell>>,
  alt_cursor: Option<(usize, usize)>,
  alt_scrollback: Option<VecDeque<(Vec<Cell>, bool)>>,
  alt_row_continuation: Option<Vec<bool>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlCommand {
  OpenAskAi { preset: Option<std::sync::Arc<str>> },
  OpenExplainAi { preset: Option<std::sync::Arc<str>> },
  CommandFailure(u8),
}

impl Grid {
  pub fn new(cols: usize, rows: usize) -> Self {
    let cells = vec![Cell::default(); cols * rows];
    Self {
      cells,
      cursor_x: 0,
      cursor_y: 0,
      current_fg: None,
      current_bg: None,
      current_uri: None,
      current_attrs: CellAttrs::empty(),
      mouse_mode: MouseMode::None,
      mouse_sgr: false,
      bracketed_paste: false,
      cols,
      rows,
      scroll_top: 0,
      scroll_bottom: rows.saturating_sub(1),
      pwd: String::from("~"),
      output_queue: Vec::new(),
      control_queue: Vec::new(),
      saved_cursor: None,
      wrap_next: false,
      row_continuation: vec![false; rows],
      scrollback: VecDeque::new(),
      scrollback_base: 0,
      zone_markers: Vec::new(),
      command_history: VecDeque::new(),
      suggestion: None,
      input_start_col: None,
      input_start_row: None,
      alt_cells: None,
      alt_cursor: None,
      alt_scrollback: None,
      alt_row_continuation: None,
    }
  }

  #[inline]
  pub fn cell_mut(&mut self, row: usize, col: usize) -> &mut Cell {
    &mut self.cells[row * self.cols + col]
  }

  pub fn row(&self, row: usize) -> &[Cell] {
    let start = row * self.cols;
    &self.cells[start..start + self.cols]
  }

  pub fn enter_alt_screen(&mut self) {
    if self.alt_cells.is_none() {
      self.alt_cells = Some(self.cells.clone());
      self.alt_cursor = Some((self.cursor_x, self.cursor_y));
      self.alt_scrollback = Some(std::mem::take(&mut self.scrollback));
      self.alt_row_continuation = Some(std::mem::take(&mut self.row_continuation));
      self.cells = vec![Cell::default(); self.cols * self.rows];
      self.row_continuation = vec![false; self.rows];
      self.cursor_x = 0;
      self.cursor_y = 0;
      self.scroll_top = 0;
      self.scroll_bottom = self.rows.saturating_sub(1);
      self.current_fg = None;
      self.current_bg = None;
      self.current_uri = None;
      self.current_attrs = CellAttrs::empty();
      self.bracketed_paste = false;
      self.wrap_next = false;
    }
  }

  pub fn leave_alt_screen(&mut self) {
    if let Some(cells) = self.alt_cells.take() {
      self.cells = cells;
    }
    if let Some((x, y)) = self.alt_cursor.take() {
      self.cursor_x = x;
      self.cursor_y = y;
    }
    if let Some(sb) = self.alt_scrollback.take() {
      self.scrollback = sb;
    }
    if let Some(rc) = self.alt_row_continuation.take() {
      self.row_continuation = rc;
    }
    self.scroll_top = 0;
    self.scroll_bottom = self.rows.saturating_sub(1);
    self.current_fg = None;
    self.current_bg = None;
    self.current_uri = None;
    self.current_attrs = CellAttrs::empty();
    self.wrap_next = false;
  }

  pub fn scroll_up(&mut self, n: usize) {
    let top = self.scroll_top;
    let bottom = self.scroll_bottom;
    let count = n.min(bottom.saturating_sub(top) + 1);

    for _ in 0..count {
      if top == 0 && bottom == self.rows.saturating_sub(1) {
        let mut row_copy = Vec::with_capacity(self.cols);
        row_copy.extend_from_slice(self.row(top));
        let is_cont = self.row_continuation[top];
        self.scrollback.push_back((row_copy, is_cont));
        if self.scrollback.len() > SCROLLBACK_LIMIT {
          self.scrollback.pop_front();
          self.scrollback_base += 1;
          self.zone_markers.retain(|&m| m >= self.scrollback_base);
        }
      }

      let start_idx = top * self.cols;
      let end_idx = (bottom + 1) * self.cols;
      self.cells[start_idx..end_idx].rotate_left(self.cols);
      self.row_continuation[top..=bottom].rotate_left(1);

      let clear_start = bottom * self.cols;
      for cell in &mut self.cells[clear_start..clear_start + self.cols] {
        *cell = Cell::default();
      }
      self.row_continuation[bottom] = false;
    }
  }

  pub fn scroll_down(&mut self, n: usize) {
    let top = self.scroll_top;
    let bottom = self.scroll_bottom;
    let count = n.min(bottom.saturating_sub(top) + 1);

    for _ in 0..count {
      let start_idx = top * self.cols;
      let end_idx = (bottom + 1) * self.cols;
      self.cells[start_idx..end_idx].rotate_right(self.cols);
      self.row_continuation[top..=bottom].rotate_right(1);

      let clear_start = top * self.cols;
      for cell in &mut self.cells[clear_start..clear_start + self.cols] {
        *cell = Cell::default();
      }
      self.row_continuation[top] = false;
    }
  }

  pub fn newline(&mut self) {
    if self.cursor_y == self.scroll_bottom {
      self.scroll_up(1);
    } else if self.cursor_y < self.rows.saturating_sub(1) {
      self.cursor_y += 1;
    }
  }

  pub fn reverse_index(&mut self) {
    if self.cursor_y == self.scroll_top {
      self.scroll_down(1);
    } else if self.cursor_y > 0 {
      self.cursor_y -= 1;
    }
  }

  pub fn resize(&mut self, new_cols: usize, new_rows: usize) {
    let new_cols = new_cols.max(1);
    let new_rows = new_rows.max(1);

    if new_cols == self.cols && new_rows == self.rows {
      return;
    }

    let mut logical_lines: Vec<Vec<Cell>> = Vec::new();
    for (cells, is_cont) in self.scrollback.iter() {
      if *is_cont {
        if let Some(last) = logical_lines.last_mut() {
          last.extend_from_slice(cells);
        } else {
          logical_lines.push(cells.clone());
        }
      } else {
        logical_lines.push(cells.clone());
      }
    }

    for r in 0..self.rows {
      let row_start = r * self.cols;
      let row_end = row_start + self.cols;
      let is_cont = self.row_continuation[r];
      if is_cont {
        if let Some(last) = logical_lines.last_mut() {
          last.extend_from_slice(&self.cells[row_start..row_end]);
        } else {
          logical_lines.push(self.cells[row_start..row_end].to_vec());
        }
      } else {
        logical_lines.push(self.cells[row_start..row_end].to_vec());
      }
    }

    self.cells = vec![Cell::default(); new_cols * new_rows];
    self.cols = new_cols;
    self.rows = new_rows;
    self.scroll_top = 0;
    self.scroll_bottom = new_rows.saturating_sub(1);
    self.cursor_x = 0;
    self.cursor_y = 0;
    self.wrap_next = false;
    self.scrollback.clear();
    self.row_continuation = vec![false; new_rows];

    for line in &logical_lines {
      self.write_line(line);
    }
  }

  pub fn push_command(&mut self, text: &str) {
    let text = text.trim().to_string();
    if text.is_empty() {
      return;
    }
    if let Some(entry) = self.command_history.iter_mut().find(|e| e.text == text) {
      entry.count += 1;
    } else {
      self
        .command_history
        .push_back(CommandEntry { text, count: 1 });
    }
    if self.command_history.len() > 2000 {
      self.command_history.pop_front();
    }
  }

  pub fn record_zone_marker(&mut self) {
    let abs = self.scrollback_base + self.scrollback.len();
    if self.zone_markers.last() != Some(&abs) {
      self.zone_markers.push(abs);
    }
  }

  pub fn find_best_suggestion(&self, partial: &str) -> Option<String> {
    if partial.is_empty() {
      return None;
    }
    let mut candidates: HashMap<&str, u64> = HashMap::new();
    for entry in self.command_history.iter().rev() {
      if entry.text.starts_with(partial) && entry.text.len() > partial.len() {
        let score = candidates.entry(entry.text.as_str()).or_insert(0);
        *score += entry.count as u64;
      }
    }
    candidates
      .into_iter()
      .max_by_key(|(_, score)| *score)
      .map(|(cmd, _)| cmd[partial.len()..].to_string())
  }

  pub fn extract_current_input(&self) -> Option<String> {
    let (start_col, start_row) = (self.input_start_col?, self.input_start_row?);
    if start_row > self.cursor_y {
      return None;
    }
    let mut input = String::new();
    if start_row == self.cursor_y {
      for col in start_col..self.cursor_x.min(self.cols) {
        let c = self.cells[start_row * self.cols + col].c;
        if c != ' ' {
          input.push(c);
        }
      }
    } else {
      for col in start_col..self.cols {
        let c = self.cells[start_row * self.cols + col].c;
        if c != ' ' {
          input.push(c);
        }
      }
      for row in (start_row + 1)..self.cursor_y {
        for col in 0..self.cols {
          let c = self.cells[row * self.cols + col].c;
          if c != ' ' {
            input.push(c);
          }
        }
      }
      for col in 0..self.cursor_x.min(self.cols) {
        let c = self.cells[self.cursor_y * self.cols + col].c;
        if c != ' ' {
          input.push(c);
        }
      }
    }
    if input.is_empty() { None } else { Some(input) }
  }

  fn write_line(&mut self, line: &[Cell]) {
    let mut idx = 0;
    while idx < line.len() {
      let available = self.cols - self.cursor_x;
      let end = (idx + available).min(line.len());
      let count = end - idx;
      let dst = self.cursor_y * self.cols + self.cursor_x;
      self.cells[dst..dst + count].clone_from_slice(&line[idx..end]);
      self.cursor_x += count;
      idx = end;

      if idx < line.len() {
        self.cursor_x = 0;
        self.newline();
        if self.cursor_y < self.rows {
          self.row_continuation[self.cursor_y] = true;
        }
      }
    }
    self.cursor_x = 0;
    self.newline();
  }
}
