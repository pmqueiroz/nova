use iced::Color;

#[derive(Clone, Copy, Debug)]
pub struct Cell {
  pub c: char,
  pub fg: Color,
}

impl Default for Cell {
  fn default() -> Self {
    Self {
      c: ' ',
      fg: Color::WHITE,
    }
  }
}

pub struct Grid {
  pub cells: Vec<Vec<Cell>>,
  pub cursor_x: usize,
  pub cursor_y: usize,
  pub current_fg: Color,
  pub cols: usize,
  pub rows: usize,
  pub pwd: String,
  pub output_queue: Vec<Vec<u8>>,
}

impl Grid {
  pub fn new(cols: usize, rows: usize) -> Self {
    let cells = vec![vec![Cell::default(); cols]; rows];
    Self {
      cells,
      cursor_x: 0,
      cursor_y: 0,
      current_fg: Color::WHITE,
      cols,
      rows,
      pwd: String::from("~"),
      output_queue: Vec::new(),
    }
  }

  pub fn newline(&mut self) {
    self.cursor_x = 0;
    if self.cursor_y < self.rows - 1 {
      self.cursor_y += 1;
    } else {
      self.cells.remove(0);
      self.cells.push(vec![Cell::default(); self.cols]);
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

    self.cursor_x = self.cursor_x.min(self.cols.saturating_sub(1));
    self.cursor_y = self.cursor_y.min(self.rows.saturating_sub(1));
  }
}
