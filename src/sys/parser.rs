use crate::core::grid::Grid;
use iced::Color;
use vte::{Params, Perform};

pub struct AnsiExecutor<'a> {
  pub grid: &'a mut Grid,
}

impl<'a> Perform for AnsiExecutor<'a> {
  fn print(&mut self, c: char) {
    let x = self.grid.cursor_x;
    let y = self.grid.cursor_y;

    if x < self.grid.cols && y < self.grid.rows {
      self.grid.cells[y][x] = crate::core::grid::Cell {
        c,
        fg: self.grid.current_fg,
      };
      self.grid.cursor_x += 1;
    }
  }

  fn execute(&mut self, byte: u8) {
    match byte {
      b'\n' => self.grid.newline(),
      b'\r' => self.grid.cursor_x = 0,
      0x08 => {
        if self.grid.cursor_x > 0 {
          self.grid.cursor_x -= 1;
        }
      }
      _ => {}
    }
  }

  fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, command: char) {
    match command {
      'm' => {
        for param in params.iter() {
          let code = param[0];
          match code {
            30 => self.grid.current_fg = Color::from_rgb(0.0, 0.0, 0.0),
            31 => self.grid.current_fg = Color::from_rgb(1.0, 0.0, 0.0),
            32 => self.grid.current_fg = Color::from_rgb(0.0, 1.0, 0.0),
            33 => self.grid.current_fg = Color::from_rgb(1.0, 1.0, 0.0),
            34 => self.grid.current_fg = Color::from_rgb(0.0, 0.0, 1.0),
            35 => self.grid.current_fg = Color::from_rgb(1.0, 0.0, 1.0),
            36 => self.grid.current_fg = Color::from_rgb(0.0, 1.0, 1.0),
            37 => self.grid.current_fg = Color::from_rgb(1.0, 1.0, 1.0),
            _ => {}
          }
        }
      }
      'A' => { /*cursor up*/ }
      'B' => { /*cursor down*/ }
      'C' => { /*cursor forward*/ }
      'D' => { /*cursor backward*/ }
      'J' => { /*erase in display*/ }
      _ => {}
    }
  }
}
