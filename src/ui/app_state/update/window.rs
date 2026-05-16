use crate::sys::pty::PtyCommand;

use super::super::helpers::{calc_grid, calc_grid_split};
use super::super::message::Message;
use super::super::nova::Nova;

impl Nova {
  pub(super) fn handle_window_resized(&mut self, width: f32, height: f32) -> iced::Task<Message> {
    if width < 100.0 || height < 100.0 {
      return iced::Task::none();
    }
    let banner_visible = self.diagnostic_banner.is_some();
    let font_size = self.settings.theme.font.size;
    let status_bar_visible = self.settings.status_bar.visible;

    for tab in self.tabs.iter_mut() {
      if tab.split.is_some() {
        let (cols, rows) =
          calc_grid_split(width, height, font_size, status_bar_visible, banner_visible);
        if tab.grid.cols != cols || tab.grid.rows != rows {
          tab.grid.resize(cols, rows);
          if let Some(tx) = &tab.pty_tx {
            let _ = tx.send_blocking(PtyCommand::Resize {
              cols: cols as u16,
              rows: rows as u16,
            });
          }
        }
        if let Some(split) = &mut tab.split
          && (split.grid.cols != cols || split.grid.rows != rows)
        {
          split.grid.resize(cols, rows);
          if let Some(tx) = &split.pty_tx {
            let _ = tx.send_blocking(PtyCommand::Resize {
              cols: cols as u16,
              rows: rows as u16,
            });
          }
        }
      } else {
        let (cols, rows) = calc_grid(width, height, font_size, status_bar_visible, banner_visible);
        if tab.grid.cols == cols && tab.grid.rows == rows {
          continue;
        }
        tab.grid.resize(cols, rows);
        if let Some(tx) = &tab.pty_tx {
          let _ = tx.send_blocking(PtyCommand::Resize {
            cols: cols as u16,
            rows: rows as u16,
          });
        }
      }
    }

    if let Some(id) = self.window_id {
      return iced::window::is_maximized(id).map(Message::WindowMaximizedState);
    }
    iced::Task::none()
  }
}
