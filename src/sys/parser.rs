use crate::core::grid::Grid;
use iced::Color;
use vte::{Params, Perform};

fn rgb8(r: u8, g: u8, b: u8) -> Color {
  Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

fn ansi_fg(index: u16, bright: bool) -> Color {
  let (v, d) = if bright { (255u8, 85u8) } else { (170u8, 0u8) };
  match index {
    0 => rgb8(d, d, d),
    1 => rgb8(v, d, d),
    2 => rgb8(d, v, d),
    3 => rgb8(v, v, d),
    4 => rgb8(d, d, v),
    5 => rgb8(v, d, v),
    6 => rgb8(d, v, v),
    _ => Color::WHITE,
  }
}

fn color_256(n: u8) -> Color {
  match n {
    0..=7 => ansi_fg(n as u16, false),
    8..=15 => ansi_fg(n as u16 - 8, true),
    16..=231 => {
      let idx = n as u32 - 16;
      let b = idx % 6;
      let g = (idx / 6) % 6;
      let r = idx / 36;
      let f = |x: u32| if x == 0 { 0u8 } else { (55 + x * 40) as u8 };
      rgb8(f(r), f(g), f(b))
    }
    232..=255 => {
      let v = (8 + (n as u32 - 232) * 10) as u8;
      rgb8(v, v, v)
    }
  }
}

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
      // tab — advance to next 8-column tab stop
      0x09 => {
        let next = (self.grid.cursor_x / 8 + 1) * 8;
        self.grid.cursor_x = next.min(self.grid.cols.saturating_sub(1));
      }
      _ => {}
    }
  }

  fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, command: char) {
    let param_value = params.iter().next().map(|p| p[0] as usize).unwrap_or(0);
    let n = if param_value == 0 { 1 } else { param_value };

    match command {
      'm' => {
        if params.iter().next().is_none() {
          self.grid.current_fg = Color::WHITE;
          return;
        }
        let mut iter = params.iter();
        while let Some(param) = iter.next() {
          let code = if param.is_empty() { 0 } else { param[0] };
          match code {
            0 | 39 => self.grid.current_fg = Color::WHITE,
            1..=9 | 21..=29 => {}
            30..=37 => self.grid.current_fg = ansi_fg(code - 30, false),
            38 => {
              if param.len() >= 3 && param[1] == 5 {
                self.grid.current_fg = color_256(param[2] as u8);
              } else if param.len() >= 5 && param[1] == 2 {
                self.grid.current_fg = rgb8(param[2] as u8, param[3] as u8, param[4] as u8);
              } else {
                let mode = iter.next().map_or(0, |p| if p.is_empty() { 0 } else { p[0] });
                match mode {
                  5 => {
                    let n = iter.next().map_or(0, |p| if p.is_empty() { 0 } else { p[0] }) as u8;
                    self.grid.current_fg = color_256(n);
                  }
                  2 => {
                    let r = iter.next().map_or(0, |p| if p.is_empty() { 0 } else { p[0] }) as u8;
                    let g = iter.next().map_or(0, |p| if p.is_empty() { 0 } else { p[0] }) as u8;
                    let b = iter.next().map_or(0, |p| if p.is_empty() { 0 } else { p[0] }) as u8;
                    self.grid.current_fg = rgb8(r, g, b);
                  }
                  _ => {}
                }
              }
            }
            40..=47 | 49 => {}
            48 => {
              if param.len() == 1 {
                let mode = iter.next().map_or(0, |p| if p.is_empty() { 0 } else { p[0] });
                match mode {
                  5 => { iter.next(); }
                  2 => { iter.next(); iter.next(); iter.next(); }
                  _ => {}
                }
              }
            }
            90..=97 => self.grid.current_fg = ansi_fg(code - 90, true),
            100..=107 => {}
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
