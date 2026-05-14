use crate::sys::pty::PtyCommand;

use super::super::helpers::calc_grid;
use super::super::message::Message;
use super::super::nova::Nova;

impl Nova {
  pub(super) fn handle_window_resized(&mut self, width: f32, height: f32) -> iced::Task<Message> {
    if width < 100.0 || height < 100.0 {
      return iced::Task::none();
    }
    self.window_size = iced::Size::new(width, height);
    let (cols, rows) = calc_grid(
      width,
      height,
      self.settings.theme.font.size,
      self.settings.status_bar.visible,
      self.diagnostic_banner.is_some(),
    );

    for tab in self.tabs.iter_mut() {
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

    if let Some(id) = self.window_id {
      return iced::window::is_maximized(id).map(Message::WindowMaximizedState);
    }
    iced::Task::none()
  }
}
