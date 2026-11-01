mod ai;
mod input;
mod mouse;
mod palette;
mod search;
mod session;
mod settings;
mod tabs;
mod window;

use base64::Engine;
use iced::{Point, Size};
use std::sync::atomic::Ordering;
use std::time::Instant;

use crate::core::config;
use crate::sys::pty::PtyCommand;
use crate::ui::components;
use crate::ui::tab::Tab;
use crate::ui::theme::motion;

use super::helpers::{
  calc_grid, derive_available_shells, extract_selection, rebuild_runtime_theme,
};
use super::message::Message;
use super::nova::{KB_RECORDING, Nova, PALETTE_OPEN, SEARCH_OPEN, SETTINGS_OPEN, SettingsTab};

impl Default for Nova {
  fn default() -> Self {
    let available_shells = config::available_shells();
    let default_shell = available_shells.first().cloned().unwrap_or_else(|| {
      #[cfg(target_os = "windows")]
      {
        "powershell".to_string()
      }
      #[cfg(not(target_os = "windows"))]
      {
        std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string())
      }
    });
    let cfg = config::get();
    let (cols, rows) = calc_grid(1024.0, 768.0, cfg.theme.font.size, false);
    let mut nova = Self {
      tabs: vec![Tab::new(0, cols, rows, default_shell, {
        let cur = std::env::current_dir().ok();
        let exe_dir = std::env::current_exe()
          .ok()
          .and_then(|e| e.parent().map(|p| p.to_path_buf()));
        let in_install_dir = cur
          .as_ref()
          .zip(exe_dir.as_ref())
          .map(|(c, e)| c == e)
          .unwrap_or(false);
        if in_install_dir {
          dirs::home_dir()
            .and_then(|p| p.into_os_string().into_string().ok())
            .unwrap_or_default()
        } else {
          cur
            .and_then(|p| p.into_os_string().into_string().ok())
            .unwrap_or_default()
        }
      })],
      active_index: 0,
      next_tab_id: 1,
      window_id: None,
      window_focused: false,
      window_maximized: false,
      window_size: Size::new(1024.0, 768.0),
      cursor_position: Point::ORIGIN,
      selection_start: None,
      selection_end: None,
      is_selecting: false,
      ctrl_held: false,
      shift_held: false,
      alt_held: false,
      last_mouse_button: None,
      click_count: 0,
      last_click_time: Instant::now(),
      last_click_cell: None,
      hovered_url: None,
      hovered_link_span: None,
      shell_picker_open: false,
      shell_picker_anchor: 0.0,
      available_shells,
      settings_open: false,
      settings_tab: SettingsTab::General,
      settings: config::get().clone(),
      settings_shell_input: String::new(),
      settings_recording_index: None,
      raw_config_content: String::new(),
      command_palette_open: false,
      palette_query: String::new(),
      palette_selected: 0,
      ai_overlay_open: false,
      ai_input: String::new(),
      ai_selected: 0,
      ai_answer: None,
      diagnostic_banner: None,
      ai_pending_diagnostic: None,
      bell_blink_visible: true,
      bell_blink_remaining: 0,
      cursor_blink_visible: true,
      resize_generation: 0,
      font_resize_generation: 0,
      default_font_size: cfg.theme.font.size,
      dragging_split: false,
      search_active: false,
      search_query: String::new(),
      search_matches: Vec::new(),
      block_matches: Vec::new(),
      search_match_index: 0,
      search_generation: 0,
      dragging_tab_index: None,
      hovered_tab: None,
      drag_pending_tab: None,
      drag_pending_pos: None,
      paste_warning: None,
      now: Instant::now(),
      sheet_opened_at: Instant::now(),
      timestamps: false,
      block_selection: None,
      block_pointer: None,
      block_url: None,
      window_pop: motion::slow(false),
    };
    nova.load_command_history();
    nova
  }
}

fn is_dangerous_paste(text: &str) -> bool {
  text.contains('\n')
}

impl Nova {
  fn do_paste(&mut self, text: String) {
    if let Some(tab) = self.tabs.get_mut(self.active_index) {
      let pane = tab.active_pane_mut();
      if pane.integration.is_blocks() && !pane.session.is_running() {
        let joined = components::join_for_input(&text);
        pane.input.text.push_str(&joined);
        return;
      }
    }
    let Some(tab) = self.tabs.get(self.active_index) else {
      return;
    };
    let (tx, bracketed) = if tab.active_pane_is_split {
      if let Some(split) = &tab.split {
        (split.pty_tx.as_ref(), split.grid.bracketed_paste)
      } else {
        return;
      }
    } else {
      (tab.pty_tx.as_ref(), tab.grid.bracketed_paste)
    };
    if let Some(tx) = tx {
      if bracketed {
        let mut payload = Vec::with_capacity(text.len() + 12);
        payload.extend_from_slice(b"\x1b[200~");
        payload.extend_from_slice(text.as_bytes());
        payload.extend_from_slice(b"\x1b[201~");
        let _ = tx.try_send(PtyCommand::Input(payload));
      } else {
        let _ = tx.try_send(PtyCommand::Input(text.into_bytes()));
      }
    }
  }

  pub fn update(&mut self, message: Message) -> iced::Task<Message> {
    match message {
      Message::Type(bytes) => self.handle_type_input(bytes),
      Message::PtyOutputReceived(tab_id, bytes) => {
        let before = self.block_mode_panes();
        let raw_before = self.raw_panes();
        let task = self.handle_pty_output(tab_id, bytes);
        let mut tasks = vec![task, self.follow_output(tab_id)];
        if self.block_mode_panes() != before {
          self.resize_all_grids();
          tasks.push(self.focus_input());
        } else if self.raw_panes() < raw_before {
          tasks.push(self.focus_input());
        }
        iced::Task::batch(tasks)
      }
      Message::CursorMoved(position) => {
        self.handle_cursor_moved(position);
        iced::Task::none()
      }
      Message::MousePressed(button) => self.handle_mouse_pressed(button),
      Message::MouseReleased(button) => self.handle_mouse_released(button),
      Message::Scroll(delta) => {
        self.handle_scroll(delta);
        iced::Task::none()
      }
      Message::WindowResized(width, height) => {
        self.window_size = iced::Size::new(width, height);
        self.resize_generation = self.resize_generation.wrapping_add(1);
        let epoch = self.resize_generation;
        iced::Task::perform(
          async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            epoch
          },
          Message::ResizeSettled,
        )
      }
      Message::ResizeSettled(epoch) => {
        if epoch == self.resize_generation {
          let w = self.window_size.width;
          let h = self.window_size.height;
          self.handle_window_resized(w, h)
        } else {
          iced::Task::none()
        }
      }
      Message::AiSubmit => {
        let task = self.open_ai_sheet();
        iced::Task::batch([task, self.start_ai(components::AiKind::Generate)])
      }
      Message::ExplainError => {
        let task = self.open_ai_sheet();
        let kind = if self
          .ai_items()
          .iter()
          .any(|i| i.kind == components::AiKind::Diagnose)
        {
          components::AiKind::Diagnose
        } else {
          components::AiKind::Explain
        };
        iced::Task::batch([task, self.start_ai(kind)])
      }
      Message::DiagnosticBannerResponse(result) => {
        self.handle_diagnostic_banner_response(result);
        iced::Task::none()
      }
      Message::PaletteConfirm => {
        self.update(Message::PaletteSelectAndConfirm(self.palette_selected))
      }
      Message::PaletteSelectAndConfirm(i) => self.handle_palette_select_confirm(i),
      Message::SettingsColorChanged(field, hex) => {
        self.handle_settings_color_changed(field, hex);
        iced::Task::none()
      }
      Message::SettingsRecordKb { key, modifiers } => {
        self.handle_settings_record_kb(key, modifiers);
        iced::Task::none()
      }
      Message::SettingsResetKb(idx) => {
        self.handle_settings_reset_kb(idx);
        iced::Task::none()
      }
      Message::CloseTab(index) => {
        self.handle_close_tab(index);
        self.hovered_tab = None;
        iced::Task::none()
      }
      Message::TabPressed(index) => {
        if index < self.tabs.len() {
          self.active_index = index;
          self.drag_pending_tab = Some(index);
          self.drag_pending_pos = Some(self.cursor_position);
        }
        iced::Task::none()
      }
      Message::TabHovered(hovered) => {
        self.hovered_tab = hovered;
        if let (Some(from), Some(to)) = (self.dragging_tab_index, hovered)
          && let Some((from, to)) = components::reorder(self.tabs.len(), from, to)
        {
          let tab = self.tabs.remove(from);
          self.tabs.insert(to, tab);
          self.active_index = to;
          self.dragging_tab_index = Some(to);
        }
        iced::Task::none()
      }
      Message::NewTab => {
        let shell = self.available_shells.first().cloned().unwrap_or_default();
        self.update(Message::NewTabWithShell(shell))
      }
      Message::OpenShellPicker => {
        if self.available_shells.len() <= 1 {
          let shell = self.available_shells.first().cloned().unwrap_or_default();
          return self.update(Message::NewTabWithShell(shell));
        }
        self.shell_picker_anchor = self.cursor_position.x;
        self.shell_picker_open = true;
        iced::Task::none()
      }
      Message::CloseShellPicker => {
        self.shell_picker_open = false;
        iced::Task::none()
      }
      Message::NewTabWithShell(shell) => {
        self.shell_picker_open = false;
        let new_id = self.next_tab_id;
        self.next_tab_id += 1;
        let (cols, rows) = calc_grid(
          self.window_size.width,
          self.window_size.height,
          self.settings.theme.font.size,
          self.diagnostic_banner.is_some(),
        );
        let parent_pwd = self
          .tabs
          .get(self.active_index)
          .map(|t| t.pwd.clone())
          .unwrap_or_default();
        let history = self.active_history();
        let mut tab = Tab::new(new_id, cols, rows, shell, parent_pwd);
        tab.session.history = history;
        self.tabs.push(tab);
        self.active_index = self.tabs.len() - 1;
        iced::Task::none()
      }
      Message::SplitPane => self.handle_split_pane(),
      Message::CloseSplitPane => {
        self.handle_close_split_pane();
        iced::Task::none()
      }
      Message::CloseLeftPane => {
        self.handle_close_left_pane();
        iced::Task::none()
      }
      Message::CloseActiveTab => {
        if let Some(tab) = self.tabs.get(self.active_index)
          && tab.split.is_some()
        {
          let msg = if tab.active_pane_is_split {
            Message::CloseSplitPane
          } else {
            Message::CloseLeftPane
          };
          return self.update(msg);
        }
        self.update(Message::CloseTab(self.active_index))
      }
      Message::NextTab => {
        if !self.tabs.is_empty() {
          self.active_index = (self.active_index + 1) % self.tabs.len();
        }
        iced::Task::none()
      }
      Message::PrevTab => {
        if !self.tabs.is_empty() {
          self.active_index = if self.active_index == 0 {
            self.tabs.len() - 1
          } else {
            self.active_index - 1
          };
        }
        iced::Task::none()
      }
      Message::PtyReady(tab_id, tx) => {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
          if let Some(cmd) = tab.pending_command.take() {
            let _ = tx.send_blocking(PtyCommand::Input(cmd));
          }
          tab.pty_tx = Some(tx);
          tab.pty_alive = true;
          return iced::Task::none();
        }
        for tab in &mut self.tabs {
          if let Some(split) = &mut tab.split
            && split.id == tab_id
          {
            split.pty_tx = Some(tx);
            split.pty_alive = true;
            return iced::Task::none();
          }
        }
        iced::Task::none()
      }
      Message::PtyExited(tab_id) => {
        let wait = config::get().general.wait_after_command;
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
          tab.pty_alive = false;
          tab.pty_tx = None;
          if wait {
            tab.waiting_after_exit = true;
          }
          return iced::Task::none();
        }
        for tab in &mut self.tabs {
          if let Some(split) = &mut tab.split
            && split.id == tab_id
          {
            split.pty_alive = false;
            split.pty_tx = None;
            if wait {
              split.waiting_after_exit = true;
            }
            return iced::Task::none();
          }
        }
        iced::Task::none()
      }
      Message::OpenSettings => {
        self.settings_open = true;
        self.settings_tab = SettingsTab::General;
        self.settings_shell_input = String::new();
        self.settings_recording_index = None;
        SETTINGS_OPEN.store(true, Ordering::SeqCst);
        iced::Task::none()
      }
      Message::CloseSettings => {
        self.settings_open = false;
        self.settings_recording_index = None;
        KB_RECORDING.store(false, Ordering::SeqCst);
        SETTINGS_OPEN.store(false, Ordering::SeqCst);
        iced::Task::none()
      }
      Message::SettingsTabSelected(tab) => {
        if tab == SettingsTab::Raw {
          self.raw_config_content = toml::to_string_pretty(&self.settings)
            .unwrap_or_else(|_| String::from("error serializing config"));
        }
        self.settings_tab = tab;
        iced::Task::none()
      }
      Message::SettingsEditorChanged(s) => {
        self.settings.general.editor = s;
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::SettingsBellChanged(bell) => {
        self.settings.general.bell = bell;
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::SettingsShellInputChanged(s) => {
        self.settings_shell_input = s;
        iced::Task::none()
      }
      Message::SettingsAddShell => {
        let s = self.settings_shell_input.trim().to_string();
        if !s.is_empty() {
          self
            .settings
            .general
            .shells
            .get_or_insert_with(Vec::new)
            .push(s);
          self.settings_shell_input = String::new();
          let _ = config::save(&self.settings);
          self.available_shells = derive_available_shells(&self.settings);
        }
        iced::Task::none()
      }
      Message::SettingsRemoveShell(i) => {
        if let Some(shells) = &mut self.settings.general.shells
          && i < shells.len()
        {
          shells.remove(i);
        }
        let _ = config::save(&self.settings);
        self.available_shells = derive_available_shells(&self.settings);
        iced::Task::none()
      }
      Message::SettingsFontFamilyChanged(s) => {
        crate::ui::theme::font::set_family(&s);
        self.settings.theme.font.family = s;
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::FontSizeUp => {
        let cur = self.settings.theme.font.size.round() as i32;
        let next = if cur % 2 == 0 { cur + 2 } else { cur + 1 };
        self.settings.theme.font.size = (next as f32).clamp(8.0, 72.0);
        self.debounce_font_resize()
      }
      Message::FontSizeDown => {
        let cur = self.settings.theme.font.size.round() as i32;
        let next = if cur % 2 == 0 { cur - 2 } else { cur - 1 };
        self.settings.theme.font.size = (next as f32).clamp(8.0, 72.0);
        self.debounce_font_resize()
      }
      Message::FontSizeReset => {
        self.settings.theme.font.size = self.default_font_size;
        self.debounce_font_resize()
      }
      Message::FontResizeSettled(epoch) => {
        if epoch == self.font_resize_generation {
          self.resize_all_grids();
        }
        iced::Task::none()
      }
      Message::SettingsFontSizeChanged(size) => {
        let size = size.clamp(8.0, 72.0);
        self.settings.theme.font.size = size;
        self.default_font_size = size;
        let _ = config::save(&self.settings);
        self.debounce_font_resize()
      }
      Message::SettingsStartRecordKb(idx) => {
        self.settings_recording_index = Some(idx);
        KB_RECORDING.store(true, Ordering::SeqCst);
        iced::Task::none()
      }
      Message::SettingsCancelRecordKb => {
        self.settings_recording_index = None;
        KB_RECORDING.store(false, Ordering::SeqCst);
        iced::Task::none()
      }
      Message::SettingsResetAll => {
        if let Ok(cfg) = config::reset_to_defaults() {
          self.settings = cfg;
          rebuild_runtime_theme(&self.settings.theme.colors);
          self.available_shells = derive_available_shells(&self.settings);
          let _ = config::reload_parsed_keybindings(&self.settings);
          self.resize_all_grids();
        }
        iced::Task::none()
      }
      Message::WindowOpened(id) => {
        self.window_id = Some(id);
        self.now = Instant::now();
        self.window_pop.go_mut(true, self.now);
        #[cfg(target_os = "windows")]
        {
          iced::window::set_mode(id, iced::window::Mode::Windowed)
        }
        #[cfg(not(target_os = "windows"))]
        iced::Task::none()
      }
      Message::MinimizeWindow => {
        if let Some(window_id) = self.window_id {
          return iced::window::minimize(window_id, true);
        }
        iced::Task::none()
      }
      Message::MaximizeWindow => {
        if let Some(window_id) = self.window_id {
          self.window_maximized = !self.window_maximized;
          return iced::window::toggle_maximize(window_id);
        }
        iced::Task::none()
      }
      Message::CloseWindow => {
        std::process::exit(0);
      }
      Message::DragWindow => {
        if let Some(window_id) = self.window_id {
          return iced::window::drag(window_id);
        }
        iced::Task::none()
      }
      Message::WindowFocused => {
        self.window_focused = true;
        self.focus_input()
      }
      Message::WindowUnfocused => {
        self.window_focused = false;
        iced::Task::none()
      }
      Message::WindowMaximizedState(maximized) => {
        self.window_maximized = maximized;
        iced::Task::none()
      }
      Message::OpenCommandPalette => {
        if self.command_palette_open {
          self.close_palette();
          return self.focus_input();
        }
        self.sheet_opened_at = Instant::now();
        self.now = self.sheet_opened_at;
        self.command_palette_open = true;
        self.palette_query = String::new();
        self.palette_selected = 0;
        PALETTE_OPEN.store(true, Ordering::SeqCst);
        iced::widget::operation::focus(components::PALETTE_INPUT_ID.clone())
      }
      Message::CloseCommandPalette => {
        self.close_palette();
        self.focus_input()
      }
      Message::PaletteHover(i) => {
        self.palette_selected = i;
        iced::Task::none()
      }
      Message::ToggleTimestamps => {
        self.timestamps = !self.timestamps;
        self.resize_all_grids();
        iced::Task::none()
      }
      Message::PaletteQueryChanged(s) => {
        self.palette_query = s;
        self.palette_selected = 0;
        iced::Task::none()
      }
      Message::PaletteNavigate(d) => {
        let count = components::palette_filtered_count(&self.palette_query);
        self.palette_selected = components::clamp_step(self.palette_selected, d, count);
        iced::Task::none()
      }
      Message::OpenAskAi => {
        if self.ai_overlay_open {
          return self.close_ai_sheet();
        }
        self.ai_input.clear();
        self.open_ai_sheet()
      }
      Message::CloseAiOverlay => self.close_ai_sheet(),
      Message::AiOverlayInputChanged(s) => {
        self.ai_input = s;
        self.ai_selected = 0;
        iced::Task::none()
      }
      Message::AiResponseReceived(result) => self.handle_ai_response(result),
      Message::AiHover(i) => {
        self.ai_selected = i;
        iced::Task::none()
      }
      Message::AiPick(i) => self.handle_ai_pick(i),
      Message::AiConfirm => {
        if self.ai_answer.is_none() {
          return self.handle_ai_pick(self.ai_selected);
        }
        iced::Task::none()
      }
      Message::AiNavigate(d) => {
        if self.ai_answer.is_none() {
          let count = self.ai_items().len();
          self.ai_selected = components::clamp_step(self.ai_selected, d, count);
        }
        iced::Task::none()
      }
      Message::AiEscape => self.handle_ai_escape(),
      Message::AiBack => self.handle_ai_escape(),
      Message::AiEnter => {
        if self.ai_answer.is_some() {
          return self.handle_ai_run();
        }
        iced::Task::none()
      }
      Message::AiRunSuggestion => self.handle_ai_run(),
      Message::RunCommand(cmd) => self.handle_run_command(cmd),
      Message::DiagnosticBannerCommand(cmd) => {
        self.diagnostic_banner = None;
        self.handle_run_command(cmd)
      }
      Message::SettingsDiagnosticBannerToggled(enabled) => {
        if !enabled {
          self.diagnostic_banner = None;
        }
        self.settings.ai.diagnostic_banner = enabled;
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::SettingsAiProviderChanged(provider) => {
        self.settings.ai.provider = provider;
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::SettingsAiModelChanged(s) => {
        self.settings.ai.model = s;
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::SettingsAiApiKeyChanged(s) => {
        self.settings.ai.api_key = s;
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::SettingsWindowControlsChanged(style) => {
        self.settings.general.window_controls = style;
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::SettingsCursorStyleChanged(style) => {
        self.settings.theme.cursor.style = style;
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::SettingsCursorBlinkToggled(enabled) => {
        self.settings.theme.cursor.blink = enabled;
        if !enabled {
          self.cursor_blink_visible = true;
        }
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::SettingsInitialCommandChanged(s) => {
        self.settings.general.initial_command = if s.trim().is_empty() { None } else { Some(s) };
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::SettingsWaitAfterCommandToggled(enabled) => {
        self.settings.general.wait_after_command = enabled;
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::SettingsAiBaseUrlChanged(s) => {
        self.settings.ai.base_url = if s.trim().is_empty() { None } else { Some(s) };
        let _ = config::save(&self.settings);
        iced::Task::none()
      }
      Message::BellBlinkTick => {
        if self.bell_blink_remaining > 0 {
          self.bell_blink_remaining -= 1;
          self.bell_blink_visible = !self.bell_blink_visible;
          if self.bell_blink_remaining == 0 {
            self.bell_blink_visible = true;
          }
        }
        iced::Task::none()
      }
      Message::CursorBlinkTick => {
        if self.settings.theme.cursor.blink {
          self.cursor_blink_visible = !self.cursor_blink_visible;
        } else {
          self.cursor_blink_visible = true;
        }
        iced::Task::none()
      }
      Message::SearchOpen => {
        if !self.search_active {
          self.search_active = true;
          self.search_query.clear();
          self.search_matches.clear();
          self.search_match_index = 0;
          SEARCH_OPEN.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        iced::widget::operation::focus(components::SEARCH_INPUT_ID.clone())
      }
      Message::SearchClose => {
        self.search_active = false;
        self.search_query.clear();
        self.search_matches.clear();
        self.search_match_index = 0;
        SEARCH_OPEN.store(false, std::sync::atomic::Ordering::SeqCst);
        iced::Task::none()
      }
      Message::SearchQueryChanged(q) => {
        self.search_query = q;
        self.search_generation = self.search_generation.wrapping_add(1);
        let search_gen = self.search_generation;
        iced::Task::perform(
          async move {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            search_gen
          },
          Message::SearchDebounced,
        )
      }
      Message::SearchDebounced(search_gen) => {
        if search_gen == self.search_generation {
          self.recompute_search();
          return self.scroll_to_block_match();
        }
        iced::Task::none()
      }
      Message::SearchNext => {
        let count = self.search_match_count();
        if count > 0 {
          self.search_match_index = (self.search_match_index + 1) % count;
          self.scroll_to_search_match();
        }
        self.scroll_to_block_match()
      }
      Message::SearchPrev => {
        let count = self.search_match_count();
        if count > 0 {
          self.search_match_index = self.search_match_index.checked_sub(1).unwrap_or(count - 1);
          self.scroll_to_search_match();
        }
        self.scroll_to_block_match()
      }
      Message::NotificationActivated => {
        if let Some(id) = self.window_id {
          return iced::window::gain_focus(id);
        }
        iced::Task::none()
      }
      Message::Tick => {
        self.expire_integrations(Instant::now());
        iced::Task::none()
      }
      Message::InputChanged(text) => self.handle_input_changed(text),
      Message::BlocksScrolled(pane_id, pinned) => self.handle_blocks_scrolled(pane_id, pinned),
      Message::FocusPane(pane_id) => {
        if let Some(tab) = self.tabs.get_mut(self.active_index) {
          if tab.split.as_ref().is_some_and(|s| s.id == pane_id) {
            tab.active_pane_is_split = true;
          } else if tab.id == pane_id {
            tab.active_pane_is_split = false;
          }
        }
        self.focus_input()
      }
      Message::BlockPointer(pane_id, block_id, p) => {
        self.handle_block_pointer(pane_id, block_id, p)
      }
      Message::BlockPress(pane_id, block_id) => self.handle_block_press(pane_id, block_id),
      Message::BlockRelease => self.handle_block_release(),
      Message::InputSubmit => self.handle_input_submit(),
      Message::InputHistory(delta) => self.handle_input_history(delta),
      Message::ClearBlocks => self.handle_clear_blocks(),
      Message::Osc52ReadResponse(target_id, text) => {
        let text = text.unwrap_or_default();
        let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
        let response = format!("\x1b]52;c;{}\x07", encoded).into_bytes();
        for tab in &self.tabs {
          if tab.id == target_id {
            if let Some(tx) = &tab.pty_tx {
              let _ = tx.send_blocking(PtyCommand::Input(response));
            }
            break;
          }
          if let Some(split) = &tab.split
            && split.id == target_id
          {
            if let Some(tx) = &split.pty_tx {
              let _ = tx.send_blocking(PtyCommand::Input(response));
            }
            break;
          }
        }
        iced::Task::none()
      }
      Message::NoOp => iced::Task::none(),
      Message::Frame(now) => {
        self.now = now;
        iced::Task::none()
      }
      Message::ClipboardReceived(text) => {
        if let Some(text) = text {
          if is_dangerous_paste(&text) {
            self.sheet_opened_at = Instant::now();
            self.now = self.sheet_opened_at;
            self.paste_warning = Some(text);
          } else {
            self.do_paste(text);
          }
        }
        iced::Task::none()
      }
      Message::PasteWarningConfirm => {
        if let Some(text) = self.paste_warning.take() {
          self.do_paste(text);
        }
        self.focus_input()
      }
      Message::PasteWarningCancel => {
        self.paste_warning = None;
        self.focus_input()
      }
      Message::PasteRequested => iced::clipboard::read().map(Message::ClipboardReceived),
      Message::CopySelection => {
        if let Some(text) = self.block_selection_text() {
          return iced::clipboard::write(text);
        }
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end)
          && let Some(active_tab) = self.tabs.get(self.active_index)
        {
          let text = extract_selection(&active_tab.grid, active_tab.scroll_offset, start, end);
          if !text.is_empty() {
            return iced::clipboard::write(text);
          }
        }
        iced::Task::none()
      }
      Message::ModifiersChanged(mods) => {
        self.ctrl_held = mods.command();
        self.shift_held = mods.shift();
        self.alt_held = mods.alt();
        self.update_hovered_url();
        iced::Task::none()
      }
    }
  }
}
