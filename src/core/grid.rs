use bitflags::bitflags;
use iced::Color;
use std::collections::VecDeque;
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
  pub saved_cursor: Option<(usize, usize)>,
  pub wrap_next: bool,
  pub scrollback: VecDeque<Vec<Cell>>,
  alt_cells: Option<Vec<Cell>>,
  alt_cursor: Option<(usize, usize)>,
  alt_scrollback: Option<VecDeque<Vec<Cell>>>,
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
      saved_cursor: None,
      wrap_next: false,
      scrollback: VecDeque::new(),
      alt_cells: None,
      alt_cursor: None,
      alt_scrollback: None,
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
      self.cells = vec![Cell::default(); self.cols * self.rows];
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
        self.scrollback.push_back(row_copy);
        if self.scrollback.len() > SCROLLBACK_LIMIT {
          self.scrollback.pop_front();
        }
      }

      let start_idx = top * self.cols;
      let end_idx = (bottom + 1) * self.cols;
      self.cells[start_idx..end_idx].rotate_left(self.cols);

      let clear_start = bottom * self.cols;
      for cell in &mut self.cells[clear_start..clear_start + self.cols] {
        *cell = Cell::default();
      }
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

      let clear_start = top * self.cols;
      for cell in &mut self.cells[clear_start..clear_start + self.cols] {
        *cell = Cell::default();
      }
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

    let mut new_cells = vec![Cell::default(); new_cols * new_rows];
    let min_rows = self.rows.min(new_rows);
    let min_cols = self.cols.min(new_cols);

    for r in 0..min_rows {
      let old_start = r * self.cols;
      let new_start = r * new_cols;
      new_cells[new_start..new_start + min_cols]
        .clone_from_slice(&self.cells[old_start..old_start + min_cols]);
    }

    self.cells = new_cells;
    self.cols = new_cols;
    self.rows = new_rows;
    self.scroll_top = 0;
    self.scroll_bottom = new_rows.saturating_sub(1);
    self.cursor_x = self.cursor_x.min(self.cols.saturating_sub(1));
    self.cursor_y = self.cursor_y.min(self.rows.saturating_sub(1));
  }
}
