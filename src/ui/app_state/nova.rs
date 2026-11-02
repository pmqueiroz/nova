use iced::animation::Animation;
use iced::{Point, Size, mouse, window};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering};
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
pub static CONFIRM_OPEN: AtomicBool = AtomicBool::new(false);
pub static SHELL_PICKER_OPEN: AtomicBool = AtomicBool::new(false);
pub static TRACK_POINTER: AtomicBool = AtomicBool::new(false);
static POINTER: AtomicU64 = AtomicU64::new(0);
static WINDOW_SIZE: AtomicU64 = AtomicU64::new(0);
static POINTER_ZONE: AtomicU8 = AtomicU8::new(0);

fn pack(a: f32, b: f32) -> u64 {
  (u64::from(a.to_bits()) << 32) | u64::from(b.to_bits())
}

fn unpack(v: u64) -> (f32, f32) {
  (f32::from_bits((v >> 32) as u32), f32::from_bits(v as u32))
}

pub fn latest_pointer() -> Point {
  let (x, y) = unpack(POINTER.load(Ordering::Relaxed));
  Point::new(x, y)
}

pub fn publish_window_size(size: Size) {
  WINDOW_SIZE.store(pack(size.width, size.height), Ordering::Relaxed);
}

fn zone_code(dir: Option<window::Direction>) -> u8 {
  match dir {
    None => 0,
    Some(window::Direction::North) => 1,
    Some(window::Direction::South) => 2,
    Some(window::Direction::East) => 3,
    Some(window::Direction::West) => 4,
    Some(window::Direction::NorthEast) => 5,
    Some(window::Direction::NorthWest) => 6,
    Some(window::Direction::SouthEast) => 7,
    Some(window::Direction::SouthWest) => 8,
  }
}

pub fn pointer_moved(position: Point) -> bool {
  POINTER.store(pack(position.x, position.y), Ordering::Relaxed);
  let (w, h) = unpack(WINDOW_SIZE.load(Ordering::Relaxed));
  let zone = zone_code(super::helpers::resize_direction(position, Size::new(w, h)));
  let zone_changed = POINTER_ZONE.swap(zone, Ordering::Relaxed) != zone;
  zone_changed || TRACK_POINTER.load(Ordering::Relaxed)
}

pub const ANIM_TAIL: std::time::Duration = std::time::Duration::from_millis(400);

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
  Appearance,
  Shell,
  Keys,
  Ai,
  About,
  Raw,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Confirm {
  Paste(String),
  Restart,
}

#[derive(Debug, Clone)]
pub enum ColorField {
  Accent,
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
  pub(in crate::ui::app_state) shell_picker_selected: usize,
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
  pub(in crate::ui::app_state) ai_diagnostic_inflight: Option<u8>,
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
  pub(in crate::ui::app_state) drag_host_tab: Option<usize>,
  pub(in crate::ui::app_state) split_drop: Option<crate::ui::tab::PaneSide>,
  pub(in crate::ui::app_state) confirm: Option<Confirm>,
  pub(in crate::ui::app_state) now: Instant,
  pub(in crate::ui::app_state) sheet_opened_at: Instant,
  pub(in crate::ui::app_state) timestamps: bool,
  pub(in crate::ui::app_state) block_selection: Option<crate::core::selection::BlockSelection>,
  pub(in crate::ui::app_state) block_pointer: Option<(usize, u64, (usize, usize))>,
  pub(in crate::ui::app_state) block_url: Option<crate::ui::components::session::BlockUrl>,
  pub(in crate::ui::app_state) window_pop: Animation<bool>,
  pub(in crate::ui::app_state) update_status: crate::core::update::Status,
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

  pub(in crate::ui::app_state) fn any_integration_detecting(&self) -> bool {
    use crate::core::router::Mode;
    self.tabs.iter().any(|t| {
      t.integration.mode == Mode::Detecting
        || t
          .split
          .as_ref()
          .is_some_and(|s| s.integration.mode == Mode::Detecting)
    })
  }

  pub(in crate::ui::app_state) fn needs_pointer_moves(&self) -> bool {
    self.dragging_split
      || self.drag_pending_pos.is_some()
      || self.dragging_tab_index.is_some()
      || self.is_selecting
      || self.ctrl_held
      || self.hovered_url.is_some()
      || self
        .tabs
        .get(self.active_index)
        .is_some_and(|t| t.grid.mouse_mode != crate::core::grid::MouseMode::None)
  }

  pub(in crate::ui::app_state) fn needs_cursor_blink(&self) -> bool {
    if !self.cursor_blink_visible {
      return true;
    }
    if !self.settings.theme.cursor.blink {
      return false;
    }
    self.tabs.get(self.active_index).is_some_and(|t| {
      [false, true]
        .into_iter()
        .filter(|&split| !split || t.split.is_some())
        .any(|split| {
          let pane = t.pane(split);
          pane.integration.mode == crate::core::router::Mode::Classic || pane.raw_grid().is_some()
        })
    })
  }

  pub(in crate::ui::app_state) fn any_sheet_open(&self) -> bool {
    self.command_palette_open
      || self.settings_open
      || self.ai_overlay_open
      || self.shell_picker_open
      || self.confirm.is_some()
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
