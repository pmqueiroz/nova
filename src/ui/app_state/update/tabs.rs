use vte::Parser;

use crate::core::grid::Grid;
use crate::sys::pty::PtyCommand;
use crate::ui::tab::{SplitPane, Tab, shell_display_name};

use super::super::helpers::{calc_grid, calc_grid_split, command_history_path};
use super::super::message::Message;
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
    let banner_visible = self.diagnostic_banner.is_some();
    for tab in self.tabs.iter_mut() {
      let has_split = tab.split.is_some();
      if has_split {
        let (cols, rows) = calc_grid_split(
          self.window_size.width,
          self.window_size.height,
          self.settings.theme.font.size,
          self.settings.status_bar.visible,
          banner_visible,
        );
        tab.grid.resize(cols, rows);
        if let Some(tx) = &tab.pty_tx {
          let _ = tx.send_blocking(PtyCommand::Resize {
            cols: cols as u16,
            rows: rows as u16,
          });
        }
        if let Some(split) = &mut tab.split {
          split.grid.resize(cols, rows);
          if let Some(tx) = &split.pty_tx {
            let _ = tx.send_blocking(PtyCommand::Resize {
              cols: cols as u16,
              rows: rows as u16,
            });
          }
        }
      } else {
        let (cols, rows) = calc_grid(
          self.window_size.width,
          self.window_size.height,
          self.settings.theme.font.size,
          self.settings.status_bar.visible,
          banner_visible,
        );
        tab.grid.resize(cols, rows);
        if let Some(tx) = &tab.pty_tx {
          let _ = tx.send_blocking(PtyCommand::Resize {
            cols: cols as u16,
            rows: rows as u16,
          });
        }
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

  pub(super) fn handle_split_pane(&mut self) -> iced::Task<Message> {
    let Some(tab) = self.tabs.get_mut(self.active_index) else {
      return iced::Task::none();
    };
    if tab.split.is_some() {
      return iced::Task::none();
    }
    let shell_cmd = tab.shell_cmd.clone();
    let initial_cwd = tab.pwd.clone();
    let split_id = self.next_tab_id;
    self.next_tab_id += 1;

    let (cols, rows) = calc_grid_split(
      self.window_size.width,
      self.window_size.height,
      self.settings.theme.font.size,
      self.settings.status_bar.visible,
      self.diagnostic_banner.is_some(),
    );

    let tab = &mut self.tabs[self.active_index];
    tab.split = Some(SplitPane {
      id: split_id,
      grid: Grid::new(cols, rows),
      pty_tx: None,
      pty_alive: true,
      ansi_parser: Parser::new(),
      shell_cmd,
      pwd: initial_cwd.clone(),
      scroll_offset: 0,
      initial_cwd,
    });
    tab.active_pane_is_split = true;

    let (primary_cols, primary_rows) = calc_grid_split(
      self.window_size.width,
      self.window_size.height,
      self.settings.theme.font.size,
      self.settings.status_bar.visible,
      self.diagnostic_banner.is_some(),
    );
    let tab = &mut self.tabs[self.active_index];
    tab.grid.resize(primary_cols, primary_rows);
    if let Some(tx) = &tab.pty_tx {
      let _ = tx.send_blocking(PtyCommand::Resize {
        cols: primary_cols as u16,
        rows: primary_rows as u16,
      });
    }

    iced::Task::none()
  }

  pub(super) fn handle_close_split_pane(&mut self) {
    let Some(tab) = self.tabs.get_mut(self.active_index) else {
      return;
    };
    tab.split = None;
    tab.active_pane_is_split = false;

    let (cols, rows) = calc_grid(
      self.window_size.width,
      self.window_size.height,
      self.settings.theme.font.size,
      self.settings.status_bar.visible,
      self.diagnostic_banner.is_some(),
    );
    let tab = &mut self.tabs[self.active_index];
    tab.grid.resize(cols, rows);
    if let Some(tx) = &tab.pty_tx {
      let _ = tx.send_blocking(PtyCommand::Resize {
        cols: cols as u16,
        rows: rows as u16,
      });
    }
  }

  pub(super) fn handle_close_left_pane(&mut self) {
    let Some(tab) = self.tabs.get_mut(self.active_index) else {
      return;
    };
    let Some(split) = tab.split.take() else {
      return;
    };

    drop(tab.pty_tx.take());

    tab.id = split.id;
    tab.grid = split.grid;
    tab.pty_tx = split.pty_tx;
    tab.pty_alive = split.pty_alive;
    tab.ansi_parser = split.ansi_parser;
    tab.shell_cmd = split.shell_cmd.clone();
    tab.shell = shell_display_name(&split.shell_cmd);
    tab.pwd = split.pwd;
    tab.scroll_offset = split.scroll_offset;
    tab.initial_cwd = split.initial_cwd;
    tab.current_input = String::new();
    tab.active_pane_is_split = false;
    tab.update_git_status();

    let (cols, rows) = calc_grid(
      self.window_size.width,
      self.window_size.height,
      self.settings.theme.font.size,
      self.settings.status_bar.visible,
      self.diagnostic_banner.is_some(),
    );
    let tab = &mut self.tabs[self.active_index];
    tab.grid.resize(cols, rows);
    if let Some(tx) = &tab.pty_tx {
      let _ = tx.send_blocking(PtyCommand::Resize {
        cols: cols as u16,
        rows: rows as u16,
      });
    }
  }
}
