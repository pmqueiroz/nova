use iced::Color;

#[derive(Clone, Copy, Debug)]
pub struct Cell {
  pub c: char,
  pub fg: Color,
  pub bg: Color,
  pub reverse: bool,
}

impl Default for Cell {
  fn default() -> Self {
    Self {
      c: ' ',
      fg: Color::WHITE,
      bg: Color::TRANSPARENT,
      reverse: false,
    }
  }
}

pub struct Grid {
  pub cells: Vec<Vec<Cell>>,
  pub cursor_x: usize,
  pub cursor_y: usize,
  pub current_fg: Color,
  pub current_bg: Color,
  pub reverse_video: bool,
  pub cols: usize,
  pub rows: usize,
  pub scroll_top: usize,
  pub scroll_bottom: usize,
  pub pwd: String,
  pub output_queue: Vec<Vec<u8>>,
  pub saved_cursor: Option<(usize, usize)>,
  pub wrap_next: bool,
  alt_cells: Option<Vec<Vec<Cell>>>,
  alt_cursor: Option<(usize, usize)>,
}

impl Grid {
  pub fn new(cols: usize, rows: usize) -> Self {
    let cells = vec![vec![Cell::default(); cols]; rows];
    Self {
      cells,
      cursor_x: 0,
      cursor_y: 0,
      current_fg: Color::WHITE,
      current_bg: Color::TRANSPARENT,
      reverse_video: false,
      cols,
      rows,
      scroll_top: 0,
      scroll_bottom: rows.saturating_sub(1),
      pwd: String::from("~"),
      output_queue: Vec::new(),
      saved_cursor: None,
      wrap_next: false,
      alt_cells: None,
      alt_cursor: None,
    }
  }

  pub fn enter_alt_screen(&mut self) {
    if self.alt_cells.is_none() {
      self.alt_cells = Some(self.cells.clone());
      self.alt_cursor = Some((self.cursor_x, self.cursor_y));
      self.cells = vec![vec![Cell::default(); self.cols]; self.rows];
      self.cursor_x = 0;
      self.cursor_y = 0;
      self.scroll_top = 0;
      self.scroll_bottom = self.rows.saturating_sub(1);
      self.current_fg = Color::WHITE;
      self.current_bg = Color::TRANSPARENT;
      self.reverse_video = false;
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
    self.scroll_top = 0;
    self.scroll_bottom = self.rows.saturating_sub(1);
    self.current_fg = Color::WHITE;
    self.current_bg = Color::TRANSPARENT;
    self.reverse_video = false;
    self.wrap_next = false;
  }

  pub fn newline(&mut self) {
    if self.cursor_y == self.scroll_bottom {
      self.cells.remove(self.scroll_top);
      self
        .cells
        .insert(self.scroll_bottom, vec![Cell::default(); self.cols]);
    } else if self.cursor_y < self.rows.saturating_sub(1) {
      self.cursor_y += 1;
    }
  }

  pub fn reverse_index(&mut self) {
    if self.cursor_y == self.scroll_top {
      self
        .cells
        .insert(self.scroll_top, vec![Cell::default(); self.cols]);
      if self.scroll_bottom + 1 < self.cells.len() {
        self.cells.remove(self.scroll_bottom + 1);
      } else {
        self.cells.pop();
      }
    } else if self.cursor_y > 0 {
      self.cursor_y -= 1;
    }
  }

  pub fn resize(&mut self, new_cols: usize, new_rows: usize) {
    let new_cols = new_cols.max(1);
    let new_rows = new_rows.max(1);

    self
      .cells
      .resize(new_rows, vec![Cell::default(); self.cols]);
    for row in self.cells.iter_mut() {
      row.resize(new_cols, Cell::default());
    }

    self.cols = new_cols;
    self.rows = new_rows;
    self.scroll_top = 0;
    self.scroll_bottom = new_rows.saturating_sub(1);
    self.cursor_x = self.cursor_x.min(self.cols.saturating_sub(1));
    self.cursor_y = self.cursor_y.min(self.rows.saturating_sub(1));
  }
}
