#[cfg(target_os = "windows")]
use std::io::Write;
use std::sync::atomic::Ordering;

use crate::core::grid::{ControlCommand, PlacedImage};
use crate::sys::kitty_graphics::{self, PendingKittyImage};
use crate::sys::parser::AnsiExecutor;
use crate::sys::pty::PtyCommand;
use crate::ui::components;

use super::super::helpers::{command_history_path, os_name};
use super::super::message::Message;
use super::super::nova::{AI_OPEN, Nova};

fn process_kitty_apc(
  content: &[u8],
  grid: &mut crate::core::grid::Grid,
  pending: &mut Option<PendingKittyImage>,
  font_size: f32,
) {
  let Some(cmd) = kitty_graphics::parse_kitty_apc(content) else {
    return;
  };

  match cmd.action {
    b'T' | b't' => {
      if cmd.more {
        match pending {
          Some(p) => p.chunks.push(cmd.data.clone()),
          None => {
            *pending = Some(PendingKittyImage {
              format: cmd.format,
              width: cmd.width,
              height: cmd.height,
              id: cmd.id,
              chunks: vec![cmd.data.clone()],
            });
          }
        }
      } else {
        let prior = pending.take();
        let format = if cmd.format != 0 {
          cmd.format
        } else {
          prior.as_ref().map_or(32, |p| p.format)
        };
        let width = if cmd.width != 0 {
          cmd.width
        } else {
          prior.as_ref().map_or(0, |p| p.width)
        };
        let height = if cmd.height != 0 {
          cmd.height
        } else {
          prior.as_ref().map_or(0, |p| p.height)
        };
        let image_id = if cmd.id != 0 {
          cmd.id
        } else {
          prior.as_ref().map_or(0, |p| p.id)
        };
        let prior_chunks: Vec<Vec<u8>> = prior.map(|p| p.chunks).unwrap_or_default();

        if let Some((rgba, pw, ph)) =
          kitty_graphics::decode_kitty_image(&prior_chunks, &cmd.data, format, width, height)
        {
          let line_height = (font_size * 1.29).max(1.0);
          let term_rows = ((ph as f32 / line_height).ceil() as usize).max(1);

          if image_id != 0 {
            grid.images.retain(|i| i.id != image_id);
          }

          grid.images.push(PlacedImage {
            id: image_id,
            row: grid.cursor_y,
            col: grid.cursor_x,
            pixel_width: pw,
            pixel_height: ph,
            rgba,
          });

          let new_y = grid.cursor_y + term_rows;
          if new_y >= grid.rows {
            let scroll_n = new_y - grid.rows + 1;
            grid.scroll_up(scroll_n);
            grid.cursor_y = grid.rows.saturating_sub(1);
          } else {
            grid.cursor_y = new_y;
          }
          grid.cursor_x = 0;

          if cmd.quiet < 2 {
            let id_part = if image_id != 0 {
              format!(",i={}", image_id)
            } else {
              String::new()
            };
            let resp = format!("\x1b_Ga=T{};OK\x1b\\", id_part).into_bytes();
            grid.output_queue.push(resp);
          }
        }
      }
    }
    b'd' => {
      if cmd.id != 0 {
        grid.images.retain(|i| i.id != cmd.id);
      } else {
        grid.images.clear();
      }
    }
    _ => {}
  }
}

impl Nova {
  pub(super) fn handle_type_input(&mut self, bytes: Vec<u8>) -> iced::Task<Message> {
    if self.settings_open || self.command_palette_open || self.ai_overlay_open || self.ai_loading {
      return iced::Task::none();
    }

    let Some(active_tab) = self.tabs.get(self.active_index) else {
      return iced::Task::none();
    };

    if active_tab.active_pane_is_split {
      if active_tab
        .split
        .as_ref()
        .map(|s| s.waiting_after_exit)
        .unwrap_or(false)
      {
        return self.update(Message::CloseSplitPane);
      }
    } else if active_tab.waiting_after_exit {
      return self.update(Message::CloseActiveTab);
    }

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
          if active_tab.shell_at_prompt {
            active_tab.grid.push_command(&input);
            active_tab.shell_at_prompt = false;
          }
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
    let font_size = self.settings.theme.font.size;
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

      let mut vte_bytes = Vec::with_capacity(bytes.len());
      let mut completed_apcs: Vec<Vec<u8>> = Vec::new();
      for byte in &bytes {
        let (pass, apc) = split.apc_state.advance(*byte);
        vte_bytes.extend_from_slice(&pass);
        if let Some(content) = apc {
          completed_apcs.push(content);
        }
      }
      for apc_content in &completed_apcs {
        process_kitty_apc(
          apc_content,
          &mut split.grid,
          &mut split.pending_kitty,
          font_size,
        );
      }

      let mut executor = AnsiExecutor {
        grid: &mut split.grid,
        bell_pending: false,
      };
      for byte in vte_bytes {
        split.ansi_parser.advance(&mut executor, &[byte]);
      }
      while !split.grid.output_queue.is_empty() {
        let response = split.grid.output_queue.remove(0);
        if let Some(tx) = &split.pty_tx {
          let _ = tx.send_blocking(PtyCommand::Input(response));
        }
      }
      let mut split_clipboard_write: Option<String> = None;
      let mut split_request_clipboard = false;
      for cmd in split.grid.control_queue.drain(..) {
        match cmd {
          ControlCommand::CommandComplete(_) => {
            if let Some(start) = split.command_start.take() {
              let elapsed = split
                .last_pty_output
                .map(|last| last.duration_since(start))
                .unwrap_or_else(|| start.elapsed());
              split.last_command_elapsed = Some(elapsed);
              split.last_pty_output = None;
            }
          }
          ControlCommand::SetClipboard(text) => split_clipboard_write = Some(text),
          ControlCommand::RequestClipboard => split_request_clipboard = true,
          _ => {}
        }
      }
      let mut split_tasks: Vec<iced::Task<Message>> = Vec::new();
      if let Some(text) = split_clipboard_write {
        split_tasks.push(iced::clipboard::write(text));
      }
      if split_request_clipboard {
        let id = tab_id;
        split_tasks.push(iced::clipboard::read().map(move |t| Message::Osc52ReadResponse(id, t)));
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
      return if split_tasks.is_empty() {
        iced::Task::none()
      } else {
        iced::Task::batch(split_tasks)
      };
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

      let mut vte_bytes = Vec::with_capacity(bytes.len());
      let mut completed_apcs: Vec<Vec<u8>> = Vec::new();
      for byte in &bytes {
        let (pass, apc) = tab.apc_state.advance(*byte);
        vte_bytes.extend_from_slice(&pass);
        if let Some(content) = apc {
          completed_apcs.push(content);
        }
      }
      for apc_content in &completed_apcs {
        process_kitty_apc(
          apc_content,
          &mut tab.grid,
          &mut tab.pending_kitty,
          font_size,
        );
      }

      let mut executor = AnsiExecutor {
        grid: &mut tab.grid,
        bell_pending: false,
      };
      for byte in vte_bytes {
        tab.ansi_parser.advance(&mut executor, &[byte]);
      }
      let bell_fired = executor.bell_pending;

      let mut open_ask_ai = false;
      let mut open_explain_ai = false;
      let mut ai_preset: Option<std::sync::Arc<str>> = None;
      let mut clipboard_write: Option<String> = None;
      let mut request_clipboard = false;
      for cmd in tab.grid.control_queue.drain(..) {
        match cmd {
          ControlCommand::SetClipboard(text) => clipboard_write = Some(text),
          ControlCommand::RequestClipboard => request_clipboard = true,
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
            tab.shell_at_prompt = true;
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

      let mut extra_tasks: Vec<iced::Task<Message>> = Vec::new();
      if let Some(text) = clipboard_write {
        extra_tasks.push(iced::clipboard::write(text));
      }
      if request_clipboard {
        let id = tab_id;
        extra_tasks.push(iced::clipboard::read().map(move |t| Message::Osc52ReadResponse(id, t)));
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
          extra_tasks.push(focus_task);
          extra_tasks.push(self.update(Message::AiSubmit));
          return iced::Task::batch(extra_tasks);
        }
        if extra_tasks.is_empty() {
          return focus_task;
        }
        extra_tasks.push(focus_task);
        return iced::Task::batch(extra_tasks);
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
        let diag_task =
          iced::Task::perform(crate::core::ai::query(q), Message::DiagnosticBannerResponse);
        if extra_tasks.is_empty() {
          return diag_task;
        }
        extra_tasks.push(diag_task);
        return iced::Task::batch(extra_tasks);
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

      if extra_tasks.is_empty() {
        return iced::Task::none();
      }
      return iced::Task::batch(extra_tasks);
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
