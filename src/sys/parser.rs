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
      'K' => {
        let mode = params.iter().next().map_or(0, |p| p[0]);
        let x = self.grid.cursor_x;
        let y = self.grid.cursor_y;

        if y < self.grid.rows {
          match mode {
            0 => {
              for col in x..self.grid.cols {
                self.grid.cells[y][col] = crate::core::grid::Cell::default();
              }
            }
            1 => {
              for col in 0..=x {
                self.grid.cells[y][col] = crate::core::grid::Cell::default();
              }
            }
            2 => {
              for col in 0..self.grid.cols {
                self.grid.cells[y][col] = crate::core::grid::Cell::default();
              }
            }
            _ => {}
          }
        }
      }
      'A' => { /*cursor up*/ }
      'B' => { /*cursor down*/ }
      'C' => { /*cursor forward*/ }
      'D' => { /*cursor backward*/ }
      'J' => {
        let mode = params.iter().next().map_or(0, |p| p[0]);
        match mode {
          2 => {
            for row in 0..self.grid.rows {
              for col in 0..self.grid.cols {
                self.grid.cells[row][col] = crate::core::grid::Cell::default();
              }
            }
            self.grid.cursor_x = 0;
            self.grid.cursor_y = 0;
          }
          _ => {}
        }
      }
      _ => {}
    }
  }

  fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
    if params.len() >= 2 {
      let cmd = params[0];

      if cmd == b"7" {
        let raw_url = String::from_utf8_lossy(params[1]).to_string();
        if let Some(path_with_host) = raw_url.split("file://").last() {
          if let Some((_host, path)) = path_with_host.split_once('/') {
            self.grid.pwd = format!("/{}", path);
          }
        }
      }
    }
  }
}
