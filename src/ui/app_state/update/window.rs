use crate::core::router::Mode;
use crate::sys::pty::PtyCommand;
use crate::ui::tab::PaneMut;

use super::super::helpers::{calc_block_grid, calc_grid, calc_grid_split_ratio, split_widths};
use super::super::message::Message;
use super::super::nova::Nova;

impl Nova {
  pub(super) fn debounce_font_resize(&mut self) -> iced::Task<Message> {
    self.font_resize_generation = self.font_resize_generation.wrapping_add(1);
    let epoch = self.font_resize_generation;
    iced::Task::perform(
      async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        epoch
      },
      Message::FontResizeSettled,
    )
  }
}

impl Nova {
  pub(super) fn handle_window_resized(&mut self, width: f32, height: f32) -> iced::Task<Message> {
    if width < 100.0 || height < 100.0 {
      return iced::Task::none();
    }
    let banner_visible = self.diagnostic_banner.is_some();
    let font_size = self.settings.theme.font.size;

    self.resize_panes(width, height, banner_visible, font_size);

    if let Some(id) = self.window_id {
      return iced::window::is_maximized(id).map(Message::WindowMaximizedState);
    }
    iced::Task::none()
  }
}

fn apply_size(pane: &mut PaneMut<'_>, cols: usize, rows: usize) {
  if pane.grid.cols != cols || pane.grid.rows != rows {
    pane.grid.resize(cols, rows);
  }
  if pane.session.cols != cols || pane.session.rows != rows {
    pane.session.resize(cols, rows);
  }
  if let Some(tx) = pane.pty_tx {
    let _ = tx.send_blocking(PtyCommand::Resize {
      cols: cols as u16,
      rows: rows as u16,
    });
  }
}

impl Nova {
  pub(super) fn resize_panes(
    &mut self,
    width: f32,
    height: f32,
    banner_visible: bool,
    font_size: f32,
  ) {
    let timestamps = self.timestamps;
    for tab in self.tabs.iter_mut() {
      let has_split = tab.split.is_some();
      let (classic_left, classic_right, classic_rows) = if has_split {
        calc_grid_split_ratio(width, height, font_size, banner_visible, tab.split_ratio)
      } else {
        let (c, r) = calc_grid(width, height, font_size, banner_visible);
        (c, c, r)
      };
      let (left_w, right_w) = if has_split {
        split_widths(width, tab.split_ratio)
      } else {
        (width, width)
      };
      for (i, mut pane) in tab.panes_mut().into_iter().enumerate() {
        let blocks = pane.integration.mode != Mode::Classic;
        let (cols, rows) = if blocks {
          calc_block_grid(
            if i == 0 { left_w } else { right_w },
            height,
            font_size,
            timestamps,
          )
        } else if i == 0 {
          (classic_left, classic_rows)
        } else {
          (classic_right, classic_rows)
        };
        if pane.grid.cols != cols || pane.grid.rows != rows {
          apply_size(&mut pane, cols, rows);
        }
      }
    }
  }
}
