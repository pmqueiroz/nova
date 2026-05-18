use iced::keyboard::key::Named;
use iced::{Point, Size, keyboard, mouse, window};
use std::path::PathBuf;

use crate::core::config::{self, KeyId, ParsedKeybinding};
use crate::core::grid;

pub const RESIZE_EDGE: f32 = 8.0;
pub const SPLIT_DIVIDER_WIDTH: f32 = 8.0;

const CHAR_WIDTH_RATIO: f32 = 0.62;
const CHAR_HEIGHT_RATIO: f32 = 1.29;
const BANNER_HEIGHT_RATIO: f32 = 2.5;
const TERM_PADDING_X: f32 = 40.0;
const PADDING_Y_WITH_STATUSBAR: f32 = 118.0;
const PADDING_Y_NO_STATUSBAR: f32 = 96.0;
const MIN_COLS: usize = 10;
const MIN_SPLIT_COLS: usize = 5;
const MIN_ROWS: usize = 5;

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

pub fn calc_grid(
  width: f32,
  height: f32,
  font_size: f32,
  status_bar_visible: bool,
  banner_visible: bool,
) -> (usize, usize) {
  let char_width = font_size * CHAR_WIDTH_RATIO;
  let char_height = font_size * CHAR_HEIGHT_RATIO;
  let banner_extra = if banner_visible {
    font_size * BANNER_HEIGHT_RATIO
  } else {
    0.0
  };
  let padding_y = if status_bar_visible {
    PADDING_Y_WITH_STATUSBAR
  } else {
    PADDING_Y_NO_STATUSBAR
  } + banner_extra;
  let cols = ((width - TERM_PADDING_X) / char_width).floor() as usize;
  let rows = ((height - padding_y) / char_height).floor() as usize;
  (cols.max(MIN_COLS), rows.max(MIN_ROWS))
}

pub fn calc_grid_split(
  total_width: f32,
  height: f32,
  font_size: f32,
  status_bar_visible: bool,
  banner_visible: bool,
) -> (usize, usize) {
  let char_width = font_size * CHAR_WIDTH_RATIO;
  let char_height = font_size * CHAR_HEIGHT_RATIO;
  let banner_extra = if banner_visible {
    font_size * BANNER_HEIGHT_RATIO
  } else {
    0.0
  };
  let padding_y = if status_bar_visible {
    PADDING_Y_WITH_STATUSBAR
  } else {
    PADDING_Y_NO_STATUSBAR
  } + banner_extra;
  let pane_width = ((total_width - SPLIT_DIVIDER_WIDTH) / 2.0).max(0.0);
  let cols = ((pane_width - TERM_PADDING_X).max(0.0) / char_width).floor() as usize;
  let rows = ((height - padding_y) / char_height).max(0.0).floor() as usize;
  (cols.max(MIN_COLS), rows.max(MIN_ROWS))
}

pub fn calc_grid_split_ratio(
  total_width: f32,
  height: f32,
  font_size: f32,
  status_bar_visible: bool,
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
  let padding_y = if status_bar_visible {
    PADDING_Y_WITH_STATUSBAR
  } else {
    PADDING_Y_NO_STATUSBAR
  } + banner_extra;
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

pub fn pixel_to_cell(pos: Point, font_size: f32) -> Option<(usize, usize)> {
  const X_ORIGIN: f32 = 20.0;
  const Y_ORIGIN: f32 = 88.0;
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

  // top edge belongs to the title bar (drag-to-move), never resize from there
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
    foreground_muted: parse(&colors.foreground_muted),
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

pub fn strip_markdown(text: &str) -> String {
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
