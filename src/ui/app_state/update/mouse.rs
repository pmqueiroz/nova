use iced::{Point, mouse};

use crate::core::grid::{Grid, MouseMode};
use crate::sys::pty::PtyCommand;

use super::super::helpers::{
  SPLIT_DIVIDER_WIDTH, alternate_scroll_keys, extract_selection, find_word_boundaries,
  get_display_row, pixel_to_cell, raw_pixel_to_cell, resize_direction, resolve_hovered_url,
  split_drop_side, split_widths, wheel_report,
};
use super::super::nova::Nova;

#[derive(Clone, Copy)]
pub(super) struct Pointer {
  mouse_mode: MouseMode,
  cell: Option<(usize, usize)>,
}

impl Pointer {
  fn reports(&self, shift_held: bool) -> bool {
    self.mouse_mode != MouseMode::None && !shift_held
  }
}

impl Nova {
  pub(super) fn selection_grid(&self) -> Option<(&Grid, usize)> {
    let tab = self.tabs.get(self.active_index)?;
    let pane = tab.active_pane();
    match pane.raw_grid() {
      Some(grid) => Some((grid, pane.raw_scroll())),
      None => Some((&tab.grid, tab.scroll_offset)),
    }
  }

  fn pointer(&self) -> Option<Pointer> {
    let tab = self.tabs.get(self.active_index)?;
    let split = tab.active_pane_is_split;
    let Some(grid) = tab.pane(split).raw_grid() else {
      return Some(Pointer {
        mouse_mode: tab.grid.mouse_mode,
        cell: pixel_to_cell(self.cursor_position, self.settings.theme.font.size),
      });
    };
    let pane_x = if split {
      split_widths(self.window_size.width, tab.split_ratio).0 + SPLIT_DIVIDER_WIDTH
    } else {
      0.0
    };
    let pos = self.cursor_position;
    let inside = pos.x >= pane_x && pos.y >= crate::ui::components::TITLE_BAR_HEIGHT;
    let compact = crate::ui::theme::tokens::block::is_compact(self.window_size.width);
    let (col, row) = raw_pixel_to_cell(pos, pane_x, compact, self.settings.theme.font.size);
    Some(Pointer {
      mouse_mode: grid.mouse_mode,
      cell: inside.then(|| {
        (
          col.min(grid.cols.saturating_sub(1)),
          row.min(grid.rows.saturating_sub(1)),
        )
      }),
    })
  }

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

    if self.dragging_split {
      let ratio = (position.x / self.window_size.width).clamp(0.1, 0.9);
      if let Some(tab) = self.tabs.get_mut(self.active_index) {
        tab.split_ratio = ratio;
      }
      return;
    }

    if let Some(pending_pos) = self.drag_pending_pos {
      let dx = position.x - pending_pos.x;
      let dy = position.y - pending_pos.y;
      if dx * dx + dy * dy > 36.0 {
        self.dragging_tab_index = self.drag_pending_tab.take();
        self.drag_pending_pos = None;
      }
    }

    if self.dragging_tab_index.is_some() {
      self.split_drop = self
        .split_drop_host()
        .and_then(|_| split_drop_side(position, self.window_size));
      return;
    }

    let pointer = self.pointer();

    let mut bypass_selection = false;
    if let Some(p) = pointer
      && p.reports(self.shift_held)
    {
      bypass_selection = true;
      if (p.mouse_mode == MouseMode::AnyEvent
        || (p.mouse_mode == MouseMode::Button && self.last_mouse_button.is_some()))
        && let Some((col, row)) = p.cell
      {
        match self.last_mouse_button {
          Some(button) => self.send_mouse_event(col, row, Some(button), true, true),
          None => self.send_mouse_event(col, row, None, false, true),
        }
      }
    }

    if self.is_selecting && !bypass_selection {
      let end = pointer.and_then(|p| p.cell);
      self.selection_end = end;
      if self.click_count >= 2
        && let (Some((end_col, end_row)), Some((grid, scroll))) = (end, self.selection_grid())
      {
        match self.click_count {
          2 => {
            if let Some(row_cells) = get_display_row(grid, scroll, end_row) {
              let (_, we) = find_word_boundaries(row_cells, end_col);
              self.selection_end = Some((we, end_row));
            }
          }
          3 => {
            self.selection_end = Some((grid.cols.saturating_sub(1), end_row));
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
    if self.settings_open || self.command_palette_open || self.ai_overlay_open {
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

    let pointer = self.pointer();
    let cell = pointer.and_then(|p| p.cell);

    if pointer.is_some_and(|p| p.reports(self.shift_held)) {
      if let Some((col, row)) = cell {
        self.send_mouse_event(col, row, Some(button), true, false);
      }
      self.click_count = 0;
      return iced::Task::none();
    }

    if button == mouse::Button::Left {
      if let Some(tab) = self.tabs.get_mut(self.active_index)
        && tab.split.is_some()
      {
        let divider_x = self.window_size.width * tab.split_ratio;
        if (self.cursor_position.x - divider_x).abs() < 6.0 {
          self.dragging_split = true;
          return iced::Task::none();
        }
        tab.active_pane_is_split = self.cursor_position.x > divider_x;
      }

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
        && let Some((grid, scroll)) = self.selection_grid()
      {
        let span = match self.click_count {
          2 => get_display_row(grid, scroll, row).map(|cells| find_word_boundaries(cells, col)),
          3 => Some((0, grid.cols.saturating_sub(1))),
          _ => None,
        };
        if let Some((start, end)) = span {
          self.selection_start = Some((start, row));
          self.selection_end = Some((end, row));
        }
      }
    }
    iced::Task::none()
  }

  pub(super) fn handle_mouse_released(
    &mut self,
    button: mouse::Button,
  ) -> iced::Task<super::super::message::Message> {
    if button == mouse::Button::Left
      && let Some(sel) = self.block_selection.as_mut()
    {
      sel.dragging = false;
    }
    if self.last_mouse_button == Some(button) {
      self.last_mouse_button = None;
    }

    if self.dragging_split && button == mouse::Button::Left {
      self.dragging_split = false;
      self.resize_all_grids();
      return iced::Task::none();
    }

    self.drag_pending_tab = None;
    self.drag_pending_pos = None;
    if button == mouse::Button::Left && self.dragging_tab_index.is_some() {
      if let (Some(side), Some(host), Some(dragged)) = (
        self.split_drop,
        self.split_drop_host(),
        self.dragging_tab_index,
      ) {
        self.merge_tab_into_split(dragged, host, side);
      }
      self.dragging_tab_index = None;
      self.drag_host_tab = None;
      self.split_drop = None;
      return iced::Task::none();
    }

    self.is_selecting = false;
    if self.settings_open || self.command_palette_open || self.ai_overlay_open {
      return iced::Task::none();
    }

    if let Some(p) = self.pointer()
      && p.reports(self.shift_held)
    {
      if let Some((col, row)) = p.cell {
        self.send_mouse_event(col, row, Some(button), false, false);
      }
      return iced::Task::none();
    }

    if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
      if start == end {
        self.selection_start = None;
        self.selection_end = None;
      } else if let Some((grid, scroll)) = self.selection_grid() {
        let text = extract_selection(grid, scroll, start, end);
        if !text.is_empty() {
          return iced::clipboard::write(text);
        }
      }
    }
    iced::Task::none()
  }

  fn wheel_button(&self, delta: f32) -> u8 {
    let mut cb = if delta < 0.0 { 65 } else { 64 };
    if self.shift_held {
      cb |= 4;
    }
    if self.alt_held {
      cb |= 8;
    }
    if self.ctrl_held {
      cb |= 16;
    }
    cb
  }

  fn scroll_raw_pane(&mut self, delta: f32) -> bool {
    let Some(tab) = self.tabs.get(self.active_index) else {
      return false;
    };
    let split = tab.active_pane_is_split;
    let Some(grid) = tab.pane(split).raw_grid() else {
      return false;
    };
    let reports = grid.mouse_mode != crate::core::grid::MouseMode::None;
    if !reports && !grid.in_alt_screen() {
      let rows = (delta.abs() * 3.0).round().max(1.0) as isize;
      let rows = if delta > 0.0 { rows } else { -rows };
      let mut moved = 0;
      if let Some(tab) = self.tabs.get_mut(self.active_index)
        && let Some(block) = tab.pane_mut(split).session.raw_block_mut()
      {
        let before = block.raw_scroll as isize;
        block.scroll_raw(rows);
        moved = block.raw_scroll as isize - before;
      }
      for point in [&mut self.selection_start, &mut self.selection_end] {
        if let Some((col, row)) = *point {
          *point = Some((col, (row as isize + moved).max(0) as usize));
        }
      }
      return true;
    }
    let tx = if split {
      tab.split.as_ref().and_then(|s| s.pty_tx.as_ref())
    } else {
      tab.pty_tx.as_ref()
    };
    let Some(tx) = tx else {
      return true;
    };
    let bytes = if reports {
      let pane_x = if split {
        split_widths(self.window_size.width, tab.split_ratio).0 + SPLIT_DIVIDER_WIDTH
      } else {
        0.0
      };
      let compact = crate::ui::theme::tokens::block::is_compact(self.window_size.width);
      let (col, row) = raw_pixel_to_cell(
        self.cursor_position,
        pane_x,
        compact,
        self.settings.theme.font.size,
      );
      wheel_report(
        self.wheel_button(delta),
        col.min(grid.cols.saturating_sub(1)),
        row.min(grid.rows.saturating_sub(1)),
        grid.mouse_sgr,
      )
    } else {
      alternate_scroll_keys(delta)
    };
    let _ = tx.try_send(PtyCommand::Input(bytes));
    true
  }

  pub(super) fn handle_scroll(&mut self, delta: f32) {
    if self.settings_open {
      return;
    }
    let font_size = self.settings.theme.font.size;
    if self.scroll_raw_pane(delta) {
      return;
    }
    let cb = self.wheel_button(delta);
    if let Some(tab) = self.tabs.get_mut(self.active_index) {
      if tab.grid.mouse_mode != crate::core::grid::MouseMode::None && !self.shift_held {
        if let Some((col, row)) = pixel_to_cell(self.cursor_position, font_size)
          && let Some(tx) = &tab.pty_tx
        {
          let report = wheel_report(cb, col, row, tab.grid.mouse_sgr);
          let _ = tx.try_send(PtyCommand::Input(report));
        }
        return;
      }

      let rows = (delta.abs() * 3.0).round() as usize;
      if tab.active_pane_is_split {
        if let Some(split) = &mut tab.split {
          if delta > 0.0 {
            let new_offset = split.scroll_offset.saturating_add(rows);
            split.scroll_offset = new_offset.min(split.grid.scrollback.len());
          } else {
            split.scroll_offset = split.scroll_offset.saturating_sub(rows);
          }
        }
        return;
      }
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
    col: usize,
    row: usize,
    button: Option<mouse::Button>,
    pressed: bool,
    motion: bool,
  ) {
    let Some(tab) = self.tabs.get(self.active_index) else {
      return;
    };
    let split = tab.active_pane_is_split;
    let raw = tab.pane(split).raw_grid();
    let tx = if raw.is_some() && split {
      tab.split.as_ref().and_then(|s| s.pty_tx.as_ref())
    } else {
      tab.pty_tx.as_ref()
    };
    let Some(tx) = tx else { return };
    let sgr = raw.unwrap_or(&tab.grid).mouse_sgr;

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

    if sgr {
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
