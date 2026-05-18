use crate::cli;
use crate::core::grid::ControlCommand;
use crate::core::grid::Grid;
use base64::Engine;
use iced::Color;
use std::sync::Arc;
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
    use unicode_width::UnicodeWidthChar;

    let char_width = c.width().unwrap_or(1) as u8;

    if char_width == 0 {
      let x = self.grid.cursor_x;
      let y = self.grid.cursor_y;
      let (bx, by) = if self.grid.wrap_next {
        let w = self.grid.cells[y * self.grid.cols + x].width;
        if w == 0 && x > 0 { (x - 1, y) } else { (x, y) }
      } else if x > 0 {
        let px = x - 1;
        let pw = self.grid.cells[y * self.grid.cols + px].width;
        if pw == 0 && px > 0 {
          (px - 1, y)
        } else {
          (px, y)
        }
      } else if y > 0 {
        let lx = self.grid.cols - 1;
        let pw = self.grid.cells[(y - 1) * self.grid.cols + lx].width;
        if pw == 0 && lx > 0 {
          (lx - 1, y - 1)
        } else {
          (lx, y - 1)
        }
      } else {
        return;
      };
      let base = self.grid.cell_mut(by, bx);
      let mut s = base.c.to_string();
      s.push(c);
      base.c = s.into_boxed_str();
      return;
    }

    {
      let x = self.grid.cursor_x;
      let y = self.grid.cursor_y;
      let (bx, by) = if self.grid.wrap_next {
        let w = self.grid.cells[y * self.grid.cols + x].width;
        if w == 0 && x > 0 { (x - 1, y) } else { (x, y) }
      } else if x > 0 {
        let px = x - 1;
        let pw = self.grid.cells[y * self.grid.cols + px].width;
        if pw == 0 && px > 0 {
          (px - 1, y)
        } else {
          (px, y)
        }
      } else {
        (usize::MAX, 0)
      };
      if bx != usize::MAX
        && self.grid.cells[by * self.grid.cols + bx]
          .c
          .ends_with('\u{200D}')
      {
        let base = self.grid.cell_mut(by, bx);
        let mut s = base.c.to_string();
        s.push(c);
        base.c = s.into_boxed_str();
        return;
      }
    }

    if self.grid.wrap_next {
      self.grid.cursor_x = 0;
      self.grid.newline();
      self.grid.wrap_next = false;
      if self.grid.cursor_y < self.grid.rows {
        self.grid.row_continuation[self.grid.cursor_y] = true;
      }
    }

    if char_width == 2 && self.grid.cursor_x + 1 >= self.grid.cols {
      let x = self.grid.cursor_x;
      let y = self.grid.cursor_y;
      if x < self.grid.cols && y < self.grid.rows {
        *self.grid.cell_mut(y, x) = crate::core::grid::Cell {
          c: Box::from(" "),
          width: 1,
          fg: self.grid.current_fg,
          bg: self.grid.current_bg,
          attrs: self.grid.current_attrs,
          uri: self.grid.current_uri.clone(),
        };
      }
      self.grid.cursor_x = 0;
      self.grid.newline();
      if self.grid.cursor_y < self.grid.rows {
        self.grid.row_continuation[self.grid.cursor_y] = true;
      }
    }

    let x = self.grid.cursor_x;
    let y = self.grid.cursor_y;

    if x < self.grid.cols && y < self.grid.rows {
      *self.grid.cell_mut(y, x) = crate::core::grid::Cell {
        c: c.to_string().into_boxed_str(),
        width: char_width,
        fg: self.grid.current_fg,
        bg: self.grid.current_bg,
        attrs: self.grid.current_attrs,
        uri: self.grid.current_uri.clone(),
      };

      if char_width == 2 && x + 1 < self.grid.cols {
        *self.grid.cell_mut(y, x + 1) = crate::core::grid::Cell {
          c: Box::from(""),
          width: 0,
          fg: self.grid.current_fg,
          bg: self.grid.current_bg,
          attrs: self.grid.current_attrs,
          uri: self.grid.current_uri.clone(),
        };
      }

      let next_x = x + char_width as usize;
      if next_x >= self.grid.cols {
        self.grid.wrap_next = true;
      } else {
        self.grid.cursor_x = next_x;
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
          self.grid.current_attrs = crate::core::grid::CellAttrs::empty();
          return;
        }
        let mut iter = params.iter();
        while let Some(param) = iter.next() {
          let code = if param.is_empty() { 0 } else { param[0] };
          match code {
            0 => {
              self.grid.current_fg = None;
              self.grid.current_bg = None;
              self.grid.current_attrs = crate::core::grid::CellAttrs::empty();
            }
            1 => self
              .grid
              .current_attrs
              .insert(crate::core::grid::CellAttrs::BOLD),
            2 => self
              .grid
              .current_attrs
              .insert(crate::core::grid::CellAttrs::DIM),
            3 => self
              .grid
              .current_attrs
              .insert(crate::core::grid::CellAttrs::ITALIC),
            4 => self
              .grid
              .current_attrs
              .insert(crate::core::grid::CellAttrs::UNDERLINE),
            5 | 6 => self
              .grid
              .current_attrs
              .insert(crate::core::grid::CellAttrs::BLINK),
            7 => self
              .grid
              .current_attrs
              .insert(crate::core::grid::CellAttrs::REVERSE),
            8 => {} // HIDDEN
            9 => self
              .grid
              .current_attrs
              .insert(crate::core::grid::CellAttrs::STRIKETHROUGH),
            21 | 22 => {
              self
                .grid
                .current_attrs
                .remove(crate::core::grid::CellAttrs::BOLD);
              self
                .grid
                .current_attrs
                .remove(crate::core::grid::CellAttrs::DIM);
            }
            23 => self
              .grid
              .current_attrs
              .remove(crate::core::grid::CellAttrs::ITALIC),
            24 => self
              .grid
              .current_attrs
              .remove(crate::core::grid::CellAttrs::UNDERLINE),
            25 => self
              .grid
              .current_attrs
              .remove(crate::core::grid::CellAttrs::BLINK),
            27 => self
              .grid
              .current_attrs
              .remove(crate::core::grid::CellAttrs::REVERSE),
            28 => {} // REVEAL
            29 => self
              .grid
              .current_attrs
              .remove(crate::core::grid::CellAttrs::STRIKETHROUGH),
            39 => self.grid.current_fg = None,
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
                *self.grid.cell_mut(y, col) = crate::core::grid::Cell::default();
              }
            }
            1 => {
              for col in 0..=x {
                *self.grid.cell_mut(y, col) = crate::core::grid::Cell::default();
              }
            }
            2 => {
              for col in 0..self.grid.cols {
                *self.grid.cell_mut(y, col) = crate::core::grid::Cell::default();
              }
            }
            _ => {}
          }
        }
      }
      'A' => {
        let param_value = params.iter().next().map_or(1, |p| p[0] as usize);
        let n = if param_value == 0 { 1 } else { param_value };
        let top = if self.grid.cursor_y >= self.grid.scroll_top {
          self.grid.scroll_top
        } else {
          0
        };
        self.grid.cursor_y = self.grid.cursor_y.saturating_sub(n).max(top);
        self.grid.wrap_next = false;
      }
      'B' => {
        let param_value = params.iter().next().map_or(1, |p| p[0] as usize);
        let n = if param_value == 0 { 1 } else { param_value };
        let bottom = if self.grid.cursor_y <= self.grid.scroll_bottom {
          self.grid.scroll_bottom
        } else {
          self.grid.rows.saturating_sub(1)
        };
        self.grid.cursor_y = (self.grid.cursor_y + n).min(bottom);
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
              *self.grid.cell_mut(y, col) = crate::core::grid::Cell::default();
            }
            for row in (y + 1)..self.grid.rows {
              for col in 0..self.grid.cols {
                *self.grid.cell_mut(row, col) = crate::core::grid::Cell::default();
              }
            }
          }
          1 => {
            for row in 0..y {
              for col in 0..self.grid.cols {
                *self.grid.cell_mut(row, col) = crate::core::grid::Cell::default();
              }
            }
            for col in 0..=x {
              *self.grid.cell_mut(y, col) = crate::core::grid::Cell::default();
            }
          }
          2 | 3 => {
            for row in 0..self.grid.rows {
              for col in 0..self.grid.cols {
                *self.grid.cell_mut(row, col) = crate::core::grid::Cell::default();
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
          let count = n.min(self.grid.cols - x);
          let start_idx = y * self.grid.cols + x;
          let end_idx = (y + 1) * self.grid.cols;

          self.grid.cells[start_idx..end_idx].rotate_right(count);
          for col in x..(x + count) {
            *self.grid.cell_mut(y, col) = crate::core::grid::Cell::default();
          }
        }
      }
      'P' => {
        let y = self.grid.cursor_y;
        let x = self.grid.cursor_x;
        if y < self.grid.rows {
          let count = n.min(self.grid.cols - x);
          let start_idx = y * self.grid.cols + x;
          let end_idx = (y + 1) * self.grid.cols;

          self.grid.cells[start_idx..end_idx].rotate_left(count);
          for col in (self.grid.cols - count)..self.grid.cols {
            *self.grid.cell_mut(y, col) = crate::core::grid::Cell::default();
          }
        }
      }
      'X' => {
        let y = self.grid.cursor_y;
        let x = self.grid.cursor_x;
        if y < self.grid.rows {
          for col in x..(x + n).min(self.grid.cols) {
            *self.grid.cell_mut(y, col) = crate::core::grid::Cell::default();
          }
        }
      }
      'L' => {
        let y = self.grid.cursor_y;
        let bottom = self.grid.scroll_bottom;
        if y <= bottom {
          let count = n.min(bottom.saturating_sub(y) + 1);
          let old_top = self.grid.scroll_top;
          self.grid.scroll_top = y;
          self.grid.scroll_down(count);
          self.grid.scroll_top = old_top;
        }
      }
      'M' => {
        let y = self.grid.cursor_y;
        let bottom = self.grid.scroll_bottom;
        if y <= bottom {
          let count = n.min(bottom.saturating_sub(y) + 1);
          let old_top = self.grid.scroll_top;
          self.grid.scroll_top = y;
          self.grid.scroll_up(count);
          self.grid.scroll_top = old_top;
        }
      }
      'S' => {
        self.grid.scroll_up(n);
      }
      'T' => {
        self.grid.scroll_down(n);
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
          1000 => {
            self.grid.mouse_mode = if command == 'h' {
              crate::core::grid::MouseMode::Normal
            } else {
              crate::core::grid::MouseMode::None
            };
          }
          1002 => {
            self.grid.mouse_mode = if command == 'h' {
              crate::core::grid::MouseMode::Button
            } else {
              crate::core::grid::MouseMode::None
            };
          }
          1003 => {
            self.grid.mouse_mode = if command == 'h' {
              crate::core::grid::MouseMode::AnyEvent
            } else {
              crate::core::grid::MouseMode::None
            };
          }
          1006 => {
            self.grid.mouse_sgr = command == 'h';
          }
          2004 => {
            self.grid.bracketed_paste = command == 'h';
          }
          25 => {
            self.grid.cursor_visible = command == 'h';
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
    if params.is_empty() {
      return;
    }
    match params[0] {
      cli::constants::PRIVATE_NOVA_OSC_CODE_BYTES
        if params.len() >= 2 && params[1] == b"ask_ai" =>
      {
        let preset = if params.len() >= 3 && !params[2].is_empty() {
          match base64::engine::general_purpose::STANDARD.decode(params[2]) {
            Ok(bytes) => String::from_utf8(bytes).ok().map(Arc::<str>::from),
            Err(_) => None,
          }
        } else {
          None
        };
        self
          .grid
          .control_queue
          .push(ControlCommand::OpenAskAi { preset });
      }
      cli::constants::PRIVATE_NOVA_OSC_CODE_BYTES
        if params.len() >= 2 && params[1] == b"command_failure" =>
      {
        let code = params
          .get(2)
          .and_then(|c| std::str::from_utf8(c).ok()?.parse().ok())
          .unwrap_or(1);
        self
          .grid
          .control_queue
          .push(ControlCommand::CommandFailure(code));
      }
      cli::constants::PRIVATE_NOVA_OSC_CODE_BYTES
        if params.len() >= 2 && params[1] == b"command_complete" =>
      {
        let code = params
          .get(2)
          .and_then(|c| std::str::from_utf8(c).ok()?.parse().ok())
          .unwrap_or(0);
        self
          .grid
          .control_queue
          .push(ControlCommand::CommandComplete(code));
      }
      cli::constants::PRIVATE_NOVA_OSC_CODE_BYTES
        if params.len() >= 2 && params[1] == b"explain_ai" =>
      {
        let preset = if params.len() >= 3 && !params[2].is_empty() {
          match base64::engine::general_purpose::STANDARD.decode(params[2]) {
            Ok(bytes) => String::from_utf8(bytes).ok().map(Arc::<str>::from),
            Err(_) => None,
          }
        } else {
          None
        };
        self
          .grid
          .control_queue
          .push(ControlCommand::OpenExplainAi { preset });
      }
      b"7" if params.len() >= 2 => {
        let raw_url = String::from_utf8_lossy(params[1]).to_string();
        if let Some(after_scheme) = raw_url.strip_prefix("file://")
          && let Some((_, path)) = after_scheme.split_once('/')
        {
          #[cfg(target_os = "windows")]
          let pwd = path.replace('/', "\\");

          #[cfg(not(target_os = "windows"))]
          let pwd = format!("/{}", path);

          self.grid.pwd = pwd;
        } else if let Some(host) = raw_url.strip_prefix("ssh://")
          && !host.is_empty()
        {
          self.grid.pwd = format!("ssh:{}", host);
        }
      }
      b"8" if params.len() >= 3 => {
        let uri = String::from_utf8_lossy(params[2]);
        self.grid.current_uri = if uri.is_empty() {
          None
        } else {
          Some(Arc::from(uri.as_ref()))
        };
      }
      b"52" if params.len() >= 3 => {
        let data = params[2];
        if data == b"?" {
          self
            .grid
            .control_queue
            .push(crate::core::grid::ControlCommand::RequestClipboard);
        } else if !data.is_empty()
          && let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data)
          && let Ok(text) = String::from_utf8(bytes)
        {
          self
            .grid
            .control_queue
            .push(crate::core::grid::ControlCommand::SetClipboard(text));
        }
      }
      _ => {}
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::grid::Grid;
  use vte::Parser;

  fn make_grid(cols: usize, rows: usize) -> Grid {
    Grid::new(cols, rows)
  }

  fn print_str(grid: &mut Grid, s: &str) {
    let mut parser = Parser::new();
    let mut exec = AnsiExecutor {
      grid,
      bell_pending: false,
    };
    parser.advance(&mut exec, s.as_bytes());
  }

  #[test]
  fn ascii_char_width_and_cursor() {
    let mut grid = make_grid(10, 5);
    print_str(&mut grid, "A");
    assert_eq!(grid.cells[0].c.as_ref(), "A");
    assert_eq!(grid.cells[0].width, 1);
    assert_eq!(grid.cursor_x, 1);
  }

  #[test]
  fn cjk_wide_char_occupies_two_columns() {
    let mut grid = make_grid(10, 5);
    print_str(&mut grid, "中");
    assert_eq!(grid.cells[0].c.as_ref(), "中");
    assert_eq!(grid.cells[0].width, 2);
    assert_eq!(grid.cells[1].width, 0);
    assert_eq!(grid.cells[1].c.as_ref(), "");
    assert_eq!(grid.cursor_x, 2);
  }

  #[test]
  fn combining_accent_appended_to_base() {
    let mut grid = make_grid(10, 5);
    print_str(&mut grid, "e\u{0301}");
    assert_eq!(grid.cells[0].c.as_ref(), "e\u{0301}");
    assert_eq!(grid.cells[0].width, 1);
    assert_eq!(grid.cursor_x, 1);
  }

  #[test]
  fn zwj_emoji_sequence_in_single_cell() {
    let mut grid = make_grid(20, 5);
    print_str(&mut grid, "\u{1F468}\u{200D}\u{1F469}");
    assert_eq!(grid.cells[0].c.as_ref(), "\u{1F468}\u{200D}\u{1F469}");
    assert_eq!(grid.cells[0].width, 2);
    assert_eq!(grid.cells[1].width, 0);
    assert_eq!(grid.cursor_x, 2);
  }

  #[test]
  fn wide_char_at_line_end_wraps() {
    let mut grid = make_grid(3, 5);
    print_str(&mut grid, "ab中");
    assert_eq!(grid.cells[0].c.as_ref(), "a");
    assert_eq!(grid.cells[1].c.as_ref(), "b");
    let row1_start = 3;
    assert_eq!(grid.cells[row1_start].c.as_ref(), "中");
    assert_eq!(grid.cells[row1_start].width, 2);
    assert_eq!(grid.cursor_x, 2);
    assert_eq!(grid.cursor_y, 1);
  }

  #[test]
  fn multiple_wide_chars_cursor_tracking() {
    let mut grid = make_grid(10, 5);
    print_str(&mut grid, "你好");
    assert_eq!(grid.cells[0].c.as_ref(), "你");
    assert_eq!(grid.cells[0].width, 2);
    assert_eq!(grid.cells[2].c.as_ref(), "好");
    assert_eq!(grid.cells[2].width, 2);
    assert_eq!(grid.cursor_x, 4);
  }

  #[test]
  fn family_emoji_zwj_sequence() {
    let mut grid = make_grid(20, 5);
    let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}";
    print_str(&mut grid, family);
    assert_eq!(grid.cells[0].c.as_ref(), family);
    assert_eq!(grid.cells[0].width, 2);
    assert_eq!(grid.cursor_x, 2);
  }
}
