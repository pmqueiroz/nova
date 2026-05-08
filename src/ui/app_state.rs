use async_channel::Sender;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::{column, mouse_area, stack};
use iced::{Element, Point, Size, Subscription, Theme, time, window};
use iced::{Event, event, keyboard, mouse, stream};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::core::config::{self, KeyId, ParsedKeybinding};
use crate::sys::parser::AnsiExecutor;
use crate::sys::pty::{PtyBridge, PtyCommand};
use crate::ui::components;
use crate::ui::tab::Tab;

pub static SETTINGS_OPEN: AtomicBool = AtomicBool::new(false);
pub static KB_RECORDING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
  General,
  Theme,
  Keybindings,
  StatusBar,
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
  hovered_url: Option<String>,
  shell_picker_open: bool,
  shell_picker_anchor: f32,
  available_shells: Vec<String>,
  settings_open: bool,
  settings_tab: SettingsTab,
  settings: config::Config,
  settings_shell_input: String,
  settings_recording_index: Option<usize>,
  raw_config_content: String,
}

#[derive(Debug, Clone)]
pub enum Message {
  Type(Vec<u8>),
  NewTab,
  SwitchTab(usize),
  CloseTab(usize),
  PtyReady(usize, Sender<PtyCommand>),
  PtyOutputReceived(usize, Vec<u8>),
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
  PasteRequested,
  ClipboardReceived(Option<String>),
  OpenSettings,
  OpenShellPicker,
  CloseShellPicker,
  NewTabWithShell(String),
  CursorMoved(Point),
  MousePressed,
  MouseReleased,
  CopySelection,
  Tick,
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
    grid.scrollback.get(sb_len - clamped + y).map(|r| r.as_slice())
  } else {
    grid.cells.get(y - clamped).map(|r| r.as_slice())
  }
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

fn extract_selection(
  grid: &crate::core::grid::Grid,
  start: (usize, usize),
  end: (usize, usize),
) -> String {
  let ((sc, sr), (ec, er)) = normalize_sel(start, end);
  if sr == er && sc == ec {
    return String::new();
  }
  let sr = sr.min(grid.rows.saturating_sub(1));
  let er = er.min(grid.rows.saturating_sub(1));
  let mut result = String::new();
  for row in sr..=er {
    let row_cells = &grid.cells[row];
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
    for col in col_start..=col_end {
      result.push(row_cells[col].c);
    }
    if row < er {
      result.push('\n');
    }
  }
  result.trim_end().to_string()
}

fn calc_grid(width: f32, height: f32, font_size: f32, status_bar_visible: bool) -> (usize, usize) {
  let char_width = font_size * 0.62;
  let char_height = font_size * 1.29;
  let padding_y = if status_bar_visible { 118.0 } else { 96.0 };
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
  if let Some(shells) = &settings.general.shells {
    if !shells.is_empty() {
      return shells.clone();
    }
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
    let (cols, rows) = calc_grid(1024.0, 768.0, cfg.theme.font.size, cfg.status_bar.visible);
    Self {
      tabs: vec![Tab::new(0, cols, rows, default_shell)],
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
      hovered_url: None,
      shell_picker_open: false,
      shell_picker_anchor: 0.0,
      available_shells,
      settings_open: false,
      settings_tab: SettingsTab::General,
      settings: config::get().clone(),
      settings_shell_input: String::new(),
      settings_recording_index: None,
      raw_config_content: String::new(),
    }
  }
}

impl Nova {
  fn update_hovered_url(&mut self) {
    if !self.ctrl_held {
      self.hovered_url = None;
      return;
    }
    let font_size = self.settings.theme.font.size;
    let Some((col, row)) = pixel_to_cell(self.cursor_position, font_size) else {
      self.hovered_url = None;
      return;
    };
    let Some(tab) = self.tabs.get(self.active_index) else {
      self.hovered_url = None;
      return;
    };
    let Some(row_cells) = get_display_row(&tab.grid, tab.scroll_offset, row) else {
      self.hovered_url = None;
      return;
    };
    self.hovered_url = crate::core::url::detect_urls(row_cells)
      .into_iter()
      .find(|(start, end, _)| col >= *start && col <= *end)
      .map(|(_, _, url)| url);
  }

  fn resize_all_grids(&mut self) {
    let (cols, rows) = calc_grid(
      self.window_size.width,
      self.window_size.height,
      self.settings.theme.font.size,
      self.settings.status_bar.visible,
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
        if self.settings_open {
          return iced::Task::none();
        }
        self.selection_start = None;
        self.selection_end = None;
        if let Some(active_tab) = self.tabs.get_mut(self.active_index) {
          active_tab.scroll_offset = 0;
          if let Some(tx) = &active_tab.pty_tx {
            let _ = tx.send_blocking(PtyCommand::Input(bytes));
          }
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
        );
        self.tabs.push(Tab::new(new_id, cols, rows, shell));
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
          );
          self
            .tabs
            .push(Tab::new(self.next_tab_id, cols, rows, shell));
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
        }
      }
      Message::OpenSettings => {
        self.settings = config::get().clone();
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
        if let Some(shells) = &mut self.settings.general.shells {
          if i < shells.len() {
            shells.remove(i);
          }
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
        if let Some(idx) = self.settings_recording_index {
          if let Some(s) = keybinding_to_string(&key, modifiers) {
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
          if std::env::var("NOVA_DEBUG_PTY").is_ok() {
            if let Ok(mut f) = std::fs::OpenOptions::new()
              .create(true)
              .append(true)
              .open("C:\\Users\\Public\\nova_pty_debug.bin")
            {
              let _ = f.write_all(&bytes);
            }
          }
          let mut executor = AnsiExecutor {
            grid: &mut tab.grid,
          };
          for byte in bytes {
            tab.ansi_parser.advance(&mut executor, &[byte]);
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
        }
      }
      Message::WindowOpened(id) => {
        self.window_id = Some(id);
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
      Message::WindowResized(width, height) => {
        self.window_size = Size::new(width, height);
        let (cols, rows) = calc_grid(
          width,
          height,
          self.settings.theme.font.size,
          self.settings.status_bar.visible,
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
      }
      Message::CursorMoved(position) => {
        self.cursor_position = position;
        if self.is_selecting {
          let font_size = self.settings.theme.font.size;
          self.selection_end = pixel_to_cell(position, font_size);
        }
        self.update_hovered_url();
      }
      Message::MousePressed => {
        if self.settings_open {
          return iced::Task::none();
        }
        if self.ctrl_held {
          if let Some(url) = self.hovered_url.clone() {
            let _ = open::that_detached(&url);
            return iced::Task::none();
          }
        }
        if let Some(window_id) = self.window_id {
          if let Some(direction) = resize_direction(self.cursor_position, self.window_size) {
            return window::drag_resize(window_id, direction);
          }
        }
        let font_size = self.settings.theme.font.size;
        let cell = pixel_to_cell(self.cursor_position, font_size);
        self.selection_start = cell;
        self.selection_end = cell;
        self.is_selecting = cell.is_some();
      }
      Message::MouseReleased => {
        self.is_selecting = false;
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
          if start == end {
            self.selection_start = None;
            self.selection_end = None;
          } else if let Some(active_tab) = self.tabs.get(self.active_index) {
            let text = extract_selection(&active_tab.grid, start, end);
            if !text.is_empty() {
              return iced::clipboard::write(text);
            }
          }
        }
      }
      Message::CopySelection => {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
          if let Some(active_tab) = self.tabs.get(self.active_index) {
            let text = extract_selection(&active_tab.grid, start, end);
            if !text.is_empty() {
              return iced::clipboard::write(text);
            }
          }
        }
      }
      Message::PasteRequested => {
        return iced::clipboard::read().map(Message::ClipboardReceived);
      }
      Message::ClipboardReceived(text) => {
        if let Some(text) = text {
          if let Some(tab) = self.tabs.get(self.active_index) {
            if let Some(tx) = &tab.pty_tx {
              let _ = tx.try_send(PtyCommand::Input(text.into_bytes()));
            }
          }
        }
      }
      Message::Scroll(delta) => {
        if !self.settings_open {
          if let Some(tab) = self.tabs.get_mut(self.active_index) {
            let rows = (delta.abs() * 3.0).round() as usize;
            if delta > 0.0 {
              let new_offset = tab.scroll_offset.saturating_add(rows);
              tab.scroll_offset = new_offset.min(tab.grid.scrollback.len());
            } else {
              tab.scroll_offset = tab.scroll_offset.saturating_sub(rows);
            }
          }
        }
      }
      Message::ModifiersChanged(mods) => {
        self.ctrl_held = mods.command();
        self.update_hovered_url();
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

    let term_interaction = if self.hovered_url.is_some() {
      mouse::Interaction::Pointer
    } else {
      mouse::Interaction::Text
    };
    let term = mouse_area(
      components::term(active_tab, selection, font_size, active_tab.scroll_offset, self.hovered_url.as_deref()),
    )
    .interaction(term_interaction);

    let mut col = column![
      components::title_bar(self.window_focused, &active_tab.pwd, self.window_maximized),
      components::tab_bar(&self.tabs, self.active_index),
      term,
    ];

    if self.settings.status_bar.visible {
      col = col.push(components::status_bar(
        active_tab,
        &self.settings.status_bar.date_format,
        &self.settings.status_bar.time_format,
        self.window_maximized,
      ));
    }

    if self.settings_open {
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
      components::app(stack![col, modal])
    } else if self.shell_picker_open {
      let picker = components::shell_picker(
        &self.available_shells,
        self.shell_picker_anchor,
        self.window_size.width,
      );
      components::app(stack![col, picker])
    } else {
      components::app(col)
    }
  }

  pub fn theme(&self) -> Theme {
    Theme::Dracula
  }

  pub fn subscription(&self) -> Subscription<Message> {
    let mut subs = Vec::new();

    let time_sub = time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick);
    subs.push(time_sub);

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
          return None;
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
        drop(kb);

        match &key {
          Key::Named(Named::Enter) => return Some(Message::Type(b"\r".to_vec())),
          Key::Named(Named::Backspace) => return Some(Message::Type(b"\x7F".to_vec())),
          Key::Named(Named::Tab) => return Some(Message::Type(b"\t".to_vec())),
          Key::Named(Named::Space) => return Some(Message::Type(b" ".to_vec())),
          Key::Named(Named::Escape) => return Some(Message::Type(b"\x1b".to_vec())),
          Key::Named(Named::ArrowUp) => return Some(Message::Type(b"\x1b[A".to_vec())),
          Key::Named(Named::ArrowDown) => return Some(Message::Type(b"\x1b[B".to_vec())),
          Key::Named(Named::ArrowRight) => return Some(Message::Type(b"\x1b[C".to_vec())),
          Key::Named(Named::ArrowLeft) => return Some(Message::Type(b"\x1b[D".to_vec())),
          Key::Named(Named::Delete) => return Some(Message::Type(b"\x1b[3~".to_vec())),
          Key::Named(Named::Home) => return Some(Message::Type(b"\x1b[H".to_vec())),
          Key::Named(Named::End) => return Some(Message::Type(b"\x1b[F".to_vec())),
          Key::Named(Named::PageUp) => return Some(Message::Type(b"\x1b[5~".to_vec())),
          Key::Named(Named::PageDown) => return Some(Message::Type(b"\x1b[6~".to_vec())),
          _ => {}
        }

        if modifiers.control() {
          if let Key::Character(c) = &key {
            if let Some(ch) = c.as_str().chars().next() {
              if ch.is_ascii_alphabetic() {
                let lower = ch.to_ascii_lowercase();
                return Some(Message::Type(vec![(lower as u8) & 0x1f]));
              }
            }
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
      Event::Window(window::Event::Opened { .. }) => {
        return Some(Message::WindowOpened(window_id));
      }
      Event::Window(window::Event::Focused) => {
        return Some(Message::WindowFocused);
      }
      Event::Window(window::Event::Unfocused) => {
        return Some(Message::WindowUnfocused);
      }
      Event::Window(window::Event::Resized(size)) => {
        return Some(Message::WindowResized(size.width, size.height));
      }
      Event::Mouse(mouse::Event::CursorMoved { position }) => {
        return Some(Message::CursorMoved(position));
      }
      Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
        return Some(Message::MousePressed);
      }
      Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
        return Some(Message::MouseReleased);
      }
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
      let tab_id = tab.id;
      let cols = tab.grid.cols as u16;
      let rows = tab.grid.rows as u16;
      let shell_cmd = tab.shell_cmd.clone();
      let pty_sub = Subscription::run_with(
        (tab_id, cols, rows, shell_cmd),
        |(tab_id, cols, rows, shell)| pty_worker(*tab_id, *cols, *rows, shell.clone()),
      );
      subs.push(pty_sub);
    }

    Subscription::batch(subs)
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

fn pty_worker(
  tab_id: usize,
  cols: u16,
  rows: u16,
  shell: String,
) -> impl iced::futures::Stream<Item = Message> {
  stream::channel(
    100,
    move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
      use iced::futures::SinkExt;

      let (tx_out, rx_out) = async_channel::unbounded::<Vec<u8>>();
      let (tx_in, rx_in) = async_channel::unbounded::<PtyCommand>();

      std::thread::spawn(move || {
        let mut pty =
          PtyBridge::new(tx_out, cols, rows, &shell).expect("failed to create PTY bridge");

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
    },
  )
}
