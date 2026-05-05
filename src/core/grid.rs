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
}
