use std::sync::atomic::Ordering;

use super::super::helpers::os_name;
use super::super::message::Message;
use super::super::nova::{AI_OPEN, Nova};

impl Nova {
  pub(super) fn handle_ai_submit(&mut self) -> iced::Task<Message> {
    if self.ai_input.trim().is_empty() {
      return iced::Task::none();
    }
    let question = self.ai_input.clone();
    let (context, shell) = self
      .tabs
      .get(self.active_index)
      .map(|tab| {
        (
          crate::core::ai::extract_last_output(&tab.grid),
          tab.shell.clone(),
        )
      })
      .unwrap_or_default();
    let ai_cfg = &self.settings.ai;
    let q = crate::core::ai::AiQuery {
      question,
      context,
      provider: ai_cfg.provider.clone(),
      model: ai_cfg.model.clone(),
      api_key: ai_cfg.api_key.clone(),
      base_url: ai_cfg.base_url.clone(),
      shell,
      os: os_name(),
    };
    self.ai_loading = true;
    self.ai_response = None;
    self.ai_overlay_open = true;
    AI_OPEN.store(true, Ordering::SeqCst);
    iced::Task::perform(crate::core::ai::query(q), Message::AiResponseReceived)
  }

  pub(super) fn handle_explain_error(&mut self) -> iced::Task<Message> {
    let (context, shell) = self
      .tabs
      .get(self.active_index)
      .map(|tab| {
        (
          crate::core::ai::extract_last_output(&tab.grid),
          tab.shell.clone(),
        )
      })
      .unwrap_or_default();
    let ai_cfg = &self.settings.ai;
    let q = crate::core::ai::AiQuery {
      question: "Explain any errors in the terminal output above.".to_string(),
      context,
      provider: ai_cfg.provider.clone(),
      model: ai_cfg.model.clone(),
      api_key: ai_cfg.api_key.clone(),
      base_url: ai_cfg.base_url.clone(),
      shell,
      os: os_name(),
    };
    self.ai_loading = true;
    self.ai_response = None;
    self.ai_overlay_open = true;
    AI_OPEN.store(true, Ordering::SeqCst);
    self.ai_input = String::new();
    iced::Task::perform(crate::core::ai::query(q), Message::AiResponseReceived)
  }

  pub(super) fn handle_diagnostic_banner_response(&mut self, result: Result<String, String>) {
    let code = self
      .diagnostic_banner
      .as_ref()
      .map(|(c, _, _)| *c)
      .unwrap_or(0);
    match result {
      Ok(text) => {
        let (msg, cmd) = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
          let message = val
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or(&text)
            .to_string();
          let command = val
            .get("command_to_solve")
            .and_then(|c| c.as_str().map(|s| s.to_string()));
          (message, command)
        } else {
          (text.clone(), None)
        };
        self.diagnostic_banner = Some((code, msg, cmd));
      }
      Err(e) => {
        self.diagnostic_banner = Some((code, format!("AI error: {}", e), None));
      }
    }
  }
}
