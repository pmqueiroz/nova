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
      // line feed
      0x0A | 0x0B | 0x0C => self.grid.newline(),
      // carriage return
      0x0D => self.grid.cursor_x = 0,
      // backspace
      0x08 | 0x7F => {
        if self.grid.cursor_x > 0 {
          self.grid.cursor_x -= 1;
        }
      }
      _ => {}
    }
  }

  fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, command: char) {
    let param_value = params.iter().next().map(|p| p[0] as usize).unwrap_or(0);
    let n = if param_value == 0 { 1 } else { param_value };

    match command {
      'm' => {
        for param in params.iter() {
          let code = param[0];
          match code {
            0 => self.grid.current_fg = Color::WHITE,
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
      'A' => {
        let param_value = params.iter().next().map_or(1, |p| p[0] as usize);
        let n = if param_value == 0 { 1 } else { param_value };

        self.grid.cursor_y = self.grid.cursor_y.saturating_sub(n);
      }
      'B' => {
        let param_value = params.iter().next().map_or(1, |p| p[0] as usize);
        let n = if param_value == 0 { 1 } else { param_value };

        self.grid.cursor_y = (self.grid.cursor_y + n).min(self.grid.rows.saturating_sub(1));
      }
      'C' => {
        self.grid.cursor_x = (self.grid.cursor_x + n).min(self.grid.cols.saturating_sub(1));
      }
      'D' => {
        self.grid.cursor_x = self.grid.cursor_x.saturating_sub(n);
      }
      'G' => {
        let col = if param_value == 0 { 0 } else { param_value - 1 };
        self.grid.cursor_x = col.min(self.grid.cols.saturating_sub(1));
      }
      'H' | 'f' => {
        let mut iter = params.iter();

        let row_param = iter.next().map_or(1, |p| p[0] as usize);
        let row = if row_param == 0 { 1 } else { row_param };

        let col_param = iter.next().map_or(1, |p| p[0] as usize);
        let col = if col_param == 0 { 1 } else { col_param };

        self.grid.cursor_y = (row - 1).min(self.grid.rows.saturating_sub(1));
        self.grid.cursor_x = (col - 1).min(self.grid.cols.saturating_sub(1));
      }
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
      'n' => {
        println!("[ANSI] received cursor position query (CPR)");
        let mut iter = params.iter();
        if let Some(param) = iter.next() {
          if param.contains(&6) {
            let row = self.grid.cursor_y + 1;
            let col = self.grid.cursor_x + 1;

            let response = format!("\x1b[{};{}R", row, col);

            self.grid.output_queue.push(response.into_bytes());
          }
        }
      }
      _ => {}
    }
  }

  fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
    if params.len() >= 2 && params[0] == b"7" {
      let raw_url = String::from_utf8_lossy(params[1]).to_string();
      if let Some(after_scheme) = raw_url.strip_prefix("file://") {
        if let Some((_, path)) = after_scheme.split_once('/') {
          #[cfg(target_os = "windows")]
          let pwd = path.replace('/', "\\");

          #[cfg(not(target_os = "windows"))]
          let pwd = format!("/{}", path);

          self.grid.pwd = pwd;
        }
      }
    }
  }
}
