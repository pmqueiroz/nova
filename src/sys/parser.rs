use crate::core::grid::Grid;
use iced::Color;
use vte::{Params, Perform};

fn rgb8(r: u8, g: u8, b: u8) -> Color {
  Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

fn ansi_color(index: u16, bright: bool) -> Color {
  match (index, bright) {
    (0, false) => rgb8(0x1a, 0x1a, 0x1a),
    (1, false) => rgb8(0xff, 0x5f, 0x57),
    (2, false) => rgb8(0x3e, 0xcf, 0x8e),
    (3, false) => rgb8(0xf0, 0xc0, 0x40),
    (4, false) => rgb8(0x7b, 0x93, 0xfd),
    (5, false) => rgb8(0xc0, 0x84, 0xfc),
    (6, false) => rgb8(0x67, 0xe8, 0xf9),
    (7, false) => rgb8(0xe5, 0xe5, 0xe5),
    (0, true) => rgb8(0x55, 0x55, 0x55),
    (1, true) => rgb8(0xff, 0x6e, 0x6e),
    (2, true) => rgb8(0x5a, 0xf7, 0x8e),
    (3, true) => rgb8(0xf4, 0xf9, 0x9d),
    (4, true) => rgb8(0xca, 0xe8, 0xff),
    (5, true) => rgb8(0xd5, 0x7b, 0xff),
    (6, true) => rgb8(0x9a, 0xed, 0xfe),
    (7, true) => rgb8(0xff, 0xff, 0xff),
    _ => Color::WHITE,
  }
}

fn ansi_fg(index: u16, bright: bool) -> Color {
  ansi_color(index, bright)
}

fn ansi_bg(index: u16, bright: bool) -> Color {
  ansi_color(index, bright)
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
  pub bell_pending: bool,
}

impl<'a> Perform for AnsiExecutor<'a> {
  fn print(&mut self, c: char) {
    if self.grid.wrap_next {
      self.grid.cursor_x = 0;
      self.grid.newline();
      self.grid.wrap_next = false;
    }

    let x = self.grid.cursor_x;
    let y = self.grid.cursor_y;

    if x < self.grid.cols && y < self.grid.rows {
      self.grid.cells[y][x] = crate::core::grid::Cell {
        c,
        fg: self.grid.current_fg,
        bg: self.grid.current_bg,
        reverse: self.grid.reverse_video,
      };
      if x + 1 >= self.grid.cols {
        self.grid.wrap_next = true;
      } else {
        self.grid.cursor_x += 1;
      }
    }
  }

  fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
    match byte {
      b'7' => self.grid.saved_cursor = Some((self.grid.cursor_x, self.grid.cursor_y)),
      b'8' => {
        if let Some((x, y)) = self.grid.saved_cursor {
          self.grid.cursor_x = x;
          self.grid.cursor_y = y;
          self.grid.wrap_next = false;
        }
      }
      b'M' => self.grid.reverse_index(),
      b'D' => self.grid.newline(),
      b'E' => {
        self.grid.cursor_x = 0;
        self.grid.newline();
      }
      _ => {}
    }
  }

  fn execute(&mut self, byte: u8) {
    match byte {
      0x0A..=0x0C => {
        self.grid.cursor_x = 0;
        self.grid.newline();
      }
      // carriage return
      0x0D => {
        self.grid.cursor_x = 0;
        self.grid.wrap_next = false;
      }
      // backspace
      0x08 | 0x7F if self.grid.cursor_x > 0 => {
        self.grid.cursor_x -= 1;
      }
      // tab — advance to next 8-column tab stop
      0x09 => {
        let next = (self.grid.cursor_x / 8 + 1) * 8;
        self.grid.cursor_x = next.min(self.grid.cols.saturating_sub(1));
      }
      // BEL
      0x07 => {
        self.bell_pending = true;
      }
      _ => {}
    }
  }

  fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, command: char) {
    let param_value = params.iter().next().map(|p| p[0] as usize).unwrap_or(0);
    let n = if param_value == 0 { 1 } else { param_value };

    match command {
      'm' => {
        if params.iter().next().is_none() {
          self.grid.current_fg = None;
          self.grid.current_bg = None;
          return;
        }
        let mut iter = params.iter();
        while let Some(param) = iter.next() {
          let code = if param.is_empty() { 0 } else { param[0] };
          match code {
            0 => {
              self.grid.current_fg = None;
              self.grid.current_bg = None;
              self.grid.reverse_video = false;
            }
            7 => self.grid.reverse_video = true,
            27 => self.grid.reverse_video = false,
            39 => self.grid.current_fg = None,
            1..=6 | 8..=9 | 21..=26 | 28..=29 => {}
            30..=37 => self.grid.current_fg = Some(ansi_fg(code - 30, false)),
            38 => {
              if param.len() >= 3 && param[1] == 5 {
                self.grid.current_fg = Some(color_256(param[2] as u8));
              } else if param.len() >= 5 && param[1] == 2 {
                self.grid.current_fg = Some(rgb8(param[2] as u8, param[3] as u8, param[4] as u8));
              } else {
                let mode = iter
                  .next()
                  .map_or(0, |p| if p.is_empty() { 0 } else { p[0] });
                match mode {
                  5 => {
                    let n = iter
                      .next()
                      .map_or(0, |p| if p.is_empty() { 0 } else { p[0] })
                      as u8;
                    self.grid.current_fg = Some(color_256(n));
                  }
                  2 => {
                    let r = iter
                      .next()
                      .map_or(0, |p| if p.is_empty() { 0 } else { p[0] })
                      as u8;
                    let g = iter
                      .next()
                      .map_or(0, |p| if p.is_empty() { 0 } else { p[0] })
                      as u8;
                    let b = iter
                      .next()
                      .map_or(0, |p| if p.is_empty() { 0 } else { p[0] })
                      as u8;
                    self.grid.current_fg = Some(rgb8(r, g, b));
                  }
                  _ => {}
                }
              }
            }
            40..=47 => self.grid.current_bg = Some(ansi_bg(code - 40, false)),
            48 => {
              if param.len() >= 3 && param[1] == 5 {
                self.grid.current_bg = Some(color_256(param[2] as u8));
              } else if param.len() >= 5 && param[1] == 2 {
                self.grid.current_bg = Some(rgb8(param[2] as u8, param[3] as u8, param[4] as u8));
              } else {
                let mode = iter
                  .next()
                  .map_or(0, |p| if p.is_empty() { 0 } else { p[0] });
                match mode {
                  5 => {
                    let n = iter
                      .next()
                      .map_or(0, |p| if p.is_empty() { 0 } else { p[0] })
                      as u8;
                    self.grid.current_bg = Some(color_256(n));
                  }
                  2 => {
                    let r = iter
                      .next()
                      .map_or(0, |p| if p.is_empty() { 0 } else { p[0] })
                      as u8;
                    let g = iter
                      .next()
                      .map_or(0, |p| if p.is_empty() { 0 } else { p[0] })
                      as u8;
                    let b = iter
                      .next()
                      .map_or(0, |p| if p.is_empty() { 0 } else { p[0] })
                      as u8;
                    self.grid.current_bg = Some(rgb8(r, g, b));
                  }
                  _ => {}
                }
              }
            }
            49 => self.grid.current_bg = None,
            90..=97 => self.grid.current_fg = Some(ansi_fg(code - 90, true)),
            100..=107 => self.grid.current_bg = Some(ansi_bg(code - 100, true)),
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
        self.grid.wrap_next = false;
      }
      'B' => {
        let param_value = params.iter().next().map_or(1, |p| p[0] as usize);
        let n = if param_value == 0 { 1 } else { param_value };
        self.grid.cursor_y = (self.grid.cursor_y + n).min(self.grid.rows.saturating_sub(1));
        self.grid.wrap_next = false;
      }
      'C' => {
        self.grid.cursor_x = (self.grid.cursor_x + n).min(self.grid.cols.saturating_sub(1));
        self.grid.wrap_next = false;
      }
      'D' => {
        self.grid.cursor_x = self.grid.cursor_x.saturating_sub(n);
        self.grid.wrap_next = false;
      }
      'G' => {
        let col = if param_value == 0 { 0 } else { param_value - 1 };
        self.grid.cursor_x = col.min(self.grid.cols.saturating_sub(1));
        self.grid.wrap_next = false;
      }
      'H' | 'f' => {
        let mut iter = params.iter();

        let row_param = iter.next().map_or(1, |p| p[0] as usize);
        let row = if row_param == 0 { 1 } else { row_param };

        let col_param = iter.next().map_or(1, |p| p[0] as usize);
        let col = if col_param == 0 { 1 } else { col_param };

        self.grid.cursor_y = (row - 1).min(self.grid.rows.saturating_sub(1));
        self.grid.cursor_x = (col - 1).min(self.grid.cols.saturating_sub(1));
        self.grid.wrap_next = false;
      }
      'J' => {
        let mode = params.iter().next().map_or(0, |p| p[0]);
        let x = self.grid.cursor_x;
        let y = self.grid.cursor_y;
        match mode {
          0 => {
            for col in x..self.grid.cols {
              self.grid.cells[y][col] = crate::core::grid::Cell::default();
            }
            for row in (y + 1)..self.grid.rows {
              for col in 0..self.grid.cols {
                self.grid.cells[row][col] = crate::core::grid::Cell::default();
              }
            }
          }
          1 => {
            for row in 0..y {
              for col in 0..self.grid.cols {
                self.grid.cells[row][col] = crate::core::grid::Cell::default();
              }
            }
            for col in 0..=x {
              self.grid.cells[y][col] = crate::core::grid::Cell::default();
            }
          }
          2 | 3 => {
            for row in 0..self.grid.rows {
              for col in 0..self.grid.cols {
                self.grid.cells[row][col] = crate::core::grid::Cell::default();
              }
            }
          }
          _ => {}
        }
      }
      '@' => {
        let y = self.grid.cursor_y;
        let x = self.grid.cursor_x;
        if y < self.grid.rows {
          let row = &mut self.grid.cells[y];
          let count = n.min(self.grid.cols - x);
          for _ in 0..count {
            row.insert(x, crate::core::grid::Cell::default());
            row.pop();
          }
        }
      }
      'P' => {
        let y = self.grid.cursor_y;
        let x = self.grid.cursor_x;
        if y < self.grid.rows {
          let row = &mut self.grid.cells[y];
          let count = n.min(self.grid.cols - x);
          for _ in 0..count {
            row.remove(x);
            row.push(crate::core::grid::Cell::default());
          }
        }
      }
      'X' => {
        let y = self.grid.cursor_y;
        let x = self.grid.cursor_x;
        if y < self.grid.rows {
          for col in x..(x + n).min(self.grid.cols) {
            self.grid.cells[y][col] = crate::core::grid::Cell::default();
          }
        }
      }
      'L' => {
        let y = self.grid.cursor_y;
        let bottom = self.grid.scroll_bottom;
        let count = n.min(bottom.saturating_sub(y) + 1);
        for _ in 0..count {
          self
            .grid
            .cells
            .insert(y, vec![crate::core::grid::Cell::default(); self.grid.cols]);
          if self.grid.cells.len() > self.grid.rows {
            self.grid.cells.remove(bottom + 1);
          }
        }
      }
      'M' => {
        let y = self.grid.cursor_y;
        let bottom = self.grid.scroll_bottom;
        let count = n.min(bottom.saturating_sub(y) + 1);
        for _ in 0..count {
          if y < self.grid.cells.len() {
            self.grid.cells.remove(y);
            self.grid.cells.insert(
              bottom,
              vec![crate::core::grid::Cell::default(); self.grid.cols],
            );
          }
        }
      }
      'r' if intermediates.is_empty() => {
        let mut iter = params.iter();
        let top = iter
          .next()
          .map_or(1, |p| if p[0] == 0 { 1 } else { p[0] as usize });
        let bottom = iter.next().map_or(self.grid.rows, |p| {
          if p[0] == 0 {
            self.grid.rows
          } else {
            p[0] as usize
          }
        });
        self.grid.scroll_top = (top - 1).min(self.grid.rows.saturating_sub(1));
        self.grid.scroll_bottom = (bottom - 1).min(self.grid.rows.saturating_sub(1));
        self.grid.cursor_x = 0;
        self.grid.cursor_y = 0;
        self.grid.wrap_next = false;
      }
      's' if intermediates.is_empty() => {
        self.grid.saved_cursor = Some((self.grid.cursor_x, self.grid.cursor_y));
      }
      'u' if intermediates.is_empty() => {
        if let Some((x, y)) = self.grid.saved_cursor {
          self.grid.cursor_x = x;
          self.grid.cursor_y = y;
          self.grid.wrap_next = false;
        }
      }
      'h' | 'l' if intermediates.contains(&b'?') => {
        let mode = params.iter().next().map_or(0, |p| p[0]);
        match mode {
          47 | 1047 | 1049 => {
            if command == 'h' {
              self.grid.enter_alt_screen();
            } else {
              self.grid.leave_alt_screen();
            }
          }
          _ => {}
        }
      }
      'n' => {
        let mut iter = params.iter();
        if let Some(param) = iter.next()
          && param.contains(&6)
        {
          let row = self.grid.cursor_y + 1;
          let col = self.grid.cursor_x + 1;

          let response = format!("\x1b[{};{}R", row, col);

          self.grid.output_queue.push(response.into_bytes());
        }
      }
      _ => {}
    }
  }

  fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
    if params.len() >= 2 && params[0] == b"7" {
      let raw_url = String::from_utf8_lossy(params[1]).to_string();
      if let Some(after_scheme) = raw_url.strip_prefix("file://")
        && let Some((_, path)) = after_scheme.split_once('/')
      {
        #[cfg(target_os = "windows")]
        let pwd = path.replace('/', "\\");

        #[cfg(not(target_os = "windows"))]
        let pwd = format!("/{}", path);

        self.grid.pwd = pwd;
      }
    }
  }
}
