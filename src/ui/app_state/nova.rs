use iced::{Point, Size, mouse, window};
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use crate::core::config;
use crate::ui::tab::Tab;

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
  pub(in crate::ui::app_state) tabs: Vec<Tab>,
  pub(in crate::ui::app_state) active_index: usize,
  pub(in crate::ui::app_state) next_tab_id: usize,
  pub(in crate::ui::app_state) window_id: Option<window::Id>,
  pub(in crate::ui::app_state) window_focused: bool,
  pub(in crate::ui::app_state) window_maximized: bool,
  pub(in crate::ui::app_state) window_size: Size,
  pub(in crate::ui::app_state) cursor_position: Point,
  pub(in crate::ui::app_state) selection_start: Option<(usize, usize)>,
  pub(in crate::ui::app_state) selection_end: Option<(usize, usize)>,
  pub(in crate::ui::app_state) is_selecting: bool,
  pub(in crate::ui::app_state) ctrl_held: bool,
  pub(in crate::ui::app_state) shift_held: bool,
  pub(in crate::ui::app_state) alt_held: bool,
  pub(in crate::ui::app_state) last_mouse_button: Option<mouse::Button>,
  pub(in crate::ui::app_state) click_count: u8,
  pub(in crate::ui::app_state) last_click_time: Instant,
  pub(in crate::ui::app_state) last_click_cell: Option<(usize, usize)>,
  pub(in crate::ui::app_state) hovered_url: Option<String>,
  pub(in crate::ui::app_state) hovered_link_span: Option<(usize, usize, usize)>,
  pub(in crate::ui::app_state) shell_picker_open: bool,
  pub(in crate::ui::app_state) shell_picker_anchor: f32,
  pub(in crate::ui::app_state) available_shells: Vec<String>,
  pub(in crate::ui::app_state) settings_open: bool,
  pub(in crate::ui::app_state) settings_tab: SettingsTab,
  pub(in crate::ui::app_state) settings: config::Config,
  pub(in crate::ui::app_state) settings_shell_input: String,
  pub(in crate::ui::app_state) settings_recording_index: Option<usize>,
  pub(in crate::ui::app_state) raw_config_content: String,
  pub(in crate::ui::app_state) command_palette_open: bool,
  pub(in crate::ui::app_state) palette_query: String,
  pub(in crate::ui::app_state) palette_selected: usize,
  pub(in crate::ui::app_state) ai_overlay_open: bool,
  pub(in crate::ui::app_state) ai_input: String,
  pub(in crate::ui::app_state) ai_loading: bool,
  pub(in crate::ui::app_state) ai_response: Option<String>,
  pub(in crate::ui::app_state) ai_is_error: bool,
  pub(in crate::ui::app_state) diagnostic_banner: Option<(u8, String, Option<String>)>,
  pub(in crate::ui::app_state) ai_pending_diagnostic: Option<u8>,
  pub(in crate::ui::app_state) bell_blink_visible: bool,
  pub(in crate::ui::app_state) bell_blink_remaining: u8,
  pub(in crate::ui::app_state) resize_generation: u64,
  pub(in crate::ui::app_state) font_resize_generation: u64,
}
