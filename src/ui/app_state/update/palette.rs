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
    PaletteAction::NewTabWithShell => Message::OpenShellPicker,
  }
}

impl Nova {
  pub(super) fn close_palette(&mut self) {
    self.command_palette_open = false;
    self.palette_query = String::new();
    self.palette_selected = 0;
    PALETTE_OPEN.store(false, Ordering::SeqCst);
  }

  pub(super) fn is_last_live_tab(&self, index: usize) -> bool {
    self.tabs.len() < 2
      && self
        .tabs
        .get(index)
        .is_some_and(|t| t.pty_alive && t.split.is_none())
  }

  pub(super) fn dismiss_sheets(&mut self) {
    self.close_palette();
    self.ai_overlay_open = false;
    self.ai_answer = None;
    super::super::nova::AI_OPEN.store(false, Ordering::SeqCst);
    self.settings_open = false;
    self.settings_recording_index = None;
    super::super::nova::SETTINGS_OPEN.store(false, Ordering::SeqCst);
    super::super::nova::KB_RECORDING.store(false, Ordering::SeqCst);
    self.shell_picker_open = false;
  }

  pub(super) fn handle_palette_select_confirm(&mut self, i: usize) -> iced::Task<Message> {
    if let Some(action) = components::palette_action_at(&self.palette_query, i) {
      self.close_palette();
      return self.update(action_message(action));
    }
    iced::Task::none()
  }
}
