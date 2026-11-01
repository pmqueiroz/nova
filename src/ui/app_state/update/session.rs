use std::time::Instant;

use crate::core::selection::{BlockSelection, cell_at, url_at};
use crate::ui::components::session::{self, BlockUrl};
use crate::ui::theme::tokens;

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
      pane.input.pulsed_at = Some(Instant::now());
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
    } else if pane.session.submit(trimmed).is_some() {
      pane.integration.on_submit();
      let bytes = pane.integration.submit_bytes(trimmed);
      pane.write(bytes);
    }
    self.save_session_history();
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

  pub(super) fn handle_blocks_scrolled(
    &mut self,
    pane_id: usize,
    pinned: bool,
  ) -> iced::Task<Message> {
    for tab in self.tabs.iter_mut() {
      if let Some(pane) = tab.pane_by_id_mut(pane_id) {
        pane.input.scrolled_up = !pinned;
        break;
      }
    }
    iced::Task::none()
  }

  pub(super) fn focus_input(&self) -> iced::Task<Message> {
    match self.tabs.get(self.active_index) {
      Some(tab) => iced::widget::operation::focus(session::input_id(tab.active_pane().id)),
      None => iced::Task::none(),
    }
  }

  pub(super) fn follow_output(&mut self, pane_id: usize) -> iced::Task<Message> {
    for tab in self.tabs.iter_mut() {
      if let Some(pane) = tab.pane_by_id_mut(pane_id) {
        if pane.integration.is_blocks() && !pane.input.scrolled_up {
          return iced::widget::operation::snap_to_end(session::scroll_id(pane_id));
        }
        break;
      }
    }
    iced::Task::none()
  }

  fn find_block(&self, pane_id: usize, block_id: u64) -> Option<&crate::core::block::Block> {
    self.tabs.iter().find_map(|t| {
      [false, true]
        .into_iter()
        .filter(|split| !*split || t.split.is_some())
        .map(|split| t.pane(split))
        .find(|p| p.id == pane_id)
        .and_then(|p| p.session.block(block_id))
    })
  }

  pub(super) fn handle_block_pointer(
    &mut self,
    pane_id: usize,
    block_id: u64,
    p: iced::Point,
  ) -> iced::Task<Message> {
    let fs = tokens::block::font_size(self.settings.theme.font.size);
    let pad =
      crate::ui::components::block::card_padding(tokens::block::is_compact(self.window_size.width));
    let cell = cell_at(
      p.x,
      p.y,
      pad.left,
      pad.top,
      fs * tokens::block::CHAR_WIDTH_RATIO,
      fs * tokens::block::LINE_HEIGHT,
    );
    self.block_pointer = Some((pane_id, block_id, cell));
    if let Some(sel) = self.block_selection.as_mut()
      && sel.dragging
      && sel.pane_id == pane_id
      && sel.block_id == block_id
    {
      sel.head = cell;
    }
    self.block_url = self.find_block(pane_id, block_id).and_then(|b| {
      url_at(b, cell).map(|(start, end, url)| BlockUrl {
        pane_id,
        block_id,
        line: cell.0,
        start,
        end,
        url,
      })
    });
    iced::Task::none()
  }

  pub(super) fn handle_block_press(
    &mut self,
    pane_id: usize,
    block_id: u64,
  ) -> iced::Task<Message> {
    if self.ctrl_held
      && let Some(u) = self.block_url.as_ref()
      && u.pane_id == pane_id
      && u.block_id == block_id
    {
      let _ = open::that_detached(&u.url);
      return iced::Task::none();
    }
    let cell = self
      .block_pointer
      .filter(|(p, b, _)| *p == pane_id && *b == block_id)
      .map(|(_, _, c)| c)
      .unwrap_or((0, 0));
    self.block_selection = Some(BlockSelection {
      pane_id,
      block_id,
      anchor: cell,
      head: cell,
      dragging: true,
    });
    iced::Task::none()
  }

  pub(super) fn handle_block_release(&mut self) -> iced::Task<Message> {
    if let Some(sel) = self.block_selection.as_mut() {
      sel.dragging = false;
      if sel.is_empty() {
        self.block_selection = None;
        return self.focus_input();
      }
    }
    iced::Task::none()
  }

  pub(super) fn block_selection_text(&self) -> Option<String> {
    let sel = self.block_selection.filter(|s| !s.is_empty())?;
    let text = sel.text(self.find_block(sel.pane_id, sel.block_id)?);
    (!text.is_empty()).then_some(text)
  }

  pub(super) fn raw_panes(&self) -> usize {
    self
      .tabs
      .iter()
      .map(|t| {
        usize::from(t.pane(false).raw_grid().is_some())
          + usize::from(t.split.is_some() && t.pane(true).raw_grid().is_some())
      })
      .sum()
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
