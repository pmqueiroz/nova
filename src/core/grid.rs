use bitflags::bitflags;
use iced::Color;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

pub const DEFAULT_SCROLLBACK_LIMIT: usize = 10_000;

pub const MARK_PROMPT: char = '\u{F8F0}';
pub const MARK_PROMPT_PREEXEC: char = '\u{F8F1}';
pub const MARK_COMMAND_START: char = '\u{F8F2}';

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
  pub c: SmolStr,
  pub width: u8,
  pub fg: Option<Rgb>,
  pub bg: Option<Rgb>,
  pub attrs: CellAttrs,
  pub uri: Option<Arc<str>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub [u8; 3]);

impl From<Color> for Rgb {
  fn from(c: Color) -> Self {
    let ch = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Self([ch(c.r), ch(c.g), ch(c.b)])
  }
}

impl From<Rgb> for Color {
  fn from(Rgb([r, g, b]): Rgb) -> Self {
    Color::from_rgb8(r, g, b)
  }
}

impl Cell {
  pub fn fg_color(&self) -> Option<Color> {
    self.fg.map(Color::from)
  }

  pub fn bg_color(&self) -> Option<Color> {
    self.bg.map(Color::from)
  }

  pub fn is_blank(&self) -> bool {
    self.c == " " && self.bg.is_none()
  }
}

fn without_trailing_blanks(cells: &[Cell]) -> &[Cell] {
  let len = cells
    .iter()
    .rposition(|c| !c.is_blank())
    .map_or(0, |i| i + 1);
  &cells[..len]
}

impl Default for Cell {
  fn default() -> Self {
    Self {
      c: SmolStr::new_inline(" "),
      width: 1,
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
  Normal,
  Button,
  AnyEvent,
}

pub struct Grid {
  cells: Vec<Cell>,
  origin: usize,
  pub cursor_x: usize,
  pub cursor_y: usize,
  pub current_fg: Option<Rgb>,
  pub current_bg: Option<Rgb>,
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
  pub command_history: VecDeque<CommandEntry>,
  pub suggestion: Option<String>,
  pub input_start_col: Option<usize>,
  pub input_start_row: Option<usize>,
  pub scrollback_limit: usize,
  pub cursor_visible: bool,
  pub kitty_keyboard_flags_stack: Vec<u32>,
  alt_cells: Option<Vec<Cell>>,
  alt_cursor: Option<(usize, usize)>,
  alt_scrollback: Option<VecDeque<(Vec<Cell>, bool)>>,
  alt_row_continuation: Option<Vec<bool>>,
  alt_snapshot: Option<Vec<Vec<Cell>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlCommand {
  OpenAskAi { preset: Option<std::sync::Arc<str>> },
  OpenExplainAi { preset: Option<std::sync::Arc<str>> },
  CommandFailure(u8),
  CommandComplete(u8),
  PromptReady { preexec: bool },
  CommandStart,
  SetClipboard(String),
  RequestClipboard,
  Cwd(String),
}

impl Grid {
  pub fn new(cols: usize, rows: usize) -> Self {
    let cells = vec![Cell::default(); cols * rows];
    Self {
      cells,
      origin: 0,
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
      command_history: VecDeque::new(),
      suggestion: None,
      input_start_col: None,
      input_start_row: None,
      scrollback_limit: DEFAULT_SCROLLBACK_LIMIT,
      cursor_visible: true,
      kitty_keyboard_flags_stack: Vec::new(),
      alt_cells: None,
      alt_cursor: None,
      alt_scrollback: None,
      alt_row_continuation: None,
      alt_snapshot: None,
    }
  }

  pub fn in_alt_screen(&self) -> bool {
    self.alt_cells.is_some()
  }

  pub fn kitty_keyboard_flags(&self) -> u32 {
    self.kitty_keyboard_flags_stack.last().copied().unwrap_or(0)
  }

  #[inline]
  fn phys(&self, row: usize) -> usize {
    debug_assert!(row < self.rows);
    let r = row + self.origin;
    if r >= self.rows { r - self.rows } else { r }
  }

  #[inline]
  fn row_range(&self, row: usize) -> std::ops::Range<usize> {
    let start = self.phys(row) * self.cols;
    start..start + self.cols
  }

  #[inline]
  pub fn cell(&self, row: usize, col: usize) -> &Cell {
    &self.row(row)[col]
  }

  #[inline]
  pub fn cell_mut(&mut self, row: usize, col: usize) -> &mut Cell {
    &mut self.row_mut(row)[col]
  }

  fn linearize(&mut self) {
    if self.origin != 0 {
      self.cells.rotate_left(self.origin * self.cols);
      self.origin = 0;
    }
  }

  fn swap_rows(&mut self, a: usize, b: usize) {
    let (pa, pb) = (self.phys(a), self.phys(b));
    if pa == pb {
      return;
    }
    let (lo, hi) = (pa.min(pb) * self.cols, pa.max(pb) * self.cols);
    let (head, tail) = self.cells.split_at_mut(hi);
    head[lo..lo + self.cols].swap_with_slice(&mut tail[..self.cols]);
  }

  pub fn blank_cell(&self) -> Cell {
    Cell {
      bg: self.current_bg,
      ..Cell::default()
    }
  }

  pub fn erase_cols(&mut self, row: usize, cols: std::ops::Range<usize>) {
    let blank = self.blank_cell();
    self.row_mut(row)[cols].fill(blank);
  }

  pub fn erase_rows(&mut self, rows: std::ops::Range<usize>) {
    let blank = self.blank_cell();
    for r in rows {
      self.row_mut(r).fill(blank.clone());
    }
  }

  #[inline]
  pub fn row(&self, row: usize) -> &[Cell] {
    &self.cells[self.row_range(row)]
  }

  #[inline]
  pub fn row_mut(&mut self, row: usize) -> &mut [Cell] {
    let range = self.row_range(row);
    &mut self.cells[range]
  }

  pub fn enter_alt_screen(&mut self) {
    if self.alt_cells.is_none() {
      self.linearize();
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
      self.alt_snapshot = Some(self.visible_lines());
      self.cells = cells;
      self.origin = 0;
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

  fn visible_lines(&self) -> Vec<Vec<Cell>> {
    let mut lines: Vec<Vec<Cell>> = (0..self.rows)
      .map(|r| without_trailing_blanks(self.row(r)).to_vec())
      .collect();
    while lines.last().is_some_and(Vec::is_empty) {
      lines.pop();
    }
    lines
  }

  pub fn take_alt_snapshot(&mut self) -> Option<Vec<Vec<Cell>>> {
    self.alt_snapshot.take()
  }

  pub fn keep_frame(&mut self, lines: &[Vec<Cell>]) {
    let used = (0..self.rows)
      .rev()
      .find(|&r| self.row(r).iter().any(|c| !c.is_blank()))
      .map_or(0, |r| r + 1);
    for r in 0..used {
      let row = self.row(r).to_vec();
      let cont = self.row_continuation[r];
      self.scrollback.push_back((row, cont));
    }
    for line in lines {
      let mut row = line.clone();
      row.resize(self.cols.max(row.len()), Cell::default());
      self.scrollback.push_back((row, false));
    }
    while self.scrollback.len() > self.scrollback_limit {
      self.scrollback.pop_front();
    }
    self.cells.fill(Cell::default());
    self.row_continuation.fill(false);
    self.cursor_x = 0;
    self.cursor_y = 0;
    self.wrap_next = false;
  }

  pub fn scroll_up(&mut self, n: usize) {
    let top = self.scroll_top;
    let bottom = self.scroll_bottom;
    let count = n.min(bottom.saturating_sub(top) + 1);
    let blank = self.blank_cell();

    let full_screen = top == 0 && bottom == self.rows.saturating_sub(1);
    for _ in 0..count {
      if full_screen {
        let mut row_copy = if self.scrollback.len() >= self.scrollback_limit {
          self.scrollback.pop_front().map(|(row, _)| row)
        } else {
          None
        }
        .unwrap_or_else(|| Vec::with_capacity(self.cols));
        row_copy.clear();
        row_copy.extend_from_slice(self.row(top));
        let is_cont = self.row_continuation[top];
        self.scrollback.push_back((row_copy, is_cont));
        while self.scrollback.len() > self.scrollback_limit {
          self.scrollback.pop_front();
        }
      }

      if full_screen {
        self.origin = self.phys(1 % self.rows);
      } else {
        for r in top..bottom {
          self.swap_rows(r, r + 1);
        }
      }
      self.row_continuation[top..=bottom].rotate_left(1);
      self.row_mut(bottom).fill(blank.clone());
      self.row_continuation[bottom] = false;
    }
  }

  pub fn scroll_down(&mut self, n: usize) {
    let top = self.scroll_top;
    let bottom = self.scroll_bottom;
    let count = n.min(bottom.saturating_sub(top) + 1);
    let blank = self.blank_cell();

    let full_screen = top == 0 && bottom == self.rows.saturating_sub(1);
    for _ in 0..count {
      if full_screen {
        self.origin = self.phys(self.rows - 1);
      } else {
        for r in (top..bottom).rev() {
          self.swap_rows(r, r + 1);
        }
      }
      self.row_continuation[top..=bottom].rotate_right(1);
      self.row_mut(top).fill(blank.clone());
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

    if self.alt_cells.is_some() {
      self.resize_alt(new_cols, new_rows);
    } else {
      self.reflow(new_cols, new_rows);
    }
  }

  fn resize_alt(&mut self, new_cols: usize, new_rows: usize) {
    let (old_cols, old_rows) = (self.cols, self.rows);
    self.linearize();
    let primary = self.alt_cells.take().unwrap_or_default();
    let alt = std::mem::replace(&mut self.cells, primary);
    self.row_continuation = self
      .alt_row_continuation
      .take()
      .unwrap_or_else(|| vec![false; old_rows]);
    let alt_scrollback = std::mem::replace(
      &mut self.scrollback,
      self.alt_scrollback.take().unwrap_or_default(),
    );
    let alt_cursor = (self.cursor_x, self.cursor_y);
    if let Some((x, y)) = self.alt_cursor {
      self.cursor_x = x;
      self.cursor_y = y;
    }

    self.reflow(new_cols, new_rows);

    let mut cropped = vec![Cell::default(); new_cols * new_rows];
    for r in 0..old_rows.min(new_rows) {
      for c in 0..old_cols.min(new_cols) {
        cropped[r * new_cols + c] = alt[r * old_cols + c].clone();
      }
    }
    self.alt_cells = Some(std::mem::replace(&mut self.cells, cropped));
    self.origin = 0;
    self.alt_row_continuation = Some(std::mem::replace(
      &mut self.row_continuation,
      vec![false; new_rows],
    ));
    self.alt_scrollback = Some(std::mem::replace(&mut self.scrollback, alt_scrollback));
    self.alt_cursor = Some((self.cursor_x, self.cursor_y));
    self.cursor_x = alt_cursor.0.min(new_cols - 1);
    self.cursor_y = alt_cursor.1.min(new_rows - 1);
    self.scroll_top = 0;
    self.scroll_bottom = new_rows - 1;
    self.wrap_next = false;
  }

  fn reflow(&mut self, new_cols: usize, new_rows: usize) {
    let pen_bg = self.current_bg.take();
    self.reflow_lines(new_cols, new_rows);
    self.current_bg = pen_bg;
  }

  fn reflow_lines(&mut self, new_cols: usize, new_rows: usize) {
    let mut logical_lines: Vec<Vec<Cell>> = Vec::with_capacity(self.scrollback.len() + self.rows);
    for (cells, is_cont) in std::mem::take(&mut self.scrollback) {
      match logical_lines.last_mut() {
        Some(last) if is_cont => last.extend(cells),
        _ => logical_lines.push(cells),
      }
    }

    for r in 0..self.rows {
      let row = self.row(r);
      match logical_lines.last_mut() {
        Some(last) if self.row_continuation[r] => last.extend_from_slice(row),
        _ => logical_lines.push(row.to_vec()),
      }
    }

    self.cells = vec![Cell::default(); new_cols * new_rows];
    self.origin = 0;
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
      self.write_line(without_trailing_blanks(line));
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

  pub fn find_best_suggestion(&self, partial: &str) -> Option<String> {
    if partial.is_empty() {
      return None;
    }
    let mut best: Option<&CommandEntry> = None;
    for entry in self.command_history.iter().rev() {
      if entry.text.len() > partial.len()
        && entry.text.starts_with(partial)
        && best.is_none_or(|b| entry.count > b.count)
      {
        best = Some(entry);
      }
    }
    best.map(|e| e.text[partial.len()..].to_string())
  }

  pub fn set_command_history(&mut self, history: VecDeque<CommandEntry>) {
    let mut merged: VecDeque<CommandEntry> = VecDeque::with_capacity(history.len());
    let mut index: HashMap<String, usize> = HashMap::with_capacity(history.len());
    for entry in history {
      match index.get(&entry.text) {
        Some(&i) => merged[i].count = merged[i].count.saturating_add(entry.count),
        None => {
          index.insert(entry.text.clone(), merged.len());
          merged.push_back(entry);
        }
      }
    }
    self.command_history = merged;
  }

  pub fn extract_current_input(&self) -> Option<String> {
    let (start_col, start_row) = (self.input_start_col?, self.input_start_row?);
    if start_row > self.cursor_y {
      return None;
    }
    let mut input = String::new();
    if start_row == self.cursor_y {
      for col in start_col..self.cursor_x.min(self.cols) {
        let cell = self.cell(start_row, col);
        if cell.width != 0 && cell.c.as_str() != " " {
          input.push_str(&cell.c);
        }
      }
    } else {
      for col in start_col..self.cols {
        let cell = self.cell(start_row, col);
        if cell.width != 0 && cell.c.as_str() != " " {
          input.push_str(&cell.c);
        }
      }
      for row in (start_row + 1)..self.cursor_y {
        for col in 0..self.cols {
          let cell = self.cell(row, col);
          if cell.width != 0 && cell.c.as_str() != " " {
            input.push_str(&cell.c);
          }
        }
      }
      for col in 0..self.cursor_x.min(self.cols) {
        let cell = self.cell(self.cursor_y, col);
        if cell.width != 0 && cell.c.as_str() != " " {
          input.push_str(&cell.c);
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
      let x = self.cursor_x;
      let y = self.cursor_y;
      self.row_mut(y)[x..x + count].clone_from_slice(&line[idx..end]);
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

#[cfg(test)]
mod tests {
  use super::*;

  fn entry(text: &str, count: u32) -> CommandEntry {
    CommandEntry {
      text: text.into(),
      count,
    }
  }

  #[test]
  fn loaded_history_merges_duplicates_before_suggesting() {
    let mut g = Grid::new(10, 2);
    g.set_command_history(VecDeque::from([
      entry("git commit", 2),
      entry("git checkout", 3),
      entry("git commit", 2),
    ]));
    assert_eq!(g.command_history.len(), 2);
    assert_eq!(g.find_best_suggestion("git c").as_deref(), Some("ommit"));
    assert_eq!(g.find_best_suggestion("git ch").as_deref(), Some("eckout"));
    assert_eq!(g.find_best_suggestion("git commit"), None);
    assert_eq!(g.find_best_suggestion(""), None);
  }
}
