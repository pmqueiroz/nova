use std::sync::atomic::Ordering;

use crate::ui::components;

use super::super::message::Message;
use super::super::nova::{Nova, PALETTE_OPEN};

impl Nova {
  pub(super) fn handle_palette_select_confirm(&mut self, i: usize) -> iced::Task<Message> {
    if let Some(id) = components::palette_command_id_at(&self.palette_query, i) {
      let msg = match id {
        "ask_ai" => Message::OpenAskAi,
        "explain_error" => Message::ExplainError,
        "new_tab" => Message::NewTab,
        "settings" => Message::OpenSettings,
        _ => Message::NoOp,
      };
      self.command_palette_open = false;
      self.palette_query = String::new();
      self.palette_selected = 0;
      PALETTE_OPEN.store(false, Ordering::SeqCst);
      return self.update(msg);
    }
    iced::Task::none()
  }
}
