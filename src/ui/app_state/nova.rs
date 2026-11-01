use iced::animation::Animation;
use iced::{Point, Size, mouse, window};
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::time::Instant;

use crate::core::config;
use crate::ui::tab::Tab;

pub static SETTINGS_OPEN: AtomicBool = AtomicBool::new(false);
pub static ACTIVE_KITTY_FLAGS: AtomicU32 = AtomicU32::new(0);
pub static KB_RECORDING: AtomicBool = AtomicBool::new(false);
pub static PALETTE_OPEN: AtomicBool = AtomicBool::new(false);
pub static AI_OPEN: AtomicBool = AtomicBool::new(false);
pub static SEARCH_OPEN: AtomicBool = AtomicBool::new(false);
pub static INPUT_OWNS_KEYS: AtomicBool = AtomicBool::new(false);
pub static HAS_BLOCK_SELECTION: AtomicBool = AtomicBool::new(false);

pub const ANIM_TAIL: std::time::Duration = std::time::Duration::from_millis(400);

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
  General,
  Theme,
  Keybindings,
  Ai,
  Raw,
  About,
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
  pub(in crate::ui::app_state) ai_selected: usize,
  pub(in crate::ui::app_state) ai_answer: Option<crate::ui::components::AiAnswer>,
  pub(in crate::ui::app_state) diagnostic_banner: Option<(u8, String, Option<String>)>,
  pub(in crate::ui::app_state) ai_pending_diagnostic: Option<u8>,
  pub(in crate::ui::app_state) bell_blink_visible: bool,
  pub(in crate::ui::app_state) bell_blink_remaining: u8,
  pub(in crate::ui::app_state) cursor_blink_visible: bool,
  pub(in crate::ui::app_state) resize_generation: u64,
  pub(in crate::ui::app_state) font_resize_generation: u64,
  pub(in crate::ui::app_state) default_font_size: f32,
  pub(in crate::ui::app_state) dragging_split: bool,
  pub(in crate::ui::app_state) search_active: bool,
  pub(in crate::ui::app_state) search_query: String,
  pub(in crate::ui::app_state) search_matches: Vec<(bool, usize, usize, usize)>,
  pub(in crate::ui::app_state) block_matches: Vec<crate::core::search::BlockMatch>,
  pub(in crate::ui::app_state) search_match_index: usize,
  pub(in crate::ui::app_state) search_generation: u64,
  pub(in crate::ui::app_state) dragging_tab_index: Option<usize>,
  pub(in crate::ui::app_state) hovered_tab: Option<usize>,
  pub(in crate::ui::app_state) drag_pending_tab: Option<usize>,
  pub(in crate::ui::app_state) drag_pending_pos: Option<Point>,
  pub(in crate::ui::app_state) paste_warning: Option<String>,
  pub(in crate::ui::app_state) now: Instant,
  pub(in crate::ui::app_state) sheet_opened_at: Instant,
  pub(in crate::ui::app_state) timestamps: bool,
  pub(in crate::ui::app_state) block_selection: Option<crate::core::selection::BlockSelection>,
  pub(in crate::ui::app_state) block_pointer: Option<(usize, u64, (usize, usize))>,
  pub(in crate::ui::app_state) block_url: Option<crate::ui::components::session::BlockUrl>,
  pub(in crate::ui::app_state) window_pop: Animation<bool>,
}

impl Nova {
  fn pane_animating(&self, tab: &Tab, split: bool) -> bool {
    let pane = tab.pane(split);
    if pane.integration.mode == crate::core::router::Mode::Classic {
      return false;
    }
    let now = self.now;
    pane
      .session
      .blocks
      .iter()
      .any(|b| crate::ui::components::block::is_animating(b, now))
      || pane
        .input
        .pulsed_at
        .is_some_and(|p| now.saturating_duration_since(p) < ANIM_TAIL)
      || now.saturating_duration_since(pane.integration.spawned_at) < ANIM_TAIL * 12
  }

  pub(in crate::ui::app_state) fn any_sheet_open(&self) -> bool {
    self.command_palette_open
      || self.settings_open
      || self.ai_overlay_open
      || self.shell_picker_open
      || self.paste_warning.is_some()
  }

  pub(in crate::ui::app_state) fn sheet_progress(&self) -> f32 {
    let t = self
      .now
      .saturating_duration_since(self.sheet_opened_at)
      .as_secs_f32()
      / crate::ui::theme::tokens::duration::D2.as_secs_f32();
    if t >= 1.0 {
      1.0
    } else {
      crate::ui::theme::motion::spring(t.max(0.0))
    }
  }

  pub(in crate::ui::app_state) fn is_animating(&self) -> bool {
    self.window_pop.is_animating(self.now)
      || (self.ai_overlay_open
        && self
          .ai_answer
          .as_ref()
          .is_some_and(|a| a.loading || !a.is_done(self.now)))
      || (self.any_sheet_open()
        && self.now.saturating_duration_since(self.sheet_opened_at) < ANIM_TAIL)
      || self.tabs.iter().any(|t| t.appear.is_animating(self.now))
      || self.tabs.get(self.active_index).is_some_and(|t| {
        [false, true]
          .into_iter()
          .any(|split| self.pane_animating(t, split))
      })
  }
}
