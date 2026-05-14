use iced::{Point, mouse};

use crate::sys::pty::PtyCommand;

use super::super::helpers::{
  extract_selection, find_word_boundaries, get_display_row, pixel_to_cell, resize_direction,
  resolve_hovered_url,
};
use super::super::nova::Nova;

impl Nova {
  pub(super) fn update_hovered_url(&mut self) {
    if !self.ctrl_held {
      self.hovered_url = None;
      self.hovered_link_span = None;
      return;
    }
    let font_size = self.settings.theme.font.size;
    let Some((col, row)) = pixel_to_cell(self.cursor_position, font_size) else {
      self.hovered_url = None;
      self.hovered_link_span = None;
      return;
    };
    let Some(tab) = self.tabs.get(self.active_index) else {
      self.hovered_url = None;
      self.hovered_link_span = None;
      return;
    };
    let (result_url, result_span) = resolve_hovered_url(&tab.grid, tab.scroll_offset, col, row);
    self.hovered_url = result_url;
    self.hovered_link_span = result_span;
  }

  pub(super) fn handle_cursor_moved(&mut self, position: Point) {
    self.cursor_position = position;
    let font_size = self.settings.theme.font.size;

    let mut bypass_selection = false;
    if let Some(tab) = self.tabs.get(self.active_index)
      && tab.grid.mouse_mode != crate::core::grid::MouseMode::None
      && !self.shift_held
    {
      bypass_selection = true;
      if (tab.grid.mouse_mode == crate::core::grid::MouseMode::AnyEvent
        || (tab.grid.mouse_mode == crate::core::grid::MouseMode::Button
          && self.last_mouse_button.is_some()))
        && let Some(cell) = pixel_to_cell(position, font_size)
      {
        if let Some(button) = self.last_mouse_button {
          self.send_mouse_event(tab, cell.0, cell.1, Some(button), true, true);
        } else {
          self.send_mouse_event(tab, cell.0, cell.1, None, false, true);
        }
      }
    }

    if self.is_selecting && !bypass_selection {
      let end = pixel_to_cell(position, font_size);
      self.selection_end = end;
      if self.click_count >= 2
        && let (Some((end_col, end_row)), Some(active_tab)) =
          (end, self.tabs.get(self.active_index))
      {
        match self.click_count {
          2 => {
            if let Some(row_cells) =
              get_display_row(&active_tab.grid, active_tab.scroll_offset, end_row)
            {
              let (_, we) = find_word_boundaries(row_cells, end_col);
              self.selection_end = Some((we, end_row));
            }
          }
          3 => {
            self.selection_end = Some((active_tab.grid.cols.saturating_sub(1), end_row));
          }
          _ => {}
        }
      }
    }
    self.update_hovered_url();
  }

  pub(super) fn handle_mouse_pressed(
    &mut self,
    button: mouse::Button,
  ) -> iced::Task<super::super::message::Message> {
    self.last_mouse_button = Some(button);
    if self.settings_open || self.command_palette_open || self.ai_overlay_open || self.ai_loading {
      return iced::Task::none();
    }
    if self.ctrl_held
      && let Some(url) = self.hovered_url.clone()
    {
      let _ = open::that_detached(&url);
      return iced::Task::none();
    }
    if let Some(window_id) = self.window_id
      && let Some(direction) = resize_direction(self.cursor_position, self.window_size)
    {
      return iced::window::drag_resize(window_id, direction);
    }

    let font_size = self.settings.theme.font.size;
    let cell = pixel_to_cell(self.cursor_position, font_size);

    if let Some(tab) = self.tabs.get(self.active_index)
      && tab.grid.mouse_mode != crate::core::grid::MouseMode::None
      && !self.shift_held
    {
      if let Some((col, row)) = cell {
        self.send_mouse_event(tab, col, row, Some(button), true, false);
      }
      self.click_count = 0;
      return iced::Task::none();
    }

    if button == mouse::Button::Left {
      let now = std::time::Instant::now();
      let threshold = std::time::Duration::from_millis(500);
      if cell.is_some()
        && cell == self.last_click_cell
        && now.duration_since(self.last_click_time) < threshold
      {
        self.click_count = (self.click_count + 1).min(3);
      } else {
        self.click_count = 1;
      }
      self.last_click_time = now;
      self.last_click_cell = cell;

      self.selection_start = cell;
      self.selection_end = cell;
      self.is_selecting = cell.is_some();

      if let Some((col, row)) = cell
        && self.click_count >= 2
        && let Some(active_tab) = self.tabs.get(self.active_index)
      {
        match self.click_count {
          2 => {
            if let Some(row_cells) =
              get_display_row(&active_tab.grid, active_tab.scroll_offset, row)
            {
              let (ws, we) = find_word_boundaries(row_cells, col);
              self.selection_start = Some((ws, row));
              self.selection_end = Some((we, row));
            }
          }
          3 => {
            self.selection_start = Some((0, row));
            self.selection_end = Some((active_tab.grid.cols.saturating_sub(1), row));
          }
          _ => {}
        }
      }
    }
    iced::Task::none()
  }

  pub(super) fn handle_mouse_released(
    &mut self,
    button: mouse::Button,
  ) -> iced::Task<super::super::message::Message> {
    if self.last_mouse_button == Some(button) {
      self.last_mouse_button = None;
    }

    self.is_selecting = false;
    if self.settings_open || self.command_palette_open || self.ai_overlay_open || self.ai_loading {
      return iced::Task::none();
    }

    if let Some(tab) = self.tabs.get(self.active_index)
      && tab.grid.mouse_mode != crate::core::grid::MouseMode::None
      && !self.shift_held
    {
      let font_size = self.settings.theme.font.size;
      if let Some((col, row)) = pixel_to_cell(self.cursor_position, font_size) {
        self.send_mouse_event(tab, col, row, Some(button), false, false);
      }
      return iced::Task::none();
    }

    if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
      if start == end {
        self.selection_start = None;
        self.selection_end = None;
      } else if let Some(active_tab) = self.tabs.get(self.active_index) {
        let text = extract_selection(&active_tab.grid, active_tab.scroll_offset, start, end);
        if !text.is_empty() {
          return iced::clipboard::write(text);
        }
      }
    }
    iced::Task::none()
  }

  pub(super) fn handle_scroll(&mut self, delta: f32) {
    if self.settings_open {
      return;
    }
    let font_size = self.settings.theme.font.size;
    if let Some(tab) = self.tabs.get_mut(self.active_index) {
      if tab.grid.mouse_mode != crate::core::grid::MouseMode::None && !self.shift_held {
        if let Some((col, row)) = pixel_to_cell(self.cursor_position, font_size) {
          let is_down = delta < 0.0;
          let btn = if is_down { 65 } else { 64 };
          let mut cb = btn;
          if self.shift_held {
            cb |= 4;
          }
          if self.alt_held {
            cb |= 8;
          }
          if self.ctrl_held {
            cb |= 16;
          }

          if let Some(tx) = &tab.pty_tx {
            if tab.grid.mouse_sgr {
              let cmd = format!("\x1b[<{};{};{}M", cb, col + 1, row + 1);
              let _ = tx.try_send(PtyCommand::Input(cmd.into_bytes()));
            } else {
              let cb_byte = (cb + 32).min(255) as u8;
              let cx = (col + 1 + 32).min(255) as u8;
              let cy = (row + 1 + 32).min(255) as u8;
              let cmd = vec![b'\x1b', b'[', b'M', cb_byte, cx, cy];
              let _ = tx.try_send(PtyCommand::Input(cmd));
            }
          }
        }
        return;
      }

      let rows = (delta.abs() * 3.0).round() as usize;
      let old_offset = tab.scroll_offset;
      if delta > 0.0 {
        let new_offset = tab.scroll_offset.saturating_add(rows);
        tab.scroll_offset = new_offset.min(tab.grid.scrollback.len());
      } else {
        tab.scroll_offset = tab.scroll_offset.saturating_sub(rows);
      }
      let scroll_delta = tab.scroll_offset as isize - old_offset as isize;
      if scroll_delta != 0 {
        if let Some((col, row)) = self.selection_start {
          let new_row = (row as isize + scroll_delta).max(0) as usize;
          self.selection_start = Some((col, new_row));
        }
        if let Some((col, row)) = self.selection_end {
          let new_row = (row as isize + scroll_delta).max(0) as usize;
          self.selection_end = Some((col, new_row));
        }
      }
    }
  }

  fn send_mouse_event(
    &self,
    tab: &crate::ui::tab::Tab,
    col: usize,
    row: usize,
    button: Option<mouse::Button>,
    pressed: bool,
    motion: bool,
  ) {
    let Some(tx) = &tab.pty_tx else { return };

    let mut cb = match button {
      Some(mouse::Button::Left) => 0,
      Some(mouse::Button::Middle) => 1,
      Some(mouse::Button::Right) => 2,
      None => 3,
      _ => return,
    };

    if motion {
      cb += 32;
    }

    if self.shift_held {
      cb |= 4;
    }
    if self.alt_held {
      cb |= 8;
    }
    if self.ctrl_held {
      cb |= 16;
    }

    if tab.grid.mouse_sgr {
      let state = if pressed { 'M' } else { 'm' };
      let cmd = format!("\x1b[<{};{};{}{}", cb, col + 1, row + 1, state);
      let _ = tx.try_send(PtyCommand::Input(cmd.into_bytes()));
    } else {
      let cb_byte = (cb + 32).min(255) as u8;
      let cx = (col + 1 + 32).min(255) as u8;
      let cy = (row + 1 + 32).min(255) as u8;
      let cmd = vec![b'\x1b', b'[', b'M', cb_byte, cx, cy];
      let _ = tx.try_send(PtyCommand::Input(cmd));
    }
  }
}
