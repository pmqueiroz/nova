use async_channel::Sender;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::{button, column, container, mouse_area, stack, text};
use iced::{
  Border, Color, Element, Length, Padding, Point, Size, Subscription, Theme, border::Radius, time,
  window,
};
use iced::{Event, event, keyboard, mouse, stream};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crate::core::config::{self, KeyId, ParsedKeybinding};
use crate::core::grid::ControlCommand;
use crate::sys::parser::AnsiExecutor;
use crate::sys::pty::{PtyBridge, PtyCommand};
use crate::ui::components;
use crate::ui::tab::Tab;
use crate::ui::theme;

pub static SETTINGS_OPEN: AtomicBool = AtomicBool::new(false);
pub static KB_RECORDING: AtomicBool = AtomicBool::new(false);
pub static PALETTE_OPEN: AtomicBool = AtomicBool::new(false);
pub static AI_OPEN: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
  General,
  Theme,
  Keybindings,
  StatusBar,
  Ai,
  Raw,
}

#[derive(Debug, Clone)]
pub enum ColorField {
  Background,
  Foreground,
  Accent,
  ForegroundMuted,
  Border,
  Cursor,
}

pub struct Nova {
  tabs: Vec<Tab>,
  active_index: usize,
  next_tab_id: usize,
  window_id: Option<window::Id>,
  window_focused: bool,
  window_maximized: bool,
  window_size: Size,
  cursor_position: Point,
  selection_start: Option<(usize, usize)>,
  selection_end: Option<(usize, usize)>,
  is_selecting: bool,
  ctrl_held: bool,
  shift_held: bool,
  alt_held: bool,
  last_mouse_button: Option<mouse::Button>,
  click_count: u8,
  last_click_time: Instant,
  last_click_cell: Option<(usize, usize)>,
  hovered_url: Option<String>,
  hovered_link_span: Option<(usize, usize, usize)>, // (start_display_row, start_col, end_display_row)
  shell_picker_open: bool,
  shell_picker_anchor: f32,
  available_shells: Vec<String>,
  settings_open: bool,
  settings_tab: SettingsTab,
  settings: config::Config,
  settings_shell_input: String,
  settings_recording_index: Option<usize>,
  raw_config_content: String,
  command_palette_open: bool,
  palette_query: String,
  palette_selected: usize,
  ai_overlay_open: bool,
  ai_input: String,
  ai_loading: bool,
  ai_response: Option<String>,
  ai_is_error: bool,
  diagnostic_banner: Option<(u8, String, Option<String>)>,
  ai_pending_diagnostic: Option<u8>,
  bell_blink_visible: bool,
  bell_blink_remaining: u8,
}

#[derive(Debug, Clone)]
pub enum Message {
  Type(Vec<u8>),
  NewTab,
  SwitchTab(usize),
  CloseTab(usize),
  PtyReady(usize, Sender<PtyCommand>),
  PtyOutputReceived(usize, Vec<u8>),
  PtyExited(usize),
  CloseActiveTab,
  NextTab,
  PrevTab,
  CloseWindow,
  MinimizeWindow,
  MaximizeWindow,
  DragWindow,
  WindowOpened(window::Id),
  WindowFocused,
  WindowUnfocused,
  WindowResized(f32, f32),
  WindowMaximizedState(bool),
  PasteRequested,
  ClipboardReceived(Option<String>),
  OpenSettings,
  OpenShellPicker,
  CloseShellPicker,
  NewTabWithShell(String),
  CursorMoved(Point),
  MousePressed(mouse::Button),
  MouseReleased(mouse::Button),
  CopySelection,
  Tick,
  BellBlinkTick,
  CloseSettings,
  SettingsTabSelected(SettingsTab),
  SettingsEditorChanged(String),
  SettingsBellChanged(config::BellType),
  SettingsShellInputChanged(String),
  SettingsAddShell,
  SettingsRemoveShell(usize),
  SettingsFontFamilyChanged(String),
  SettingsFontSizeChanged(f32),
  SettingsColorChanged(ColorField, String),
  SettingsStatusBarToggled(bool),
  SettingsDateFormatChanged(String),
  SettingsTimeFormatChanged(String),
  SettingsStartRecordKb(usize),
  SettingsRecordKb {
    key: keyboard::Key,
    modifiers: keyboard::Modifiers,
  },
  SettingsCancelRecordKb,
  SettingsResetKb(usize),
  SettingsResetAll,
  Scroll(f32),
  ModifiersChanged(keyboard::Modifiers),
  // Command palette
  OpenCommandPalette,
  CloseCommandPalette,
  PaletteQueryChanged(String),
  PaletteNavigate(i32),
  PaletteConfirm,
  PaletteSelectAndConfirm(usize),
  // AI overlay
  OpenAskAi,
  CloseAiOverlay,
  AiOverlayInputChanged(String),
  AiSubmit,
  AiResponseReceived(Result<String, String>),
  ExplainError,
  CopyCodeBlock(String),
  RunCodeInTerminal(String),
  // Settings AI tab
  SettingsAiProviderChanged(config::AiProvider),
  SettingsAiModelChanged(String),
  SettingsAiApiKeyChanged(String),
  SettingsAiBaseUrlChanged(String),
  SettingsWindowControlsChanged(config::WindowControls),
  DiagnosticBannerResponse(Result<String, String>),
  DiagnosticBannerCommand(String),
  SettingsDiagnosticBannerToggled(bool),
  NoOp,
}

fn get_display_row(
  grid: &crate::core::grid::Grid,
  scroll_offset: usize,
  y: usize,
) -> Option<&[crate::core::grid::Cell]> {
  let sb_len = grid.scrollback.len();
  let clamped = scroll_offset.min(sb_len);
  if y < clamped {
    grid
      .scrollback
      .get(sb_len - clamped + y)
      .map(|(r, _)| r.as_slice())
  } else if y - clamped < grid.rows {
    Some(grid.row(y - clamped))
  } else {
    None
  }
}

fn stitch_continuation(
  base: String,
  grid: &crate::core::grid::Grid,
  scroll_offset: usize,
  start_row: usize,
) -> (String, usize) {
  let mut url = base;
  let mut end_row = start_row.saturating_sub(1);
  let mut r = start_row;
  while let Some(cells) = get_display_row(grid, scroll_offset, r) {
    let cont = crate::core::url::url_continuation_len(cells);
    if cont == 0 {
      break;
    }
    url.extend(cells[..cont].iter().map(|c| c.c));
    end_row = r;
    if cont < cells.len() {
      break;
    }
    r += 1;
  }
  (url, end_row)
}

fn pixel_to_cell(pos: Point, font_size: f32) -> Option<(usize, usize)> {
  let x_origin = 20.0_f32;
  let y_origin = 88.0_f32;
  if pos.y < y_origin || pos.x < x_origin {
    return None;
  }
  let col = ((pos.x - x_origin) / (font_size * 0.62)).floor() as usize;
  let row = ((pos.y - y_origin) / (font_size * 1.29)).floor() as usize;
  Some((col, row))
}

fn normalize_sel(start: (usize, usize), end: (usize, usize)) -> ((usize, usize), (usize, usize)) {
  let (sc, sr) = start;
  let (ec, er) = end;
  if sr < er || (sr == er && sc <= ec) {
    (start, end)
  } else {
    (end, start)
  }
}

fn find_word_boundaries(row_cells: &[crate::core::grid::Cell], col: usize) -> (usize, usize) {
  let is_word = |c: char| c.is_alphanumeric() || c == '_';
  let col = col.min(row_cells.len().saturating_sub(1));
  let clicked_is_word = row_cells.get(col).map(|c| is_word(c.c)).unwrap_or(false);

  let start = (0..=col)
    .rev()
    .find(|&i| {
      let c = row_cells.get(i).map(|c| c.c).unwrap_or(' ');
      is_word(c) != clicked_is_word
    })
    .map(|i| i + 1)
    .unwrap_or(0);

  let end = (col..row_cells.len())
    .find(|&i| {
      let c = row_cells.get(i).map(|c| c.c).unwrap_or(' ');
      is_word(c) != clicked_is_word
    })
    .map(|i| i.saturating_sub(1))
    .unwrap_or(row_cells.len().saturating_sub(1));

  (start, end.max(start))
}

fn extract_selection(
  grid: &crate::core::grid::Grid,
  scroll_offset: usize,
  start: (usize, usize),
  end: (usize, usize),
) -> String {
  let ((sc, sr), (ec, er)) = normalize_sel(start, end);
  if sr == er && sc == ec {
    return String::new();
  }
  let clamped = scroll_offset.min(grid.scrollback.len());
  let max_display = clamped.saturating_add(grid.rows).saturating_sub(1);
  let sr = sr.min(max_display);
  let er = er.min(max_display);
  let mut result = String::new();
  for row in sr..=er {
    let Some(row_cells) = get_display_row(grid, scroll_offset, row) else {
      continue;
    };
    let col_start = if row == sr {
      sc.min(grid.cols.saturating_sub(1))
    } else {
      0
    };
    let col_end = if row == er {
      ec.min(grid.cols.saturating_sub(1))
    } else {
      grid.cols.saturating_sub(1)
    };
    for cell in row_cells.iter().take(col_end + 1).skip(col_start) {
      result.push(cell.c);
    }
    if row < er && !grid.row_continuation[row + 1] {
      result.push('\n');
    }
  }
  result.trim_end().to_string()
}

fn calc_grid(
  width: f32,
  height: f32,
  font_size: f32,
  status_bar_visible: bool,
  banner_visible: bool,
) -> (usize, usize) {
  let char_width = font_size * 0.62;
  let char_height = font_size * 1.29;
  let banner_extra = if banner_visible { font_size * 2.5 } else { 0.0 };
  let padding_y = if status_bar_visible { 118.0 } else { 96.0 } + banner_extra;
  let cols = ((width - 40.0) / char_width).floor() as usize;
  let rows = ((height - padding_y) / char_height).floor() as usize;
  (cols.max(10), rows.max(5))
}

fn matches_kb(kb: &ParsedKeybinding, key: &Key, mods: keyboard::Modifiers) -> bool {
  if kb.ctrl != mods.control()
    || kb.shift != mods.shift()
    || kb.alt != mods.alt()
    || kb.meta != mods.logo()
  {
    return false;
  }
  match (&kb.key, key) {
    (KeyId::Tab, Key::Named(Named::Tab)) => true,
    (KeyId::Char(c), Key::Character(s)) => s
      .as_str()
      .chars()
      .next()
      .map(|sc| sc == *c)
      .unwrap_or(false),
    _ => false,
  }
}

fn keybinding_to_string(key: &Key, mods: keyboard::Modifiers) -> Option<String> {
  let mut parts: Vec<&str> = vec![];
  if mods.control() {
    parts.push("ctrl");
  }
  if mods.shift() {
    parts.push("shift");
  }
  if mods.alt() {
    parts.push("alt");
  }
  if mods.logo() {
    parts.push("cmd");
  }
  match key {
    Key::Named(Named::Tab) => parts.push("tab"),
    Key::Character(s) => {
      let lower = s.as_str().to_ascii_lowercase();
      let leaked: &'static str = Box::leak(lower.into_boxed_str());
      parts.push(leaked);
    }
    _ => return None,
  }
  Some(parts.join("+"))
}

fn derive_available_shells(settings: &config::Config) -> Vec<String> {
  if let Some(shells) = &settings.general.shells
    && !shells.is_empty()
  {
    return shells.clone();
  }
  config::detect_shells()
}

fn rebuild_runtime_theme(colors: &config::ThemeColorsConfig) {
  use crate::ui::theme::color::{RuntimeTheme, update_runtime};
  let parse = |h: &str| config::parse_hex_color(h).unwrap_or(iced::Color::BLACK);
  update_runtime(RuntimeTheme {
    background: parse(&colors.background),
    foreground: parse(&colors.foreground),
    accent: parse(&colors.accent),
    foreground_muted: parse(&colors.foreground_muted),
    border: parse(&colors.border),
    cursor: parse(&colors.cursor),
  });
}

fn command_history_path() -> Option<PathBuf> {
  let data_dir = dirs::data_dir()?;
  let dir = data_dir.join("nova");
  let _ = std::fs::create_dir_all(&dir);
  Some(dir.join("command_history.bin"))
}

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
    let (cols, rows) = calc_grid(
      1024.0,
      768.0,
      cfg.theme.font.size,
      cfg.status_bar.visible,
      false,
    );
    let mut nova = Self {
      tabs: vec![Tab::new(0, cols, rows, default_shell, String::new())],
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
      ai_loading: false,
      ai_response: None,
      ai_is_error: false,
      diagnostic_banner: None,
      ai_pending_diagnostic: None,
      bell_blink_visible: true,
      bell_blink_remaining: 0,
    };
    nova.load_command_history();
    nova
  }
}

impl Nova {
  fn load_command_history(&mut self) {
    let Some(path) = command_history_path() else {
      return;
    };
    let Ok(data) = std::fs::read(&path) else {
      return;
    };
    let Ok(history) =
      bincode::deserialize::<std::collections::VecDeque<crate::core::grid::CommandEntry>>(&data)
    else {
      return;
    };
    for tab in self.tabs.iter_mut() {
      tab.grid.command_history = history.clone();
    }
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

  fn update_hovered_url(&mut self) {
    if !self.ctrl_held {
      self.hovered_url = None;
      self.hovered_link_span = None;
      return;
    }
    let font_size = self.settings.theme.font.size;
    let Some((col, row)) = pixel_to_cell(self.cursor_position, font_size) else {
      self.hovered_url = None;
      self.hovered_link_span = None;
      return;
    };
    let Some(tab) = self.tabs.get(self.active_index) else {
      self.hovered_url = None;
      self.hovered_link_span = None;
      return;
    };

    let (result_url, result_span) =
      Self::resolve_hovered_url(&tab.grid, tab.scroll_offset, col, row);
    self.hovered_url = result_url;
    self.hovered_link_span = result_span;
  }

  fn send_mouse_event(
    &self,
    tab: &Tab,
    col: usize,
    row: usize,
    button: Option<mouse::Button>,
    pressed: bool,
    motion: bool,
  ) {
    let Some(tx) = &tab.pty_tx else { return };

    let mut cb = match button {
      Some(mouse::Button::Left) => 0,
      Some(mouse::Button::Middle) => 1,
      Some(mouse::Button::Right) => 2,
      None => 3, // Release or hover without button
      _ => return,
    };

    if motion {
      cb += 32;
    }

    if self.shift_held {
      cb |= 4;
    }
    if self.alt_held {
      cb |= 8;
    }
    if self.ctrl_held {
      cb |= 16;
    }

    if tab.grid.mouse_sgr {
      let state = if pressed { 'M' } else { 'm' };
      let cmd = format!("\x1b[<{};{};{}{}", cb, col + 1, row + 1, state);
      let _ = tx.try_send(PtyCommand::Input(cmd.into_bytes()));
    } else {
      let cb_byte = (cb + 32).min(255) as u8;
      let cx = (col + 1 + 32).min(255) as u8;
      let cy = (row + 1 + 32).min(255) as u8;
      let cmd = vec![b'\x1b', b'[', b'M', cb_byte, cx, cy];
      let _ = tx.try_send(PtyCommand::Input(cmd));
    }
  }

  fn resolve_hovered_url(
    grid: &crate::core::grid::Grid,
    scroll_offset: usize,
    col: usize,
    row: usize,
  ) -> (Option<String>, Option<(usize, usize, usize)>) {
    let Some(row_cells) = get_display_row(grid, scroll_offset, row) else {
      return (None, None);
    };

    if let Some(uri) = row_cells.get(col).and_then(|c| c.uri.as_deref()) {
      return (Some(uri.to_owned()), Some((row, col, row)));
    }

    let row_len = row_cells.len();
    let plain = crate::core::url::detect_urls(row_cells)
      .into_iter()
      .find(|(s, e, _)| col >= *s && col <= *e);

    if let Some((start_col, end_col, partial)) = plain {
      let (full_url, end_row) = if end_col == row_len.saturating_sub(1) {
        stitch_continuation(partial, grid, scroll_offset, row + 1)
      } else {
        (partial, row)
      };
      return (Some(full_url), Some((row, start_col, end_row)));
    }

    if row > 0
      && let Some(prev_cells) = get_display_row(grid, scroll_offset, row - 1)
    {
      let prev_len = prev_cells.len();
      let prev_ending = crate::core::url::detect_urls(prev_cells)
        .into_iter()
        .find(|(_, e, _)| *e == prev_len.saturating_sub(1));
      if let Some((start_col, _, partial)) = prev_ending {
        let cont_len = crate::core::url::url_continuation_len(row_cells);
        if cont_len > 0 && col < cont_len {
          let (full_url, end_row) = stitch_continuation(partial, grid, scroll_offset, row);
          return (Some(full_url), Some((row - 1, start_col, end_row)));
        }
      }
    }

    (None, None)
  }

  fn resize_all_grids(&mut self) {
    let (cols, rows) = calc_grid(
      self.window_size.width,
      self.window_size.height,
      self.settings.theme.font.size,
      self.settings.status_bar.visible,
      self.diagnostic_banner.is_some(),
    );
    for tab in self.tabs.iter_mut() {
      tab.grid.resize(cols, rows);
      if let Some(tx) = &tab.pty_tx {
        let _ = tx.send_blocking(PtyCommand::Resize {
          cols: cols as u16,
          rows: rows as u16,
        });
      }
    }
  }

  pub fn update(&mut self, message: Message) -> iced::Task<Message> {
    match message {
      Message::Type(bytes) => {
        if self.settings_open
          || self.command_palette_open
          || self.ai_overlay_open
          || self.ai_loading
        {
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
            }
          } else if bytes == b"\x7F" || bytes == b"\x08" {
            active_tab.current_input.pop();
          } else if bytes == b"\x03"
            || bytes == b"\x15"
            || (bytes.len() >= 2 && bytes[0] == 0x1b && (bytes[1] == b'A' || bytes[1] == b'B'))
          {
            active_tab.current_input.clear();
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
      }
      Message::NewTab => {
        let shell = self.available_shells.first().cloned().unwrap_or_default();
        return self.update(Message::NewTabWithShell(shell));
      }
      Message::OpenShellPicker => {
        if self.available_shells.len() <= 1 {
          let shell = self.available_shells.first().cloned().unwrap_or_default();
          return self.update(Message::NewTabWithShell(shell));
        }
        self.shell_picker_anchor = self.cursor_position.x;
        self.shell_picker_open = true;
      }
      Message::CloseShellPicker => {
        self.shell_picker_open = false;
      }
      Message::NewTabWithShell(shell) => {
        self.shell_picker_open = false;
        let new_id = self.next_tab_id;
        self.next_tab_id += 1;
        let (cols, rows) = calc_grid(
          self.window_size.width,
          self.window_size.height,
          self.settings.theme.font.size,
          self.settings.status_bar.visible,
          self.diagnostic_banner.is_some(),
        );
        let parent_pwd = self
          .tabs
          .get(self.active_index)
          .map(|t| t.pwd.clone())
          .unwrap_or_default();
        self
          .tabs
          .push(Tab::new(new_id, cols, rows, shell, parent_pwd));
        self.active_index = self.tabs.len() - 1;
      }
      Message::SwitchTab(index) => {
        if index < self.tabs.len() {
          self.active_index = index;
        }
      }
      Message::CloseTab(index) => {
        self.tabs.remove(index);
        if self.tabs.is_empty() {
          let shell = self.available_shells.first().cloned().unwrap_or_default();
          let (cols, rows) = calc_grid(
            self.window_size.width,
            self.window_size.height,
            self.settings.theme.font.size,
            self.settings.status_bar.visible,
            self.diagnostic_banner.is_some(),
          );
          self
            .tabs
            .push(Tab::new(self.next_tab_id, cols, rows, shell, String::new()));
          self.next_tab_id += 1;
          self.active_index = 0;
        } else if self.active_index >= self.tabs.len() {
          self.active_index = self.tabs.len() - 1;
        }
      }
      Message::CloseActiveTab => {
        return self.update(Message::CloseTab(self.active_index));
      }
      Message::NextTab => {
        if !self.tabs.is_empty() {
          self.active_index = (self.active_index + 1) % self.tabs.len();
        }
      }
      Message::PrevTab => {
        if !self.tabs.is_empty() {
          self.active_index = if self.active_index == 0 {
            self.tabs.len() - 1
          } else {
            self.active_index - 1
          };
        }
      }
      Message::PtyReady(tab_id, tx) => {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
          if let Some(cmd) = tab.pending_command.take() {
            let _ = tx.send_blocking(PtyCommand::Input(cmd));
          }
          tab.pty_tx = Some(tx);
          tab.pty_alive = true;
        }
      }
      Message::PtyExited(tab_id) => {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
          tab.pty_alive = false;
          tab.pty_tx = None;
        }
      }
      Message::OpenSettings => {
        self.settings_open = true;
        self.settings_tab = SettingsTab::General;
        self.settings_shell_input = String::new();
        self.settings_recording_index = None;
        SETTINGS_OPEN.store(true, Ordering::SeqCst);
      }
      Message::CloseSettings => {
        self.settings_open = false;
        self.settings_recording_index = None;
        KB_RECORDING.store(false, Ordering::SeqCst);
        SETTINGS_OPEN.store(false, Ordering::SeqCst);
      }
      Message::SettingsTabSelected(tab) => {
        if tab == SettingsTab::Raw {
          self.raw_config_content = toml::to_string_pretty(&self.settings)
            .unwrap_or_else(|_| String::from("# error serializing config"));
        }
        self.settings_tab = tab;
      }
      Message::SettingsEditorChanged(s) => {
        self.settings.general.editor = s;
        let _ = config::save(&self.settings);
      }
      Message::SettingsBellChanged(bell) => {
        self.settings.general.bell = bell;
        let _ = config::save(&self.settings);
      }
      Message::SettingsShellInputChanged(s) => {
        self.settings_shell_input = s;
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
      }
      Message::SettingsRemoveShell(i) => {
        if let Some(shells) = &mut self.settings.general.shells
          && i < shells.len()
        {
          shells.remove(i);
        }
        let _ = config::save(&self.settings);
        self.available_shells = derive_available_shells(&self.settings);
      }
      Message::SettingsFontFamilyChanged(s) => {
        self.settings.theme.font.family = s;
        let _ = config::save(&self.settings);
      }
      Message::SettingsFontSizeChanged(size) => {
        let size = size.clamp(8.0, 72.0);
        self.settings.theme.font.size = size;
        let _ = config::save(&self.settings);
        self.resize_all_grids();
      }
      Message::SettingsColorChanged(field, hex) => {
        if config::parse_hex_color(&hex).is_ok() {
          match field {
            ColorField::Background => self.settings.theme.colors.background = hex,
            ColorField::Foreground => self.settings.theme.colors.foreground = hex,
            ColorField::Accent => self.settings.theme.colors.accent = hex,
            ColorField::ForegroundMuted => self.settings.theme.colors.foreground_muted = hex,
            ColorField::Border => self.settings.theme.colors.border = hex,
            ColorField::Cursor => self.settings.theme.colors.cursor = hex,
          }
          let _ = config::save(&self.settings);
          rebuild_runtime_theme(&self.settings.theme.colors);
        }
      }
      Message::SettingsStatusBarToggled(visible) => {
        self.settings.status_bar.visible = visible;
        let _ = config::save(&self.settings);
        self.resize_all_grids();
      }
      Message::SettingsDateFormatChanged(s) => {
        self.settings.status_bar.date_format = s;
        let _ = config::save(&self.settings);
      }
      Message::SettingsTimeFormatChanged(s) => {
        self.settings.status_bar.time_format = s;
        let _ = config::save(&self.settings);
      }
      Message::SettingsStartRecordKb(idx) => {
        self.settings_recording_index = Some(idx);
        KB_RECORDING.store(true, Ordering::SeqCst);
      }
      Message::SettingsRecordKb { key, modifiers } => {
        if let Some(idx) = self.settings_recording_index
          && let Some(s) = keybinding_to_string(&key, modifiers)
        {
          match idx {
            0 => self.settings.keybindings.new_tab = s,
            1 => self.settings.keybindings.close_tab = s,
            2 => self.settings.keybindings.next_tab = s,
            3 => self.settings.keybindings.prev_tab = s,
            4 => self.settings.keybindings.paste = s,
            5 => self.settings.keybindings.copy = s,
            _ => {}
          }
          let _ = config::save(&self.settings);
          let _ = config::reload_parsed_keybindings(&self.settings);
          self.settings_recording_index = None;
          KB_RECORDING.store(false, Ordering::SeqCst);
        }
      }
      Message::SettingsCancelRecordKb => {
        self.settings_recording_index = None;
        KB_RECORDING.store(false, Ordering::SeqCst);
      }
      Message::SettingsResetKb(idx) => {
        let default_cfg: config::Config =
          toml::from_str(config::default_config_str()).expect("invalid default config");
        match idx {
          0 => self.settings.keybindings.new_tab = default_cfg.keybindings.new_tab,
          1 => self.settings.keybindings.close_tab = default_cfg.keybindings.close_tab,
          2 => self.settings.keybindings.next_tab = default_cfg.keybindings.next_tab,
          3 => self.settings.keybindings.prev_tab = default_cfg.keybindings.prev_tab,
          4 => self.settings.keybindings.paste = default_cfg.keybindings.paste,
          5 => self.settings.keybindings.copy = default_cfg.keybindings.copy,
          _ => {}
        }
        let _ = config::save(&self.settings);
        let _ = config::reload_parsed_keybindings(&self.settings);
      }
      Message::SettingsResetAll => {
        if let Ok(cfg) = config::reset_to_defaults() {
          self.settings = cfg;
          rebuild_runtime_theme(&self.settings.theme.colors);
          self.available_shells = derive_available_shells(&self.settings);
          let _ = config::reload_parsed_keybindings(&self.settings);
          self.resize_all_grids();
        }
      }
      Message::PtyOutputReceived(tab_id, bytes) => {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
          if std::env::var("NOVA_DEBUG_PTY").is_ok()
            && let Ok(mut f) = std::fs::OpenOptions::new()
              .create(true)
              .append(true)
              .open("C:\\Users\\Public\\nova_pty_debug.bin")
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
                  && !self.settings.ai.api_key.is_empty()
                {
                  self.diagnostic_banner = Some((code, "Loading...".into(), None));
                  self.ai_pending_diagnostic = Some(code);
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
            return iced::Task::perform(
              crate::core::ai::query(q),
              Message::DiagnosticBannerResponse,
            );
          }

          if bell_fired {
            match self.settings.general.bell {
              config::BellType::Blink => {
                self.bell_blink_visible = false;
                self.bell_blink_remaining = 5;
              }
              config::BellType::Audio => {
                crate::sys::bell::play();
              }
              config::BellType::None => {}
            }
          }
        }
      }
      Message::WindowOpened(id) => {
        self.window_id = Some(id);
        #[cfg(target_os = "windows")]
        return window::set_mode(id, window::Mode::Windowed);
      }
      Message::MinimizeWindow => {
        if let Some(window_id) = self.window_id {
          return window::minimize(window_id, true);
        }
      }
      Message::MaximizeWindow => {
        if let Some(window_id) = self.window_id {
          self.window_maximized = !self.window_maximized;
          return window::toggle_maximize(window_id);
        }
      }
      Message::CloseWindow => {
        std::process::exit(0);
      }
      Message::DragWindow => {
        if let Some(window_id) = self.window_id {
          return window::drag(window_id);
        }
      }
      Message::WindowFocused => {
        self.window_focused = true;
      }
      Message::WindowUnfocused => {
        self.window_focused = false;
      }
      Message::WindowMaximizedState(maximized) => {
        self.window_maximized = maximized;
      }
      Message::WindowResized(width, height) => {
        if width < 100.0 || height < 100.0 {
          return iced::Task::none();
        }
        self.window_size = Size::new(width, height);
        let (cols, rows) = calc_grid(
          width,
          height,
          self.settings.theme.font.size,
          self.settings.status_bar.visible,
          self.diagnostic_banner.is_some(),
        );

        for tab in self.tabs.iter_mut() {
          if tab.grid.cols == cols && tab.grid.rows == rows {
            continue;
          }
          tab.grid.resize(cols, rows);
          if let Some(tx) = &tab.pty_tx {
            let _ = tx.send_blocking(PtyCommand::Resize {
              cols: cols as u16,
              rows: rows as u16,
            });
          }
        }

        if let Some(id) = self.window_id {
          return iced::window::is_maximized(id).map(Message::WindowMaximizedState);
        }
      }
      Message::CursorMoved(position) => {
        self.cursor_position = position;
        let font_size = self.settings.theme.font.size;

        let mut bypass_selection = false;
        if let Some(tab) = self.tabs.get(self.active_index)
          && tab.grid.mouse_mode != crate::core::grid::MouseMode::None
          && !self.shift_held
        {
          bypass_selection = true;
          if (tab.grid.mouse_mode == crate::core::grid::MouseMode::AnyEvent
            || (tab.grid.mouse_mode == crate::core::grid::MouseMode::Button
              && self.last_mouse_button.is_some()))
            && let Some(cell) = pixel_to_cell(position, font_size)
          {
            if let Some(button) = self.last_mouse_button {
              self.send_mouse_event(tab, cell.0, cell.1, Some(button), true, true);
            } else {
              self.send_mouse_event(tab, cell.0, cell.1, None, false, true);
            }
          }
        }

        if self.is_selecting && !bypass_selection {
          let end = pixel_to_cell(position, font_size);
          self.selection_end = end;
          if self.click_count >= 2
            && let (Some((end_col, end_row)), Some(active_tab)) =
              (end, self.tabs.get(self.active_index))
          {
            match self.click_count {
              2 => {
                if let Some(row_cells) =
                  get_display_row(&active_tab.grid, active_tab.scroll_offset, end_row)
                {
                  let (_, we) = find_word_boundaries(row_cells, end_col);
                  self.selection_end = Some((we, end_row));
                }
              }
              3 => {
                self.selection_end = Some((active_tab.grid.cols.saturating_sub(1), end_row));
              }
              _ => {}
            }
          }
        }
        self.update_hovered_url();
      }
      Message::MousePressed(button) => {
        self.last_mouse_button = Some(button);
        if self.settings_open
          || self.command_palette_open
          || self.ai_overlay_open
          || self.ai_loading
        {
          return iced::Task::none();
        }
        if self.ctrl_held
          && let Some(url) = self.hovered_url.clone()
        {
          let _ = open::that_detached(&url);
          return iced::Task::none();
        }
        if let Some(window_id) = self.window_id
          && let Some(direction) = resize_direction(self.cursor_position, self.window_size)
        {
          return window::drag_resize(window_id, direction);
        }

        let font_size = self.settings.theme.font.size;
        let cell = pixel_to_cell(self.cursor_position, font_size);

        if let Some(tab) = self.tabs.get(self.active_index)
          && tab.grid.mouse_mode != crate::core::grid::MouseMode::None
          && !self.shift_held
        {
          if let Some((col, row)) = cell {
            self.send_mouse_event(tab, col, row, Some(button), true, false);
          }
          self.click_count = 0;
          return iced::Task::none();
        }

        if button == mouse::Button::Left {
          let now = Instant::now();
          let threshold = std::time::Duration::from_millis(500);
          if cell.is_some()
            && cell == self.last_click_cell
            && now.duration_since(self.last_click_time) < threshold
          {
            self.click_count = (self.click_count + 1).min(3);
          } else {
            self.click_count = 1;
          }
          self.last_click_time = now;
          self.last_click_cell = cell;

          self.selection_start = cell;
          self.selection_end = cell;
          self.is_selecting = cell.is_some();

          if let Some((col, row)) = cell
            && self.click_count >= 2
            && let Some(active_tab) = self.tabs.get(self.active_index)
          {
            match self.click_count {
              2 => {
                if let Some(row_cells) =
                  get_display_row(&active_tab.grid, active_tab.scroll_offset, row)
                {
                  let (ws, we) = find_word_boundaries(row_cells, col);
                  self.selection_start = Some((ws, row));
                  self.selection_end = Some((we, row));
                }
              }
              3 => {
                self.selection_start = Some((0, row));
                self.selection_end = Some((active_tab.grid.cols.saturating_sub(1), row));
              }
              _ => {}
            }
          }
        }
      }
      Message::MouseReleased(button) => {
        if self.last_mouse_button == Some(button) {
          self.last_mouse_button = None;
        }

        self.is_selecting = false;
        if self.settings_open
          || self.command_palette_open
          || self.ai_overlay_open
          || self.ai_loading
        {
          return iced::Task::none();
        }

        if let Some(tab) = self.tabs.get(self.active_index)
          && tab.grid.mouse_mode != crate::core::grid::MouseMode::None
          && !self.shift_held
        {
          let font_size = self.settings.theme.font.size;
          if let Some((col, row)) = pixel_to_cell(self.cursor_position, font_size) {
            self.send_mouse_event(tab, col, row, Some(button), false, false);
          }
          return iced::Task::none();
        }

        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
          if start == end {
            self.selection_start = None;
            self.selection_end = None;
          } else if let Some(active_tab) = self.tabs.get(self.active_index) {
            let text = extract_selection(&active_tab.grid, active_tab.scroll_offset, start, end);
            if !text.is_empty() {
              return iced::clipboard::write(text);
            }
          }
        }
      }
      Message::CopySelection => {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end)
          && let Some(active_tab) = self.tabs.get(self.active_index)
        {
          let text = extract_selection(&active_tab.grid, active_tab.scroll_offset, start, end);
          if !text.is_empty() {
            return iced::clipboard::write(text);
          }
        }
      }
      Message::PasteRequested => {
        return iced::clipboard::read().map(Message::ClipboardReceived);
      }
      Message::ClipboardReceived(text) => {
        if let Some(text) = text
          && let Some(tab) = self.tabs.get(self.active_index)
          && let Some(tx) = &tab.pty_tx
        {
          if tab.grid.bracketed_paste {
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
      Message::Scroll(delta) => {
        if !self.settings_open {
          let font_size = self.settings.theme.font.size;
          if let Some(tab) = self.tabs.get_mut(self.active_index) {
            if tab.grid.mouse_mode != crate::core::grid::MouseMode::None && !self.shift_held {
              if let Some((col, row)) = pixel_to_cell(self.cursor_position, font_size) {
                let is_down = delta < 0.0;
                let btn = if is_down { 65 } else { 64 };
                let mut cb = btn;
                if self.shift_held {
                  cb |= 4;
                }
                if self.alt_held {
                  cb |= 8;
                }
                if self.ctrl_held {
                  cb |= 16;
                }

                if let Some(tx) = &tab.pty_tx {
                  if tab.grid.mouse_sgr {
                    let cmd = format!("\x1b[<{};{};{}M", cb, col + 1, row + 1);
                    let _ = tx.try_send(PtyCommand::Input(cmd.into_bytes()));
                  } else {
                    let cb_byte = (cb + 32).min(255) as u8;
                    let cx = (col + 1 + 32).min(255) as u8;
                    let cy = (row + 1 + 32).min(255) as u8;
                    let cmd = vec![b'\x1b', b'[', b'M', cb_byte, cx, cy];
                    let _ = tx.try_send(PtyCommand::Input(cmd));
                  }
                }
              }
              return iced::Task::none();
            }

            let rows = (delta.abs() * 3.0).round() as usize;
            let old_offset = tab.scroll_offset;
            if delta > 0.0 {
              let new_offset = tab.scroll_offset.saturating_add(rows);
              tab.scroll_offset = new_offset.min(tab.grid.scrollback.len());
            } else {
              tab.scroll_offset = tab.scroll_offset.saturating_sub(rows);
            }
            let scroll_delta = tab.scroll_offset as isize - old_offset as isize;
            if scroll_delta != 0 {
              if let Some((col, row)) = self.selection_start {
                let new_row = (row as isize + scroll_delta).max(0) as usize;
                self.selection_start = Some((col, new_row));
              }
              if let Some((col, row)) = self.selection_end {
                let new_row = (row as isize + scroll_delta).max(0) as usize;
                self.selection_end = Some((col, new_row));
              }
            }
          }
        }
      }
      Message::ModifiersChanged(mods) => {
        self.ctrl_held = mods.command();
        self.shift_held = mods.shift();
        self.alt_held = mods.alt();
        self.update_hovered_url();
      }
      Message::OpenCommandPalette => {
        self.command_palette_open = true;
        self.palette_query = String::new();
        self.palette_selected = 0;
        PALETTE_OPEN.store(true, Ordering::SeqCst);
        return iced::widget::operation::focus(components::PALETTE_INPUT_ID.clone());
      }
      Message::CloseCommandPalette => {
        self.command_palette_open = false;
        self.palette_query = String::new();
        self.palette_selected = 0;
        PALETTE_OPEN.store(false, Ordering::SeqCst);
      }
      Message::PaletteQueryChanged(s) => {
        self.palette_query = s;
        self.palette_selected = 0;
      }
      Message::PaletteNavigate(d) => {
        let count = components::palette_filtered_count(&self.palette_query);
        if count > 0 {
          self.palette_selected =
            (self.palette_selected as i32 + d).rem_euclid(count as i32) as usize;
        }
      }
      Message::PaletteConfirm => {
        return self.update(Message::PaletteSelectAndConfirm(self.palette_selected));
      }
      Message::PaletteSelectAndConfirm(i) => {
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
      }
      Message::OpenAskAi => {
        self.ai_overlay_open = true;
        AI_OPEN.store(true, Ordering::SeqCst);
        self.ai_input = String::new();
        self.ai_response = None;
        self.ai_is_error = false;
        self.ai_loading = false;
        return iced::widget::operation::focus(components::AI_INPUT_ID.clone());
      }
      Message::CloseAiOverlay => {
        self.ai_overlay_open = false;
        AI_OPEN.store(false, Ordering::SeqCst);
        self.ai_input = String::new();
        self.ai_response = None;
        self.ai_is_error = false;
        self.ai_loading = false;
      }
      Message::AiOverlayInputChanged(s) => {
        self.ai_input = s;
      }
      Message::AiSubmit => {
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
        return iced::Task::perform(crate::core::ai::query(q), Message::AiResponseReceived);
      }
      Message::AiResponseReceived(result) => {
        self.ai_loading = false;
        match result {
          Ok(text) => {
            self.ai_response = Some(text);
            self.ai_is_error = false;
          }
          Err(e) => {
            self.ai_response = Some(e);
            self.ai_is_error = true;
          }
        }
      }
      Message::DiagnosticBannerResponse(result) => {
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
      Message::DiagnosticBannerCommand(cmd) => {
        if let Some(tab) = self.tabs.get(self.active_index)
          && let Some(tx) = &tab.pty_tx
        {
          let _ = tx.try_send(crate::sys::pty::PtyCommand::Input(cmd.into_bytes()));
        }
      }
      Message::SettingsDiagnosticBannerToggled(enabled) => {
        if !enabled {
          self.diagnostic_banner = None;
        }
        self.settings.ai.diagnostic_banner = enabled;
        let _ = config::save(&self.settings);
      }
      Message::ExplainError => {
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
        return iced::Task::perform(crate::core::ai::query(q), Message::AiResponseReceived);
      }
      Message::CopyCodeBlock(code) => {
        return iced::clipboard::write(code);
      }
      Message::RunCodeInTerminal(code) => {
        self.ai_overlay_open = false;
        AI_OPEN.store(false, Ordering::SeqCst);
        self.ai_response = None;
        self.ai_is_error = false;
        self.ai_loading = false;
        if let Some(tab) = self.tabs.get(self.active_index)
          && let Some(tx) = &tab.pty_tx
        {
          let _ = tx.try_send(crate::sys::pty::PtyCommand::Input(code.into_bytes()));
        }
      }
      Message::SettingsAiProviderChanged(provider) => {
        self.settings.ai.provider = provider;
        let _ = config::save(&self.settings);
      }
      Message::SettingsAiModelChanged(s) => {
        self.settings.ai.model = s;
        let _ = config::save(&self.settings);
      }
      Message::SettingsAiApiKeyChanged(s) => {
        self.settings.ai.api_key = s;
        let _ = config::save(&self.settings);
      }
      Message::SettingsWindowControlsChanged(style) => {
        self.settings.general.window_controls = style;
        let _ = config::save(&self.settings);
      }
      Message::SettingsAiBaseUrlChanged(s) => {
        self.settings.ai.base_url = if s.trim().is_empty() { None } else { Some(s) };
        let _ = config::save(&self.settings);
      }
      Message::BellBlinkTick => {
        if self.bell_blink_remaining > 0 {
          self.bell_blink_remaining -= 1;
          self.bell_blink_visible = !self.bell_blink_visible;
          if self.bell_blink_remaining == 0 {
            self.bell_blink_visible = true;
          }
        }
      }
      Message::Tick => {}
      Message::NoOp => {}
    }

    iced::Task::none()
  }

  pub fn view(&self) -> Element<'_, Message> {
    let active_tab = &self.tabs[self.active_index];

    let selection = match (self.selection_start, self.selection_end) {
      (Some(start), Some(end)) if start != end => {
        let ((sc, sr), (ec, er)) = normalize_sel(start, end);
        Some((sc, sr, ec, er))
      }
      _ => None,
    };

    let font_size = self.settings.theme.font.size;

    let resize_cursor = resize_direction(self.cursor_position, self.window_size).map(dir_to_cursor);

    let term_interaction = resize_cursor.unwrap_or_else(|| {
      if self.hovered_url.is_some() {
        mouse::Interaction::Pointer
      } else {
        mouse::Interaction::Text
      }
    });
    let term = mouse_area(components::term(
      active_tab,
      selection,
      font_size,
      active_tab.scroll_offset,
      self.hovered_url.as_deref(),
      self.hovered_link_span,
      active_tab.grid.suggestion.as_deref(),
    ))
    .interaction(term_interaction);

    let tb_interaction = resize_cursor.unwrap_or(mouse::Interaction::Idle);

    let mut col = column![
      components::title_bar(
        self.window_focused,
        &active_tab.pwd,
        self.window_maximized,
        tb_interaction,
        &self.settings.general.window_controls,
        self.bell_blink_visible,
      ),
      components::tab_bar(&self.tabs, self.active_index),
      term,
    ];

    if let Some((_code, ref message, ref command)) = self.diagnostic_banner {
      let rt = theme::color::runtime();
      let bg = rt.background;
      let accent = rt.accent;
      let fg = rt.foreground;
      drop(rt);
      let mut inner = column![].spacing(6);
      inner = inner.push(
        text(" \u{2726} NOVA \u{00B7} AI ")
          .font(theme::font::BOLD)
          .size(12)
          .color(accent),
      );
      inner = inner.push(
        text(format!(" {}", strip_markdown(message)))
          .font(theme::font::REGULAR)
          .size(12)
          .color(fg),
      );
      if let Some(cmd) = command {
        let cmd_text = cmd.clone();
        inner = inner.push(
          button(
            text(format!(" {} ", cmd_text))
              .font(theme::font::REGULAR)
              .size(12)
              .color(accent),
          )
          .on_press(Message::DiagnosticBannerCommand(cmd_text))
          .padding(Padding::from([4, 10]))
          .style(move |_t, _s| button::Style {
            background: Some(Color { a: 0.08, ..accent }.into()),
            border: Border {
              color: accent,
              radius: Radius::new(4.0),
              width: 0.0,
            },
            text_color: accent,
            ..Default::default()
          }),
        );
      }
      col = col.push(
        container(
          container(inner)
            .padding(Padding::from([8, 12]))
            .style(move |_| container::Style {
              background: Some(Color { a: 0.08, ..accent }.into()),
              border: Border {
                color: accent,
                radius: Radius::new(8.0),
                width: 1.0,
              },
              ..Default::default()
            })
            .width(Length::Fill),
        )
        .padding(Padding::from([8, 8]))
        .style(move |_| container::Style {
          background: Some(bg.into()),
          ..Default::default()
        })
        .width(Length::Fill),
      );
    }
    if self.settings.status_bar.visible {
      col = col.push(components::status_bar(
        active_tab,
        &self.settings.status_bar.date_format,
        &self.settings.status_bar.time_format,
        self.window_maximized,
      ));
    }

    let outer_interaction = resize_cursor.unwrap_or(mouse::Interaction::Idle);

    let inner = if self.settings_open {
      let config_path_str = config::config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
      let modal = components::settings_modal(
        &self.settings,
        &self.settings_tab,
        &self.settings_shell_input,
        self.settings_recording_index,
        &self.raw_config_content,
        config_path_str,
      );
      components::app(stack![col, modal], self.window_maximized)
    } else if self.command_palette_open {
      let palette = components::command_palette(&self.palette_query, self.palette_selected);
      components::app(stack![col, palette], self.window_maximized)
    } else if self.ai_overlay_open || self.ai_loading {
      let overlay = components::ai_overlay(
        &self.ai_input,
        self.ai_response.as_deref(),
        self.ai_loading,
        self.ai_is_error,
      );
      components::app(stack![col, overlay], self.window_maximized)
    } else if self.shell_picker_open {
      let picker = components::shell_picker(
        &self.available_shells,
        self.shell_picker_anchor,
        self.window_size.width,
      );
      components::app(stack![col, picker], self.window_maximized)
    } else {
      components::app(col, self.window_maximized)
    };

    mouse_area(inner).interaction(outer_interaction).into()
  }

  pub fn theme(&self) -> Theme {
    let rt = crate::ui::theme::color::runtime();
    Theme::custom(
      "Nova",
      iced::theme::Palette {
        background: rt.background,
        text: rt.foreground,
        primary: rt.accent,
        success: rt.accent,
        warning: iced::Color::from_rgb(1.0, 0.75, 0.0),
        danger: iced::Color::from_rgb(0.9, 0.3, 0.3),
      },
    )
  }

  pub fn subscription(&self) -> Subscription<Message> {
    let mut subs = Vec::new();

    let time_sub = time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick);
    subs.push(time_sub);

    if self.bell_blink_remaining > 0 {
      let blink_sub =
        time::every(std::time::Duration::from_millis(200)).map(|_| Message::BellBlinkTick);
      subs.push(blink_sub);
    }

    let global_sub = event::listen_with(|event, _s, window_id| match event {
      Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        modifiers,
        modified_key,
        ..
      }) => {
        if KB_RECORDING.load(Ordering::SeqCst) {
          return match &key {
            Key::Named(Named::Escape) => Some(Message::SettingsCancelRecordKb),
            _ => Some(Message::SettingsRecordKb {
              key: key.clone(),
              modifiers,
            }),
          };
        }
        if SETTINGS_OPEN.load(Ordering::SeqCst) {
          return match &key {
            Key::Named(Named::Escape) => Some(Message::CloseSettings),
            _ => None,
          };
        }
        if AI_OPEN.load(Ordering::SeqCst) {
          return match &key {
            Key::Named(Named::Escape) => Some(Message::CloseAiOverlay),
            _ => None,
          };
        }
        if PALETTE_OPEN.load(Ordering::SeqCst) {
          return match &key {
            Key::Named(Named::Escape) => Some(Message::CloseCommandPalette),
            Key::Named(Named::ArrowUp) => Some(Message::PaletteNavigate(-1)),
            Key::Named(Named::ArrowDown) => Some(Message::PaletteNavigate(1)),
            Key::Named(Named::Enter) => Some(Message::PaletteConfirm),
            _ => None,
          };
        }

        let kb = config::keybindings();
        if matches_kb(&kb.prev_tab, &key, modifiers) {
          return Some(Message::PrevTab);
        }
        if matches_kb(&kb.next_tab, &key, modifiers) {
          return Some(Message::NextTab);
        }
        if matches_kb(&kb.new_tab, &key, modifiers) {
          return Some(Message::NewTab);
        }
        if matches_kb(&kb.close_tab, &key, modifiers) {
          return Some(Message::CloseActiveTab);
        }
        if matches_kb(&kb.paste, &key, modifiers) {
          return Some(Message::PasteRequested);
        }
        if matches_kb(&kb.copy, &key, modifiers) {
          return Some(Message::CopySelection);
        }
        if matches_kb(&kb.open_palette, &key, modifiers) {
          return Some(Message::OpenCommandPalette);
        }
        drop(kb);

        match &key {
          Key::Named(Named::Enter) => return Some(Message::Type(b"\r".to_vec())),
          Key::Named(Named::Backspace) => return Some(Message::Type(b"\x7F".to_vec())),
          Key::Named(Named::Tab) => return Some(Message::Type(b"\t".to_vec())),
          Key::Named(Named::Space) => return Some(Message::Type(b" ".to_vec())),
          Key::Named(Named::Escape) => return Some(Message::Type(b"\x1b".to_vec())),
          Key::Named(Named::ArrowUp) => return Some(Message::Type(b"\x1b[A".to_vec())),
          Key::Named(Named::ArrowDown) => return Some(Message::Type(b"\x1b[B".to_vec())),
          #[cfg(target_os = "macos")]
          Key::Named(Named::ArrowRight) => {
            return Some(Message::Type(if modifiers.logo() {
              b"\x05".to_vec()
            } else if modifiers.alt() || modifiers.control() {
              b"\x1bf".to_vec()
            } else {
              b"\x1b[C".to_vec()
            }));
          }
          #[cfg(not(target_os = "macos"))]
          Key::Named(Named::ArrowRight) => {
            return Some(Message::Type(if modifiers.alt() {
              b"\x05".to_vec()
            } else if modifiers.control() {
              b"\x1bf".to_vec()
            } else {
              b"\x1b[C".to_vec()
            }));
          }
          #[cfg(target_os = "macos")]
          Key::Named(Named::ArrowLeft) => {
            return Some(Message::Type(if modifiers.logo() {
              b"\x01".to_vec()
            } else if modifiers.alt() || modifiers.control() {
              b"\x1bb".to_vec()
            } else {
              b"\x1b[D".to_vec()
            }));
          }
          #[cfg(not(target_os = "macos"))]
          Key::Named(Named::ArrowLeft) => {
            return Some(Message::Type(if modifiers.alt() {
              b"\x01".to_vec()
            } else if modifiers.control() {
              b"\x1bb".to_vec()
            } else {
              b"\x1b[D".to_vec()
            }));
          }
          Key::Named(Named::Delete) => return Some(Message::Type(b"\x1b[3~".to_vec())),
          Key::Named(Named::Home) => return Some(Message::Type(b"\x1b[H".to_vec())),
          Key::Named(Named::End) => return Some(Message::Type(b"\x1b[F".to_vec())),
          Key::Named(Named::PageUp) => return Some(Message::Type(b"\x1b[5~".to_vec())),
          Key::Named(Named::PageDown) => return Some(Message::Type(b"\x1b[6~".to_vec())),
          _ => {}
        }

        if modifiers.control() {
          if let Key::Character(c) = &key
            && let Some(ch) = c.as_str().chars().next()
            && ch.is_ascii_alphabetic()
          {
            let lower = ch.to_ascii_lowercase();
            return Some(Message::Type(vec![(lower as u8) & 0x1f]));
          }
          return None;
        }

        let char_source = match &modified_key {
          Key::Character(_) => &modified_key,
          _ => &key,
        };

        if let Key::Character(c) = char_source {
          let mut s = c.as_str().to_string();
          if modifiers.shift() {
            if s == "'" {
              s = "\"".to_string();
            }
            if s == "`" {
              s = "~".to_string();
            }
          }
          return Some(Message::Type(s.into_bytes()));
        }

        None
      }
      Event::Window(window::Event::Opened { .. }) => Some(Message::WindowOpened(window_id)),
      Event::Window(window::Event::Focused) => Some(Message::WindowFocused),
      Event::Window(window::Event::Unfocused) => Some(Message::WindowUnfocused),
      Event::Window(window::Event::Resized(size)) => {
        Some(Message::WindowResized(size.width, size.height))
      }
      Event::Mouse(mouse::Event::CursorMoved { position }) => Some(Message::CursorMoved(position)),
      Event::Mouse(mouse::Event::ButtonPressed(button)) => Some(Message::MousePressed(button)),
      Event::Mouse(mouse::Event::ButtonReleased(button)) => Some(Message::MouseReleased(button)),
      Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
        let lines = match delta {
          mouse::ScrollDelta::Lines { y, .. } => y,
          mouse::ScrollDelta::Pixels { y, .. } => y / 20.0,
        };
        if lines != 0.0 {
          return Some(Message::Scroll(lines));
        }
        None
      }
      Event::Keyboard(keyboard::Event::ModifiersChanged(mods)) => {
        Some(Message::ModifiersChanged(mods))
      }
      _ => None,
    });

    subs.push(global_sub);

    for tab in &self.tabs {
      if !tab.pty_alive {
        continue;
      }
      let key = PtyKey {
        tab_id: tab.id,
        shell_cmd: tab.shell_cmd.clone(),
        initial_cols: tab.grid.cols as u16,
        initial_rows: tab.grid.rows as u16,
        initial_cwd: tab.initial_cwd.clone(),
      };
      let pty_sub = Subscription::run_with(key, |k| {
        pty_worker(
          k.tab_id,
          k.initial_cols,
          k.initial_rows,
          k.shell_cmd.clone(),
          k.initial_cwd.clone(),
        )
      });
      subs.push(pty_sub);
    }

    Subscription::batch(subs)
  }
}

fn strip_markdown(text: &str) -> String {
  text
    .replace("**", "")
    .replace("__", "")
    .replace("```", "")
    .replace("`", "")
    .lines()
    .map(|l| l.trim().to_string())
    .collect::<Vec<_>>()
    .join(" ")
    .replace("  ", " ")
    .trim()
    .to_string()
}

fn os_name() -> String {
  match std::env::consts::OS {
    "macos" => "macOS".to_string(),
    "windows" => "Windows".to_string(),
    "linux" => "Linux".to_string(),
    other => other.to_string(),
  }
}

fn dir_to_cursor(dir: window::Direction) -> mouse::Interaction {
  match dir {
    window::Direction::North | window::Direction::South => mouse::Interaction::ResizingVertically,
    window::Direction::East | window::Direction::West => mouse::Interaction::ResizingHorizontally,
    window::Direction::NorthWest | window::Direction::SouthEast => {
      mouse::Interaction::ResizingDiagonallyDown
    }
    window::Direction::NorthEast | window::Direction::SouthWest => {
      mouse::Interaction::ResizingDiagonallyUp
    }
  }
}

const RESIZE_EDGE: f32 = 8.0;

fn resize_direction(pos: Point, size: Size) -> Option<window::Direction> {
  let left = pos.x < RESIZE_EDGE;
  let right = pos.x > size.width - RESIZE_EDGE;
  let top = pos.y < RESIZE_EDGE;
  let bottom = pos.y > size.height - RESIZE_EDGE;

  match (top, bottom, left, right) {
    (true, _, true, _) => Some(window::Direction::NorthWest),
    (true, _, _, true) => Some(window::Direction::NorthEast),
    (_, true, true, _) => Some(window::Direction::SouthWest),
    (_, true, _, true) => Some(window::Direction::SouthEast),
    (true, _, false, false) => Some(window::Direction::North),
    (_, true, false, false) => Some(window::Direction::South),
    (false, false, true, _) => Some(window::Direction::West),
    (false, false, _, true) => Some(window::Direction::East),
    _ => None,
  }
}

struct PtyKey {
  tab_id: usize,
  shell_cmd: String,
  initial_cols: u16,
  initial_rows: u16,
  initial_cwd: String,
}

impl std::hash::Hash for PtyKey {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.tab_id.hash(state);
    self.shell_cmd.hash(state);
  }
}

impl PartialEq for PtyKey {
  fn eq(&self, other: &Self) -> bool {
    self.tab_id == other.tab_id && self.shell_cmd == other.shell_cmd
  }
}

impl Eq for PtyKey {}

fn pty_worker(
  tab_id: usize,
  cols: u16,
  rows: u16,
  shell: String,
  initial_cwd: String,
) -> impl iced::futures::Stream<Item = Message> {
  stream::channel(
    100,
    move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
      use iced::futures::SinkExt;

      let (tx_out, rx_out) = async_channel::unbounded::<Vec<u8>>();
      let (tx_in, rx_in) = async_channel::unbounded::<PtyCommand>();

      std::thread::spawn(move || {
        let cwd = if initial_cwd.is_empty() || initial_cwd == "~" {
          None
        } else {
          Some(initial_cwd.as_str())
        };
        let mut pty =
          PtyBridge::new(tx_out, cols, rows, &shell, cwd).expect("failed to create PTY bridge");

        while let Ok(command) = rx_in.recv_blocking() {
          match command {
            PtyCommand::Input(bytes) => pty.write_to_pty(&bytes),
            PtyCommand::Resize { cols, rows } => pty.resize_pty(cols, rows),
          }
        }
      });

      let _ = output.send(Message::PtyReady(tab_id, tx_in)).await;

      while let Ok(bytes) = rx_out.recv().await {
        let _ = output.send(Message::PtyOutputReceived(tab_id, bytes)).await;
      }

      let _ = output.send(Message::PtyExited(tab_id)).await;
    },
  )
}
