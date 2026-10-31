use std::time::Instant;

use super::super::message::Message;
use super::super::nova::Nova;

pub fn is_local_clear(command: &str) -> bool {
  matches!(command.trim(), "clear" | "cls")
}

impl Nova {
  pub(super) fn handle_input_changed(&mut self, text: String) -> iced::Task<Message> {
    if let Some(tab) = self.tabs.get_mut(self.active_index) {
      tab.active_pane_mut().input.set(text);
    }
    iced::Task::none()
  }

  pub(super) fn handle_input_submit(&mut self) -> iced::Task<Message> {
    let Some(tab) = self.tabs.get_mut(self.active_index) else {
      return iced::Task::none();
    };
    let pane = tab.active_pane_mut();
    if !pane.integration.is_blocks() || pane.session.is_running() {
      pane.input.submitted += 1;
      return iced::Task::none();
    }
    let command = pane.input.take();
    let trimmed = command.trim();
    if trimmed.is_empty() {
      return iced::Task::none();
    }
    if is_local_clear(trimmed) {
      pane.session.push_history(trimmed);
      pane.session.clear();
      return iced::Task::none();
    }
    if pane.session.submit(trimmed).is_some() {
      pane.integration.on_submit();
      let mut bytes = trimmed.as_bytes().to_vec();
      bytes.push(b'\r');
      pane.write(bytes);
    }
    iced::Task::none()
  }

  pub(super) fn handle_input_history(&mut self, delta: i32) -> iced::Task<Message> {
    if let Some(tab) = self.tabs.get_mut(self.active_index) {
      let pane = tab.active_pane_mut();
      if delta < 0 {
        pane.input.history_up(pane.session);
      } else {
        pane.input.history_down(pane.session);
      }
    }
    iced::Task::none()
  }

  pub(super) fn handle_clear_blocks(&mut self) -> iced::Task<Message> {
    if let Some(tab) = self.tabs.get_mut(self.active_index) {
      tab.active_pane_mut().session.clear();
    }
    iced::Task::none()
  }

  pub(super) fn block_mode_panes(&self) -> usize {
    self
      .tabs
      .iter()
      .map(|t| {
        usize::from(t.integration.is_blocks())
          + t
            .split
            .as_ref()
            .map_or(0, |s| usize::from(s.integration.is_blocks()))
      })
      .sum()
  }

  pub(super) fn expire_integrations(&mut self, now: Instant) {
    let mut expired = false;
    for tab in self.tabs.iter_mut() {
      for pane in tab.panes_mut() {
        if pane.integration.expire(now) {
          expired = true;
        }
      }
    }
    if expired {
      self.resize_all_grids();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn clear_aliases_are_local() {
    assert!(is_local_clear(" clear "));
    assert!(is_local_clear("cls"));
    assert!(!is_local_clear("clear-host"));
  }
}
