use iced::keyboard::key::Named;
use iced::{Point, Size, keyboard, mouse, window};
use std::path::PathBuf;

use crate::core::config::{self, KeyId, ParsedKeybinding};
use crate::core::grid;
use crate::ui::theme::tokens;

pub const RESIZE_EDGE: f32 = 8.0;
pub const SPLIT_DIVIDER_WIDTH: f32 = 8.0;

const CHAR_WIDTH_RATIO: f32 = 0.62;
const CHAR_HEIGHT_RATIO: f32 = 1.29;
const BANNER_HEIGHT_RATIO: f32 = 2.5;
const TERM_PADDING_X: f32 = 40.0;
const PADDING_Y: f32 = 76.0;
const MIN_COLS: usize = 10;
const MIN_SPLIT_COLS: usize = 5;
const MIN_ROWS: usize = 5;
pub const INITIAL_WINDOW_SIZE: Size = Size::new(1200.0, 750.0);
const WINDOW_ASPECT: f32 = 1.6;
const WINDOW_WIDTH_FRACTION: f32 = 0.65;
const WINDOW_HEIGHT_FRACTION: f32 = 0.8;

pub fn fit_window_size(monitor: Size) -> Size {
  let width = (monitor.width * WINDOW_WIDTH_FRACTION)
    .min(monitor.height * WINDOW_HEIGHT_FRACTION * WINDOW_ASPECT);
  Size::new(width.round(), (width / WINDOW_ASPECT).round())
}

pub fn recenter(position: Point, from: Size, to: Size) -> Point {
  Point::new(
    position.x + (from.width - to.width) / 2.0,
    position.y + (from.height - to.height) / 2.0,
  )
}

pub fn get_display_row(grid: &grid::Grid, scroll_offset: usize, y: usize) -> Option<&[grid::Cell]> {
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

pub fn stitch_continuation(
  base: String,
  grid: &grid::Grid,
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
    url.extend(cells[..cont].iter().flat_map(|c| c.c.chars()));
    end_row = r;
    if cont < cells.len() {
      break;
    }
    r += 1;
  }
  (url, end_row)
}

pub fn resolve_hovered_url(
  grid: &grid::Grid,
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

pub fn normalize_sel(
  start: (usize, usize),
  end: (usize, usize),
) -> ((usize, usize), (usize, usize)) {
  let (sc, sr) = start;
  let (ec, er) = end;
  if sr < er || (sr == er && sc <= ec) {
    (start, end)
  } else {
    (end, start)
  }
}

pub fn find_word_boundaries(row_cells: &[grid::Cell], col: usize) -> (usize, usize) {
  let is_word = |c: char| c.is_alphanumeric() || c == '_';
  let col = col.min(row_cells.len().saturating_sub(1));
  let clicked_is_word = row_cells
    .get(col)
    .map(|c| is_word(c.c.chars().next().unwrap_or(' ')))
    .unwrap_or(false);

  let start = (0..=col)
    .rev()
    .find(|&i| {
      let c = row_cells
        .get(i)
        .map(|c| c.c.chars().next().unwrap_or(' '))
        .unwrap_or(' ');
      is_word(c) != clicked_is_word
    })
    .map(|i| i + 1)
    .unwrap_or(0);

  let end = (col..row_cells.len())
    .find(|&i| {
      let c = row_cells
        .get(i)
        .map(|c| c.c.chars().next().unwrap_or(' '))
        .unwrap_or(' ');
      is_word(c) != clicked_is_word
    })
    .map(|i| i.saturating_sub(1))
    .unwrap_or(row_cells.len().saturating_sub(1));

  (start, end.max(start))
}

pub fn extract_selection(
  grid: &grid::Grid,
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
      result.push_str(&cell.c);
    }
    if row < er && !grid.row_continuation[row + 1] {
      result.push('\n');
    }
  }
  result.trim_end().to_string()
}

pub fn calc_grid(width: f32, height: f32, font_size: f32, banner_visible: bool) -> (usize, usize) {
  let char_width = font_size * CHAR_WIDTH_RATIO;
  let char_height = font_size * CHAR_HEIGHT_RATIO;
  let banner_extra = if banner_visible {
    font_size * BANNER_HEIGHT_RATIO
  } else {
    0.0
  };
  let padding_y = PADDING_Y + banner_extra;
  let cols = ((width - TERM_PADDING_X) / char_width).floor() as usize;
  let rows = ((height - padding_y) / char_height).floor() as usize;
  (cols.max(MIN_COLS), rows.max(MIN_ROWS))
}

pub fn calc_grid_split(
  total_width: f32,
  height: f32,
  font_size: f32,
  banner_visible: bool,
) -> (usize, usize) {
  let char_width = font_size * CHAR_WIDTH_RATIO;
  let char_height = font_size * CHAR_HEIGHT_RATIO;
  let banner_extra = if banner_visible {
    font_size * BANNER_HEIGHT_RATIO
  } else {
    0.0
  };
  let padding_y = PADDING_Y + banner_extra;
  let pane_width = ((total_width - SPLIT_DIVIDER_WIDTH) / 2.0).max(0.0);
  let cols = ((pane_width - TERM_PADDING_X).max(0.0) / char_width).floor() as usize;
  let rows = ((height - padding_y) / char_height).max(0.0).floor() as usize;
  (cols.max(MIN_COLS), rows.max(MIN_ROWS))
}

pub fn calc_grid_split_ratio(
  total_width: f32,
  height: f32,
  font_size: f32,
  banner_visible: bool,
  ratio: f32,
) -> (usize, usize, usize) {
  let char_width = font_size * CHAR_WIDTH_RATIO;
  let char_height = font_size * CHAR_HEIGHT_RATIO;
  let banner_extra = if banner_visible {
    font_size * BANNER_HEIGHT_RATIO
  } else {
    0.0
  };
  let padding_y = PADDING_Y + banner_extra;
  let avail_width = (total_width - SPLIT_DIVIDER_WIDTH).max(0.0);
  let left_cols = (((avail_width * ratio) - TERM_PADDING_X).max(0.0) / char_width).floor() as usize;
  let right_cols =
    (((avail_width * (1.0 - ratio)) - TERM_PADDING_X).max(0.0) / char_width).floor() as usize;
  let rows = ((height - padding_y) / char_height).max(0.0).floor() as usize;
  (
    left_cols.max(MIN_SPLIT_COLS),
    right_cols.max(MIN_SPLIT_COLS),
    rows.max(MIN_ROWS),
  )
}

pub fn block_output_width(pane_width: f32, timestamps: bool, compact: bool) -> f32 {
  use tokens::space::{S16, S24, S32, S48};
  let ts = if timestamps { S48 + S16 } else { 0.0 };
  let (scroll, card) = if compact { (S16, S16) } else { (S32, S24) };
  (pane_width - 2.0 * scroll - 2.0 * card - ts).max(0.0)
}

pub fn block_viewport_height(window_height: f32) -> f32 {
  use crate::ui::components::TITLE_BAR_HEIGHT;
  use tokens::height::H56;
  use tokens::space::{S16, S24, S32};
  (window_height - TITLE_BAR_HEIGHT - (H56 + S16) - S16 - S24 - S32).max(0.0)
}

pub fn calc_block_grid(
  pane_width: f32,
  window_height: f32,
  font_size: f32,
  timestamps: bool,
  compact: bool,
) -> (usize, usize) {
  let fs = tokens::block::font_size(font_size);
  let cols = (block_output_width(pane_width, timestamps, compact) / (fs * CHAR_WIDTH_RATIO)).floor()
    as usize;
  let rows =
    (block_viewport_height(window_height) / (fs * tokens::block::LINE_HEIGHT)).floor() as usize;
  (cols.max(MIN_COLS), rows.max(MIN_ROWS))
}

pub fn calc_raw_grid(
  pane_width: f32,
  window_height: f32,
  font_size: f32,
  compact: bool,
) -> (usize, usize) {
  let fs = tokens::block::font_size(font_size);
  let pad = crate::ui::components::session::raw_padding(compact);
  let width = (pane_width - pad.left - pad.right).max(0.0);
  let height =
    (window_height - crate::ui::components::TITLE_BAR_HEIGHT - pad.top - pad.bottom).max(0.0);
  let cols = (width / (fs * CHAR_WIDTH_RATIO)).floor() as usize;
  let rows = (height / (fs * CHAR_HEIGHT_RATIO)).floor() as usize;
  (cols.max(MIN_COLS), rows.max(MIN_ROWS))
}

pub fn raw_pixel_to_cell(pos: Point, pane_x: f32, compact: bool, font_size: f32) -> (usize, usize) {
  let fs = tokens::block::font_size(font_size);
  let pad = crate::ui::components::session::raw_padding(compact);
  let x = (pos.x - pane_x - pad.left).max(0.0);
  let y = (pos.y - crate::ui::components::TITLE_BAR_HEIGHT - pad.top).max(0.0);
  (
    (x / (fs * CHAR_WIDTH_RATIO)).floor() as usize,
    (y / (fs * CHAR_HEIGHT_RATIO)).floor() as usize,
  )
}

pub fn wheel_report(cb: u8, col: usize, row: usize, sgr: bool) -> Vec<u8> {
  if sgr {
    format!("\x1b[<{};{};{}M", cb, col + 1, row + 1).into_bytes()
  } else {
    let cb_byte = cb.saturating_add(32);
    let cx = (col + 1 + 32).min(255) as u8;
    let cy = (row + 1 + 32).min(255) as u8;
    vec![b'\x1b', b'[', b'M', cb_byte, cx, cy]
  }
}

pub fn alternate_scroll_keys(delta: f32) -> Vec<u8> {
  let lines = ((delta.abs() * 3.0).round() as usize).max(1);
  let key: &[u8] = if delta > 0.0 { b"\x1b[A" } else { b"\x1b[B" };
  key.repeat(lines)
}

pub fn split_widths(total_width: f32, ratio: f32) -> (f32, f32) {
  let avail = (total_width - SPLIT_DIVIDER_WIDTH).max(0.0);
  (avail * ratio, avail * (1.0 - ratio))
}

pub fn pixel_to_cell(pos: Point, font_size: f32) -> Option<(usize, usize)> {
  const X_ORIGIN: f32 = 20.0;
  const Y_ORIGIN: f32 = 68.0;
  if pos.y < Y_ORIGIN || pos.x < X_ORIGIN {
    return None;
  }
  let col = ((pos.x - X_ORIGIN) / (font_size * CHAR_WIDTH_RATIO)).floor() as usize;
  let row = ((pos.y - Y_ORIGIN) / (font_size * CHAR_HEIGHT_RATIO)).floor() as usize;
  Some((col, row))
}

pub fn resize_direction(pos: Point, size: Size) -> Option<window::Direction> {
  let left = pos.x < RESIZE_EDGE;
  let right = pos.x > size.width - RESIZE_EDGE;
  let top = pos.y < RESIZE_EDGE;
  let bottom = pos.y > size.height - RESIZE_EDGE;

  if top {
    return None;
  }

  match (bottom, left, right) {
    (true, true, _) => Some(window::Direction::SouthWest),
    (true, _, true) => Some(window::Direction::SouthEast),
    (true, false, false) => Some(window::Direction::South),
    (false, true, _) => Some(window::Direction::West),
    (false, _, true) => Some(window::Direction::East),
    _ => None,
  }
}

pub fn dir_to_cursor(dir: window::Direction) -> mouse::Interaction {
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

pub fn matches_kb(kb: &ParsedKeybinding, key: &keyboard::Key, mods: keyboard::Modifiers) -> bool {
  if kb.ctrl != mods.control()
    || kb.shift != mods.shift()
    || kb.alt != mods.alt()
    || kb.meta != mods.logo()
  {
    return false;
  }
  match (&kb.key, key) {
    (KeyId::Tab, keyboard::Key::Named(Named::Tab)) => true,
    (KeyId::Char(c), keyboard::Key::Character(s)) => s
      .as_str()
      .chars()
      .next()
      .map(|sc| sc == *c)
      .unwrap_or(false),
    _ => false,
  }
}

pub fn keybinding_to_string(key: &keyboard::Key, mods: keyboard::Modifiers) -> Option<String> {
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
    keyboard::Key::Named(Named::Tab) => parts.push("tab"),
    keyboard::Key::Character(s) => {
      let lower = s.as_str().to_ascii_lowercase();
      let leaked: &'static str = Box::leak(lower.into_boxed_str());
      parts.push(leaked);
    }
    _ => return None,
  }
  Some(parts.join("+"))
}

pub fn derive_available_shells(settings: &config::Config) -> Vec<String> {
  if let Some(shells) = &settings.general.shells
    && !shells.is_empty()
  {
    return shells.clone();
  }
  config::detect_shells()
}

pub fn rebuild_runtime_theme(colors: &config::ThemeColorsConfig) {
  use crate::ui::theme::color::{RuntimeTheme, update_runtime};
  let parse = |h: &str| config::parse_hex_color(h).unwrap_or(iced::Color::BLACK);
  update_runtime(RuntimeTheme {
    background: parse(&colors.background),
    foreground: parse(&colors.foreground),
    accent: parse(&colors.accent),
    border: parse(&colors.border),
    cursor: parse(&colors.cursor),
  });
}

pub fn command_history_path() -> Option<PathBuf> {
  let data_dir = dirs::data_dir()?;
  let dir = data_dir.join("nova");
  let _ = std::fs::create_dir_all(&dir);
  Some(dir.join("command_history.bin"))
}

pub fn os_name() -> String {
  match std::env::consts::OS {
    "macos" => "macOS".to_string(),
    "windows" => "Windows".to_string(),
    "linux" => "Linux".to_string(),
    other => other.to_string(),
  }
}

#[cfg(test)]
mod block_grid_tests {
  use super::*;

  #[test]
  fn alternate_scroll_sends_three_arrows_per_notch() {
    assert_eq!(alternate_scroll_keys(1.0), b"\x1b[A\x1b[A\x1b[A".to_vec());
    assert_eq!(alternate_scroll_keys(-0.1), b"\x1b[B".to_vec());
  }

  #[test]
  fn wheel_report_encodes_sgr_and_x10() {
    assert_eq!(wheel_report(64, 0, 4, true), b"\x1b[<64;1;5M".to_vec());
    assert_eq!(
      wheel_report(65, 0, 0, false),
      vec![0x1b, b'[', b'M', 97, 33, 33]
    );
  }

  #[test]
  fn block_grid_uses_smaller_output_font() {
    let (c16, _) = calc_block_grid(1024.0, 768.0, 16.0, false, false);
    let expected = ((1024.0 - 64.0 - 48.0) / (14.0 * CHAR_WIDTH_RATIO)).floor() as usize;
    assert_eq!(c16, expected);
  }

  #[test]
  fn timestamps_column_narrows_output() {
    let (a, _) = calc_block_grid(1024.0, 768.0, 16.0, false, false);
    let (b, _) = calc_block_grid(1024.0, 768.0, 16.0, true, false);
    assert!(b < a);
  }

  #[test]
  fn compact_layout_gains_columns() {
    let (wide, _) = calc_block_grid(600.0, 768.0, 16.0, false, false);
    let (compact, _) = calc_block_grid(600.0, 768.0, 16.0, false, true);
    assert!(compact > wide);
  }

  #[test]
  fn tiny_window_keeps_minimums() {
    assert_eq!(
      calc_block_grid(10.0, 10.0, 16.0, false, false),
      (MIN_COLS, MIN_ROWS)
    );
  }
}

#[cfg(test)]
mod window_fit_tests {
  use super::*;

  #[test]
  fn fits_scaled_1080p_without_overflow() {
    let s = fit_window_size(Size::new(1280.0, 720.0));
    assert_eq!(s, Size::new(832.0, 520.0));
  }

  #[test]
  fn height_bound_on_tall_ratio_limits() {
    let s = fit_window_size(Size::new(3440.0, 1440.0));
    assert!(s.height <= 1440.0 * 0.8);
    assert!((s.width / s.height - 1.6).abs() < 0.01);
  }

  #[test]
  fn recenter_keeps_center() {
    let p = recenter(
      Point::new(100.0, 50.0),
      Size::new(1200.0, 750.0),
      Size::new(800.0, 500.0),
    );
    assert_eq!(p, Point::new(300.0, 175.0));
  }
}
