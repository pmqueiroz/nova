use std::sync::atomic::Ordering;

use crate::ui::components::{self, PaletteAction};

use super::super::message::Message;
use super::super::nova::{Nova, PALETTE_OPEN};

pub fn action_message(action: PaletteAction) -> Message {
  match action {
    PaletteAction::NewTab => Message::NewTab,
    PaletteAction::CloseTab => Message::CloseActiveTab,
    PaletteAction::Clear => Message::ClearBlocks,
    PaletteAction::AskAi => Message::OpenAskAi,
    PaletteAction::FontUp => Message::FontSizeUp,
    PaletteAction::FontDown => Message::FontSizeDown,
    PaletteAction::ToggleTimestamps => Message::ToggleTimestamps,
    PaletteAction::Settings => Message::OpenSettings,
    PaletteAction::SplitPane => Message::SplitPane,
    PaletteAction::Find => Message::SearchOpen,
  }
}

impl Nova {
  pub(super) fn close_palette(&mut self) {
    self.command_palette_open = false;
    self.palette_query = String::new();
    self.palette_selected = 0;
    PALETTE_OPEN.store(false, Ordering::SeqCst);
  }

  pub(super) fn handle_palette_select_confirm(&mut self, i: usize) -> iced::Task<Message> {
    if let Some(action) = components::palette_action_at(&self.palette_query, i) {
      self.close_palette();
      return self.update(action_message(action));
    }
    iced::Task::none()
  }
}
