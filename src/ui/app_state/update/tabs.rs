use crate::sys::pty::PtyCommand;
use crate::ui::tab::Tab;

use super::super::helpers::{calc_grid, command_history_path};
use super::super::nova::Nova;

impl Nova {
  pub(super) fn load_command_history(&mut self) {
    let Some(path) = command_history_path() else {
      return;
    };
    let Ok(data) = std::fs::read(&path) else {
      return;
    };
    let Ok(history) =
      bincode::deserialize::<std::collections::VecDeque<crate::core::grid::CommandEntry>>(&data)
    else {
      return;
    };
    for tab in self.tabs.iter_mut() {
      tab.grid.command_history = history.clone();
    }
  }

  pub(super) fn resize_all_grids(&mut self) {
    let (cols, rows) = calc_grid(
      self.window_size.width,
      self.window_size.height,
      self.settings.theme.font.size,
      self.settings.status_bar.visible,
      self.diagnostic_banner.is_some(),
    );
    for tab in self.tabs.iter_mut() {
      tab.grid.resize(cols, rows);
      if let Some(tx) = &tab.pty_tx {
        let _ = tx.send_blocking(PtyCommand::Resize {
          cols: cols as u16,
          rows: rows as u16,
        });
      }
    }
  }

  pub(super) fn handle_close_tab(&mut self, index: usize) {
    self.tabs.remove(index);
    if self.tabs.is_empty() {
      let shell = self.available_shells.first().cloned().unwrap_or_default();
      let (cols, rows) = calc_grid(
        self.window_size.width,
        self.window_size.height,
        self.settings.theme.font.size,
        self.settings.status_bar.visible,
        self.diagnostic_banner.is_some(),
      );
      self
        .tabs
        .push(Tab::new(self.next_tab_id, cols, rows, shell, String::new()));
      self.next_tab_id += 1;
      self.active_index = 0;
    } else if self.active_index >= self.tabs.len() {
      self.active_index = self.tabs.len() - 1;
    }
  }
}
