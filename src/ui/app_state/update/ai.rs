use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::core::router::Mode;
use crate::ui::components::{AiAnswer, AiItem, AiKind, ai_items};

use super::super::helpers::os_name;
use super::super::message::Message;
use super::super::nova::{AI_OPEN, Nova};

pub const RUN_DELAY: Duration = Duration::from_millis(120);

pub fn question_for(kind: AiKind, query: &str, last: Option<(&str, i32)>) -> Option<String> {
  match kind {
    AiKind::Generate => Some(format!(
      "Suggest one shell command for: {query}\nAnswer with one short sentence, then the command in a single fenced code block."
    )),
    AiKind::Diagnose => last.map(|(cmd, code)| {
      format!(
        "The command `{cmd}` exited with code {code}. In one or two short sentences, name the concrete cause in this case (for example a typo, a missing file, a wrong flag), not what the exit code means in general and not a restatement of the error text; wrap commands and names in backticks. If the non-zero exit looks expected or intentional, say so briefly. Then give a corrected command that keeps the user's arguments, in a single fenced code block."
      )
    }),
    AiKind::Explain => last.map(|(cmd, _)| {
      format!(
        "Explain in one or two short sentences what `{cmd}` does (wrap commands and names in backticks), then suggest one useful related command in a single fenced code block."
      )
    }),
  }
}

impl Nova {
  fn last_command(&self) -> Option<(String, i32, String)> {
    let tab = self.tabs.get(self.active_index)?;
    let pane = tab.active_pane();
    if pane.integration.mode == Mode::Classic {
      return None;
    }
    let b = pane.session.last_finished()?;
    Some((b.command.clone(), b.exit_code.unwrap_or(0), b.text()))
  }

  fn last_failure(&self) -> Option<(String, i32)> {
    let tab = self.tabs.get(self.active_index)?;
    let b = tab.active_pane().session.last_failure()?;
    Some((b.command.clone(), b.exit_code.unwrap_or(1)))
  }

  pub(in crate::ui::app_state) fn ai_items(&self) -> Vec<AiItem> {
    let failure = self.last_failure();
    ai_items(
      &self.ai_input,
      failure.as_ref().map(|(c, code)| (c.as_str(), *code)),
    )
  }

  fn classic_context(&self) -> String {
    self
      .tabs
      .get(self.active_index)
      .map(|t| crate::core::ai::extract_last_output(&t.grid))
      .unwrap_or_default()
  }

  pub(super) fn open_ai_sheet(&mut self) -> iced::Task<Message> {
    self.sheet_opened_at = Instant::now();
    self.now = self.sheet_opened_at;
    self.ai_overlay_open = true;
    self.ai_answer = None;
    self.ai_selected = 0;
    AI_OPEN.store(true, Ordering::SeqCst);
    iced::widget::operation::focus(crate::ui::components::AI_INPUT_ID.clone())
  }

  pub(super) fn close_ai_sheet(&mut self) -> iced::Task<Message> {
    self.ai_overlay_open = false;
    self.ai_answer = None;
    self.ai_input.clear();
    self.ai_selected = 0;
    AI_OPEN.store(false, Ordering::SeqCst);
    self.focus_input()
  }

  pub(super) fn start_ai(&mut self, kind: AiKind) -> iced::Task<Message> {
    let now = Instant::now();
    self.now = now;
    let last = self.last_command();
    let last_ref = match kind {
      AiKind::Diagnose => self.last_failure(),
      _ => last.as_ref().map(|(c, code, _)| (c.clone(), *code)),
    };
    let Some(question) = question_for(
      kind,
      self.ai_input.trim(),
      last_ref.as_ref().map(|(c, code)| (c.as_str(), *code)),
    ) else {
      let mut a = AiAnswer::loading(kind, now);
      a.loading = false;
      a.body = "Run a command first and I'll walk through what it did.".to_string();
      self.ai_answer = Some(a);
      return iced::Task::none();
    };
    let context = match (&last, kind) {
      (Some((_, _, out)), AiKind::Diagnose | AiKind::Explain) => out.clone(),
      (None, _) => self.classic_context(),
      _ => String::new(),
    };
    let shell = self
      .tabs
      .get(self.active_index)
      .map(|t| t.shell.clone())
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
    self.ai_answer = Some(AiAnswer::loading(kind, now));
    iced::Task::perform(crate::core::ai::query(q), Message::AiResponseReceived)
  }

  pub(super) fn handle_ai_response(
    &mut self,
    result: Result<String, String>,
  ) -> iced::Task<Message> {
    let now = Instant::now();
    self.now = now;
    if let Some(a) = self.ai_answer.as_mut() {
      a.loading = false;
      a.started = now;
      match result {
        Ok(text) => {
          let (body, cmd) = crate::core::ai::split_answer(&text);
          a.body = body;
          a.cmd = cmd;
          a.error = false;
        }
        Err(e) => {
          a.body = e;
          a.cmd = None;
          a.error = true;
        }
      }
    }
    iced::Task::none()
  }

  pub(super) fn handle_ai_pick(&mut self, i: usize) -> iced::Task<Message> {
    match self.ai_items().get(i).map(|it| it.kind) {
      Some(kind) => self.start_ai(kind),
      None => iced::Task::none(),
    }
  }

  pub(super) fn handle_ai_escape(&mut self) -> iced::Task<Message> {
    if self.ai_answer.is_some() {
      self.ai_answer = None;
      return iced::widget::operation::focus(crate::ui::components::AI_INPUT_ID.clone());
    }
    self.close_ai_sheet()
  }

  pub(super) fn handle_ai_run(&mut self) -> iced::Task<Message> {
    let Some(cmd) = self
      .ai_answer
      .as_ref()
      .filter(|a| a.is_done(self.now))
      .and_then(|a| a.cmd.clone())
    else {
      return iced::Task::none();
    };
    let close = self.close_ai_sheet();
    let run = iced::Task::perform(
      async move {
        tokio::time::sleep(RUN_DELAY).await;
        cmd
      },
      Message::RunCommand,
    );
    iced::Task::batch([close, run])
  }

  pub(super) fn handle_run_command(&mut self, cmd: String) -> iced::Task<Message> {
    let Some(tab) = self.tabs.get_mut(self.active_index) else {
      return iced::Task::none();
    };
    let pane = tab.active_pane_mut();
    if pane.integration.is_blocks() {
      pane.input.set(cmd);
      return self.handle_input_submit();
    }
    let mut bytes = cmd.into_bytes();
    bytes.push(b'\r');
    pane.write(bytes);
    iced::Task::none()
  }

  pub(super) fn handle_diagnostic_banner_response(&mut self, result: Result<String, String>) {
    let Some(code) = self.ai_diagnostic_inflight.take() else {
      return;
    };
    if !self.settings.ai.diagnostic_banner {
      return;
    }
    self.diagnostic_banner = match result {
      Ok(text) => crate::core::ai::parse_diagnosis(&text).map(|d| (code, d.message, d.command)),
      Err(e) => Some((code, format!("AI error: {}", e), None)),
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn questions_need_a_last_command_except_generate() {
    assert!(question_for(AiKind::Generate, "list ports", None).is_some());
    assert!(question_for(AiKind::Explain, "", None).is_none());
    let q = question_for(AiKind::Diagnose, "", Some(("clsd", 127))).unwrap();
    assert!(q.contains("`clsd`") && q.contains("127"));
  }
}
