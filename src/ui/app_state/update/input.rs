#[cfg(target_os = "windows")]
use std::io::Write;
use std::sync::atomic::Ordering;

use crate::core::grid::ControlCommand;
use crate::sys::parser::AnsiExecutor;
use crate::sys::pty::PtyCommand;
use crate::ui::components;

use super::super::helpers::{command_history_path, os_name};
use super::super::message::Message;
use super::super::nova::{AI_OPEN, Nova};

impl Nova {
  pub(super) fn handle_type_input(&mut self, bytes: Vec<u8>) -> iced::Task<Message> {
    if self.settings_open || self.command_palette_open || self.ai_overlay_open || self.ai_loading {
      return iced::Task::none();
    }

    let Some(active_tab) = self.tabs.get(self.active_index) else {
      return iced::Task::none();
    };
    let active_pane_is_split = active_tab.active_pane_is_split;

    if active_pane_is_split {
      self.selection_start = None;
      self.selection_end = None;
      self.click_count = 0;
      self.diagnostic_banner = None;
      self.ai_pending_diagnostic = None;
      if let Some(active_tab) = self.tabs.get_mut(self.active_index)
        && let Some(split) = &mut active_tab.split
      {
        if bytes == b"\t"
          && let Some(suggestion) = split.grid.suggestion.take()
          && !suggestion.is_empty()
        {
          if let Some(tx) = &split.pty_tx {
            let _ = tx.send_blocking(PtyCommand::Input(suggestion.into_bytes()));
          }
          split.grid.input_start_col = None;
          split.grid.input_start_row = None;
          return iced::Task::none();
        }
        split.scroll_offset = 0;
        if bytes == b"\r" {
          if !split.current_input.is_empty() {
            split.current_input.clear();
            split.command_start = Some(std::time::Instant::now());
            split.last_pty_output = None;
          }
        } else if bytes == b"\x7F" || bytes == b"\x08" {
          split.current_input.pop();
        } else if bytes == b"\x03" || bytes == b"\x15" {
          split.current_input.clear();
          split.command_start = None;
          split.last_pty_output = None;
        } else if bytes.len() == 1 {
          let b = bytes[0];
          if b.is_ascii_graphic() || b == b' ' {
            split.current_input.push(b as char);
            if split.grid.input_start_col.is_none()
              && let Some(start) = split.command_start.take()
            {
              let elapsed = split
                .last_pty_output
                .map(|last| last.duration_since(start))
                .unwrap_or_else(|| start.elapsed());
              split.last_command_elapsed = Some(elapsed);
              split.last_pty_output = None;
            }
          } else {
            split.current_input.clear();
          }
        }
        if let Some(tx) = &split.pty_tx {
          let _ = tx.send_blocking(PtyCommand::Input(bytes));
        }
      }
      return iced::Task::none();
    }

    if bytes == b"\t"
      && let Some(active_tab) = self.tabs.get_mut(self.active_index)
      && let Some(suggestion) = active_tab.grid.suggestion.take()
      && !suggestion.is_empty()
    {
      if let Some(tx) = &active_tab.pty_tx {
        let _ = tx.send_blocking(PtyCommand::Input(suggestion.into_bytes()));
      }
      active_tab.current_input.clear();
      active_tab.grid.input_start_col = None;
      active_tab.grid.input_start_row = None;
      return iced::Task::none();
    }

    self.selection_start = None;
    self.selection_end = None;
    self.click_count = 0;
    self.diagnostic_banner = None;
    self.ai_pending_diagnostic = None;
    let entered = bytes == b"\r";
    if let Some(active_tab) = self.tabs.get_mut(self.active_index) {
      active_tab.scroll_offset = 0;

      if bytes == b"\r" {
        if !active_tab.current_input.is_empty() {
          let input = std::mem::take(&mut active_tab.current_input);
          active_tab.grid.push_command(&input);
          active_tab.grid.suggestion = None;
          active_tab.grid.input_start_col = None;
          active_tab.grid.input_start_row = None;
          active_tab.command_start = Some(std::time::Instant::now());
          active_tab.last_pty_output = None;
        }
      } else if bytes == b"\x7F" || bytes == b"\x08" {
        active_tab.current_input.pop();
      } else if bytes == b"\x03"
        || bytes == b"\x15"
        || (bytes.len() >= 2 && bytes[0] == 0x1b && (bytes[1] == b'A' || bytes[1] == b'B'))
      {
        active_tab.current_input.clear();
        active_tab.command_start = None;
        active_tab.last_pty_output = None;
        active_tab.grid.suggestion = None;
        active_tab.grid.input_start_col = None;
        active_tab.grid.input_start_row = None;
      } else if bytes.len() == 1 {
        let b = bytes[0];
        if b.is_ascii_graphic() || b == b' ' {
          active_tab.current_input.push(b as char);
          if active_tab.grid.input_start_col.is_none() {
            let col = active_tab.grid.cursor_x.saturating_sub(1);
            active_tab.grid.input_start_col = Some(col);
            active_tab.grid.input_start_row = Some(active_tab.grid.cursor_y);
            if let Some(start) = active_tab.command_start.take() {
              let elapsed = active_tab
                .last_pty_output
                .map(|last| last.duration_since(start))
                .unwrap_or_else(|| start.elapsed());
              active_tab.last_command_elapsed = Some(elapsed);
              active_tab.last_pty_output = None;
            }
          }
        } else {
          active_tab.current_input.clear();
          active_tab.grid.suggestion = None;
          active_tab.grid.input_start_col = None;
          active_tab.grid.input_start_row = None;
        }
      }

      if let Some(tx) = &active_tab.pty_tx {
        let _ = tx.send_blocking(PtyCommand::Input(bytes));
      }
    }
    if entered {
      self.save_command_history();
    }
    iced::Task::none()
  }

  pub(super) fn handle_pty_output(&mut self, tab_id: usize, bytes: Vec<u8>) -> iced::Task<Message> {
    if let Some(tab_idx) = self
      .tabs
      .iter()
      .position(|t| t.split.as_ref().map(|s| s.id) == Some(tab_id))
    {
      let tab = &mut self.tabs[tab_idx];
      let split = tab.split.as_mut().unwrap();
      #[cfg(target_os = "windows")]
      if std::env::var("NOVA_DEBUG_PTY").is_ok()
        && let Ok(mut f) = std::fs::OpenOptions::new()
          .create(true)
          .append(true)
          .open(std::env::temp_dir().join("nova_pty_debug.bin"))
      {
        let _ = f.write_all(&bytes);
      }
      let mut executor = AnsiExecutor {
        grid: &mut split.grid,
        bell_pending: false,
      };
      for byte in bytes {
        split.ansi_parser.advance(&mut executor, &[byte]);
      }
      while !split.grid.output_queue.is_empty() {
        let response = split.grid.output_queue.remove(0);
        if let Some(tx) = &split.pty_tx {
          let _ = tx.send_blocking(PtyCommand::Input(response));
        }
      }
      for cmd in split.grid.control_queue.drain(..) {
        if let ControlCommand::CommandComplete(_) = cmd
          && let Some(start) = split.command_start.take()
        {
          let elapsed = split
            .last_pty_output
            .map(|last| last.duration_since(start))
            .unwrap_or_else(|| start.elapsed());
          split.last_command_elapsed = Some(elapsed);
          split.last_pty_output = None;
        }
      }
      if split.command_start.is_some() {
        split.last_pty_output = Some(std::time::Instant::now());
      }
      let new_pwd = split.grid.pwd.clone();
      if new_pwd != split.pwd {
        split.pwd = new_pwd;
        split.update_git_status();
      }
      split.scroll_offset = 0;
      if let Some(partial) = split.grid.extract_current_input() {
        split.grid.suggestion = split.grid.find_best_suggestion(&partial);
      } else {
        split.grid.suggestion = None;
      }
      return iced::Task::none();
    }

    let active_tab_id = self.tabs.get(self.active_index).map(|t| t.id);
    if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
      #[cfg(target_os = "windows")]
      if std::env::var("NOVA_DEBUG_PTY").is_ok()
        && let Ok(mut f) = std::fs::OpenOptions::new()
          .create(true)
          .append(true)
          .open(std::env::temp_dir().join("nova_pty_debug.bin"))
      {
        let _ = f.write_all(&bytes);
      }
      let mut executor = AnsiExecutor {
        grid: &mut tab.grid,
        bell_pending: false,
      };
      for byte in bytes {
        tab.ansi_parser.advance(&mut executor, &[byte]);
      }
      let bell_fired = executor.bell_pending;

      let mut open_ask_ai = false;
      let mut open_explain_ai = false;
      let mut ai_preset: Option<std::sync::Arc<str>> = None;
      for cmd in tab.grid.control_queue.drain(..) {
        match cmd {
          ControlCommand::OpenAskAi { preset } => {
            open_ask_ai = true;
            if let Some(p) = preset
              && !p.trim().is_empty()
            {
              ai_preset = Some(p);
            }
          }
          ControlCommand::OpenExplainAi { preset } => {
            open_explain_ai = true;
            if let Some(p) = preset
              && !p.trim().is_empty()
            {
              ai_preset = Some(p);
            }
          }
          ControlCommand::CommandFailure(code) => {
            if self.settings.ai.diagnostic_banner
              && code != 0
              && code < 128
              && !self.settings.ai.api_key.is_empty()
            {
              self.diagnostic_banner = Some((code, "Loading...".into(), None));
              self.ai_pending_diagnostic = Some(code);
            }
          }
          ControlCommand::CommandComplete(_code) => {
            const NOTIFY_THRESHOLD_SECS: u64 = 10;
            if let Some(start) = tab.command_start.take() {
              let elapsed = tab
                .last_pty_output
                .map(|last| last.duration_since(start))
                .unwrap_or_else(|| start.elapsed());
              tab.last_command_elapsed = Some(elapsed);
              tab.last_pty_output = None;
              if elapsed.as_secs() >= NOTIFY_THRESHOLD_SECS
                && (!self.window_focused || Some(tab_id) != active_tab_id)
              {
                let secs = elapsed.as_secs();
                let body = if secs < 60 {
                  format!("Finished in {}s", secs)
                } else {
                  format!("Finished in {}m {}s", secs / 60, secs % 60)
                };
                crate::sys::notification::send("Nova", &body);
                tab.command_done = true;
              }
            }
          }
        }
      }

      while !tab.grid.output_queue.is_empty() {
        let response = tab.grid.output_queue.remove(0);
        if let Some(tx) = &tab.pty_tx {
          let _ = tx.send_blocking(PtyCommand::Input(response));
        }
      }

      let new_pwd = tab.grid.pwd.clone();
      if new_pwd != tab.pwd {
        tab.pwd = new_pwd;
      }
      tab.update_git_status();
      tab.scroll_offset = 0;

      if tab.command_start.is_some() {
        tab.last_pty_output = Some(std::time::Instant::now());
      }

      if let Some(partial) = tab.grid.extract_current_input() {
        tab.grid.suggestion = tab.grid.find_best_suggestion(&partial);
      } else {
        tab.grid.suggestion = None;
      }

      if open_ask_ai || open_explain_ai {
        self.ai_overlay_open = true;
        AI_OPEN.store(true, Ordering::SeqCst);

        if open_explain_ai {
          self.ai_input = ai_preset
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Explain any errors in the terminal output above.".to_string());
        } else {
          self.ai_input = ai_preset.map(|s| s.to_string()).unwrap_or_default();
        }

        self.ai_response = None;
        self.ai_is_error = false;
        self.ai_loading = false;

        let focus_task = iced::widget::operation::focus(components::AI_INPUT_ID.clone());
        if !self.ai_input.trim().is_empty() {
          return iced::Task::batch(vec![focus_task, self.update(Message::AiSubmit)]);
        }
        return focus_task;
      }

      if let Some(code) = self.ai_pending_diagnostic.take() {
        let context = crate::core::ai::extract_last_output(&tab.grid);
        let ai_cfg = self.settings.ai.clone();
        let question = format!(
          "The last command exited with code {}. Output:\n{}\n\nRespond in EXACTLY this JSON format (no markdown, no code fences): {{\"message\": \"short explanation\", \"command_to_solve\": \"command to fix it\" or null}}",
          code, context,
        );
        let q = crate::core::ai::AiQuery {
          question,
          context,
          provider: ai_cfg.provider,
          model: ai_cfg.model,
          api_key: ai_cfg.api_key,
          base_url: ai_cfg.base_url,
          shell: tab.shell.clone(),
          os: os_name(),
        };
        return iced::Task::perform(crate::core::ai::query(q), Message::DiagnosticBannerResponse);
      }

      if bell_fired {
        match self.settings.general.bell {
          crate::core::config::BellType::Blink => {
            self.bell_blink_visible = false;
            self.bell_blink_remaining = 5;
          }
          crate::core::config::BellType::Audio => {
            crate::sys::bell::play();
          }
          crate::core::config::BellType::None => {}
        }
      }
    }
    iced::Task::none()
  }

  fn save_command_history(&self) {
    let Some(path) = command_history_path() else {
      return;
    };
    if let Some(tab) = self.tabs.get(self.active_index)
      && let Ok(data) = bincode::serialize(&tab.grid.command_history)
    {
      let _ = std::fs::write(&path, &data);
    }
  }
}
