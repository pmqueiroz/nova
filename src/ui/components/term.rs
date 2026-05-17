use iced::{
  Background, Color, Element, Length, Padding,
  widget::{
    column, container, rich_text, row,
    text::{self, Span},
  },
};

use crate::core::grid::Grid;
use crate::core::url::detect_urls;
use crate::ui::{app_state::Message, theme};

fn in_selection(x: usize, y: usize, sel: Option<(usize, usize, usize, usize)>) -> bool {
  let (sc, sr, ec, er) = match sel {
    Some(s) => s,
    None => return false,
  };
  if sr == er {
    y == sr && x >= sc && x <= ec
  } else if y == sr {
    x >= sc
  } else if y == er {
    x <= ec
  } else {
    y > sr && y < er
  }
}

fn is_url_terminator(c: char) -> bool {
  c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | ')' | ']')
}

fn compute_url_highlight(
  row_cells: &[crate::core::grid::Cell],
  display_y: usize,
  hovered_url: Option<&str>,
  hovered_link_span: Option<(usize, usize, usize)>,
) -> Vec<bool> {
  let Some(url) = hovered_url else {
    return vec![];
  };
  match hovered_link_span {
    Some((span_start, span_col, span_end)) => {
      let mut v = vec![false; row_cells.len()];
      let scan_from = if display_y == span_start {
        span_col
      } else if display_y > span_start && display_y <= span_end {
        0
      } else {
        return v;
      };
      let mut i = scan_from;
      while i < row_cells.len() && !is_url_terminator(row_cells[i].c) {
        v[i] = true;
        i += 1;
      }
      v
    }
    None => {
      let mut v = vec![false; row_cells.len()];
      for (start, end, u) in detect_urls(row_cells) {
        if u == url {
          for i in start..=end.min(v.len().saturating_sub(1)) {
            v[i] = true;
          }
        }
      }
      v
    }
  }
}

const SEARCH_MATCH_BG: Color = Color {
  r: 0.98,
  g: 0.76,
  b: 0.0,
  a: 0.35,
};
const SEARCH_CURRENT_BG: Color = Color {
  r: 0.98,
  g: 0.76,
  b: 0.0,
  a: 1.0,
};
const SEARCH_CURRENT_FG: Color = Color {
  r: 0.1,
  g: 0.1,
  b: 0.1,
  a: 1.0,
};

#[allow(clippy::too_many_arguments)]
fn row_spans(
  row_cells: &[crate::core::grid::Cell],
  cursor_col: Option<usize>,
  cursor_color: Color,
  y: usize,
  selection: Option<(usize, usize, usize, usize)>,
  font_size: f32,
  hovered_url: Option<&str>,
  url_highlight: &[bool],
  suggestion: Option<&str>,
  search_hl: Option<(&[bool], &[bool])>,
) -> Vec<Span<'static>> {
  let mut spans: Vec<Span<'static>> = Vec::new();

  for (x, cell) in row_cells.iter().enumerate() {
    let is_cursor = cursor_col == Some(x);
    let is_selected = in_selection(x, y, selection);
    let is_url_hovered = if let Some(hover) = hovered_url {
      cell.uri.as_deref() == Some(hover) || url_highlight.get(x).copied().unwrap_or(false)
    } else {
      false
    };
    let is_search_current = search_hl
      .map(|(_, cur)| cur.get(x).copied().unwrap_or(false))
      .unwrap_or(false);
    let is_search_match = !is_search_current
      && search_hl
        .map(|(m, _)| m.get(x).copied().unwrap_or(false))
        .unwrap_or(false);

    let (mut eff_fg, eff_bg, mut eff_attrs) = if is_selected {
      let rt = theme::color::runtime();
      (
        Some(rt.background),
        Some(rt.accent),
        crate::core::grid::CellAttrs::empty(),
      )
    } else if is_search_current {
      (
        Some(SEARCH_CURRENT_FG),
        Some(SEARCH_CURRENT_BG),
        crate::core::grid::CellAttrs::empty(),
      )
    } else if is_search_match {
      (cell.fg, Some(SEARCH_MATCH_BG), cell.attrs)
    } else {
      (cell.fg, cell.bg, cell.attrs)
    };

    if is_url_hovered {
      eff_attrs.insert(crate::core::grid::CellAttrs::UNDERLINE);
    }

    if is_cursor {
      if let Some(sugg) = suggestion {
        let max_chars = row_cells.len().saturating_sub(x);
        let mut chars = sugg.chars().take(max_chars);
        if let Some(first) = chars.next() {
          spans.push(
            Span::new(first.to_string())
              .color(cursor_color)
              .underline(true)
              .font(theme::font::REGULAR)
              .size(font_size),
          );
          let rt = theme::color::runtime();
          let mut dim_color = rt.foreground;
          dim_color.a = 0.35;
          for c in chars {
            spans.push(
              Span::new(c.to_string())
                .color(dim_color)
                .font(theme::font::REGULAR)
                .size(font_size),
            );
          }
        }
        break;
      } else {
        eff_fg = Some(cursor_color);
        eff_attrs.insert(crate::core::grid::CellAttrs::UNDERLINE);
      }
    }
    spans.push(cell_span(
      cell.c.to_string(),
      eff_fg,
      eff_bg,
      eff_attrs,
      font_size,
    ));
  }

  spans
}

#[allow(clippy::too_many_arguments)]
pub fn term<'a>(
  grid: &Grid,
  selection: Option<(usize, usize, usize, usize)>,
  font_size: f32,
  scroll_offset: usize,
  hovered_url: Option<&str>,
  hovered_link_span: Option<(usize, usize, usize)>,
  search_matches: &[(bool, usize, usize, usize)],
  search_current: Option<usize>,
) -> Element<'a, Message> {
  let mut grid_ui = column![].spacing(0);
  let char_width = font_size * 0.62;

  let cursor_x = grid.cursor_x;
  let cursor_y = grid.cursor_y;
  let cursor_visible = grid.cursor_visible;
  let scrollback = &grid.scrollback;
  let sb_len = scrollback.len();
  let clamped_offset = scroll_offset.min(sb_len);

  let mut viewport_y = 0usize;
  let mut display_y = 0usize;

  let line_height = font_size * 1.29;
  let cursor_color = theme::color::runtime().cursor;
  let render_row = |spans: Vec<Span<'static>>| {
    let mut r = row![].spacing(0);
    for span in spans {
      r = r.push(
        container(
          rich_text([span])
            .size(font_size)
            .font(theme::font::REGULAR)
            .wrapping(text::Wrapping::None),
        )
        .width(char_width)
        .height(line_height),
      );
    }
    r
  };

  let search_hl_for = |is_sb: bool, row: usize, cols: usize| -> Option<(Vec<bool>, Vec<bool>)> {
    let relevant: Vec<_> = search_matches
      .iter()
      .enumerate()
      .filter(|(_, m)| m.0 == is_sb && m.1 == row)
      .collect();
    if relevant.is_empty() {
      return None;
    }
    let mut is_match = vec![false; cols];
    let mut is_current = vec![false; cols];
    for (idx, m) in &relevant {
      for col in m.2..m.3.min(cols) {
        is_match[col] = true;
        if search_current == Some(*idx) {
          is_current[col] = true;
        }
      }
    }
    Some((is_match, is_current))
  };

  let sb_start = sb_len.saturating_sub(clamped_offset);
  for (sb_row_idx, (row_cells, _)) in scrollback.range(sb_start..).enumerate() {
    let abs_sb_idx = sb_start + sb_row_idx;
    let hl = compute_url_highlight(row_cells, display_y, hovered_url, hovered_link_span);
    let sh = search_hl_for(true, abs_sb_idx, row_cells.len());
    let segments = row_spans(
      row_cells,
      None,
      cursor_color,
      viewport_y,
      selection,
      font_size,
      hovered_url,
      &hl,
      None,
      sh.as_ref().map(|(m, c)| (m.as_slice(), c.as_slice())),
    );
    grid_ui = grid_ui.push(render_row(segments));
    display_y += 1;
    viewport_y += 1;
  }

  let live_count = grid.rows.saturating_sub(clamped_offset);
  for (y, row_cells) in grid.cells[..live_count * grid.cols]
    .chunks_exact(grid.cols)
    .enumerate()
  {
    let cursor_col = if cursor_visible && clamped_offset == 0 && y == cursor_y {
      Some(cursor_x)
    } else {
      None
    };
    let row_suggestion = if clamped_offset == 0 && y == cursor_y {
      grid.suggestion.as_deref()
    } else {
      None
    };
    let hl = compute_url_highlight(row_cells, display_y, hovered_url, hovered_link_span);
    let sh = search_hl_for(false, y, row_cells.len());
    let segments = row_spans(
      row_cells,
      cursor_col,
      cursor_color,
      viewport_y,
      selection,
      font_size,
      hovered_url,
      &hl,
      row_suggestion,
      sh.as_ref().map(|(m, c)| (m.as_slice(), c.as_slice())),
    );
    grid_ui = grid_ui.push(render_row(segments));
    display_y += 1;
    viewport_y += 1;
  }

  let term_bg = theme::color::runtime().background;

  container(grid_ui)
    .style(move |_| container::Style {
      background: Some(term_bg.into()),
      ..Default::default()
    })
    .padding(Padding {
      top: 12.0,
      right: 20.0,
      bottom: 8.0,
      left: 20.0,
    })
    .height(Length::Fill)
    .width(Length::Fill)
    .clip(true)
    .into()
}

fn cell_span(
  text: String,
  mut fg: Option<Color>,
  bg: Option<Color>,
  attrs: crate::core::grid::CellAttrs,
  font_size: f32,
) -> Span<'static> {
  let rt = theme::color::runtime();
  let term_fg = rt.foreground;
  let term_bg = rt.background;
  let accent = rt.accent;
  drop(rt);

  let reverse = attrs.contains(crate::core::grid::CellAttrs::REVERSE);
  let bold = attrs.contains(crate::core::grid::CellAttrs::BOLD);
  let italic = attrs.contains(crate::core::grid::CellAttrs::ITALIC);
  let underline = attrs.contains(crate::core::grid::CellAttrs::UNDERLINE);
  let strikethrough = attrs.contains(crate::core::grid::CellAttrs::STRIKETHROUGH);
  let dim = attrs.contains(crate::core::grid::CellAttrs::DIM);

  let original_fg = fg;
  let original_bg = bg;

  if dim {
    if let Some(c) = fg.as_mut() {
      c.a = 0.5;
    } else {
      let mut c = term_fg;
      c.a = 0.5;
      fg = Some(c);
    }
  }

  let font = match (bold, italic) {
    (true, true) => iced::Font {
      family: iced::font::Family::Name("FiraCode Nerd Font"),
      weight: iced::font::Weight::Bold,
      style: iced::font::Style::Italic,
      stretch: iced::font::Stretch::Normal,
    },
    (true, false) => theme::font::BOLD,
    (false, true) => iced::Font {
      family: iced::font::Family::Name("FiraCode Nerd Font"),
      weight: iced::font::Weight::Normal,
      style: iced::font::Style::Italic,
      stretch: iced::font::Stretch::Normal,
    },
    (false, false) => theme::font::REGULAR,
  };

  if reverse {
    let rev_fg = bg.unwrap_or(term_bg);
    let rev_bg = if original_fg.is_none() && original_bg.is_none() {
      accent
    } else {
      fg.unwrap_or(term_fg)
    };
    Span::new(text)
      .color(rev_fg)
      .background(Background::Color(rev_bg))
      .underline(underline)
      .strikethrough(strikethrough)
      .font(font)
      .size(font_size)
  } else {
    let color = fg.unwrap_or(term_fg);
    let mut span = Span::new(text)
      .color(color)
      .underline(underline)
      .strikethrough(strikethrough)
      .font(font)
      .size(font_size);
    if let Some(bg_color) = bg {
      let is_selection_like = bg_color.r > 0.85
        && bg_color.g > 0.85
        && bg_color.b > 0.85
        && color.r < 0.3
        && color.g < 0.3
        && color.b < 0.3;
      if is_selection_like {
        span = span.background(Background::Color(accent));
      } else {
        span = span.background(Background::Color(bg_color));
      }
    }
    span
  }
}
