use iced::{
  Background, Border, Color, Element, Length, Padding,
  border::Radius,
  widget::{column, container, rich_text, text::Span},
};

use crate::core::url::detect_urls;
use crate::ui::{app_state::Message, tab::Tab, theme};

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

#[allow(clippy::too_many_arguments)]
fn row_spans(
  row_cells: &[crate::core::grid::Cell],
  cursor_col: Option<usize>,
  y: usize,
  selection: Option<(usize, usize, usize, usize)>,
  font_size: f32,
  hovered_url: Option<&str>,
  url_highlight: &[bool],
  suggestion: Option<&str>,
) -> Vec<Span<'static>> {
  let mut spans: Vec<Span<'static>> = Vec::new();
  let mut seg_text = String::new();
  let mut seg_fg: Option<Color> = None;
  let mut seg_bg: Option<Color> = None;
  let mut seg_attrs = crate::core::grid::CellAttrs::empty();

  for (x, cell) in row_cells.iter().enumerate() {
    let is_cursor = cursor_col == Some(x);
    let is_selected = in_selection(x, y, selection);
    let is_url_hovered = if let Some(hover) = hovered_url {
      cell.uri.as_deref() == Some(hover) || url_highlight.get(x).copied().unwrap_or(false)
    } else {
      false
    };

    let (eff_fg, eff_bg, mut eff_attrs) = if is_selected {
      let rt = theme::color::runtime();
      (
        Some(rt.background),
        Some(rt.accent),
        crate::core::grid::CellAttrs::empty(),
      )
    } else {
      (cell.fg, cell.bg, cell.attrs)
    };

    if is_url_hovered {
      eff_attrs.insert(crate::core::grid::CellAttrs::UNDERLINE);
    }

    if is_cursor {
      if !seg_text.is_empty() {
        spans.push(cell_span(
          std::mem::take(&mut seg_text),
          seg_fg,
          seg_bg,
          seg_attrs,
          font_size,
        ));
      }
      let cursor_color = theme::color::runtime().cursor;
      if let Some(sugg) = suggestion {
        let max_chars = row_cells.len().saturating_sub(x);
        let display: String = sugg.chars().take(max_chars).collect();
        if let Some(first) = display.chars().next() {
          spans.push(
            Span::new(first.to_string())
              .color(cursor_color)
              .underline(true)
              .font(theme::font::REGULAR)
              .size(font_size),
          );
          let rest: String = display.chars().skip(1).collect();
          if !rest.is_empty() {
            let rt = theme::color::runtime();
            let mut dim_color = rt.foreground;
            dim_color.a = 0.35;
            spans.push(
              Span::new(rest)
                .color(dim_color)
                .font(theme::font::REGULAR)
                .size(font_size),
            );
          }
        }
        break;
      }
      let ch = if cell.c == ' ' { '_' } else { cell.c };
      spans.push(
        Span::new(ch.to_string())
          .color(cursor_color)
          .underline(true)
          .font(theme::font::REGULAR)
          .size(font_size),
      );
      seg_fg = eff_fg;
      seg_bg = eff_bg;
      seg_attrs = eff_attrs;
    } else {
      if eff_fg != seg_fg || eff_bg != seg_bg || eff_attrs != seg_attrs {
        if !seg_text.is_empty() {
          spans.push(cell_span(
            std::mem::take(&mut seg_text),
            seg_fg,
            seg_bg,
            seg_attrs,
            font_size,
          ));
        }
        seg_fg = eff_fg;
        seg_bg = eff_bg;
        seg_attrs = eff_attrs;
      }
      seg_text.push(cell.c);
    }
  }

  if !seg_text.is_empty() {
    spans.push(cell_span(seg_text, seg_fg, seg_bg, seg_attrs, font_size));
  }

  spans
}

pub fn term<'a>(
  active_tab: &Tab,
  selection: Option<(usize, usize, usize, usize)>,
  font_size: f32,
  scroll_offset: usize,
  hovered_url: Option<&str>,
  hovered_link_span: Option<(usize, usize, usize)>,
  suggestion: Option<&str>,
) -> Element<'a, Message> {
  let mut grid_ui = column![].spacing(0);

  let cursor_x = active_tab.grid.cursor_x;
  let cursor_y = active_tab.grid.cursor_y;
  let scrollback = &active_tab.grid.scrollback;
  let sb_len = scrollback.len();
  let clamped_offset = scroll_offset.min(sb_len);

  let mut viewport_y = 0usize;
  let mut display_y = 0usize;

  let sb_start = sb_len.saturating_sub(clamped_offset);
  for (row_cells, _) in scrollback.range(sb_start..) {
    let hl = compute_url_highlight(row_cells, display_y, hovered_url, hovered_link_span);
    let spans = row_spans(
      row_cells,
      None,
      viewport_y,
      selection,
      font_size,
      hovered_url,
      &hl,
      None,
    );
    grid_ui = grid_ui.push(rich_text(spans).size(font_size).font(theme::font::REGULAR));
    display_y += 1;
    viewport_y += 1;
  }

  let live_count = active_tab.grid.rows.saturating_sub(clamped_offset);
  for (y, row_cells) in active_tab.grid.cells[..live_count * active_tab.grid.cols]
    .chunks_exact(active_tab.grid.cols)
    .enumerate()
  {
    let cursor_col = if clamped_offset == 0 && y == cursor_y {
      Some(cursor_x)
    } else {
      None
    };
    let row_suggestion = if clamped_offset == 0 && y == cursor_y {
      suggestion
    } else {
      None
    };
    let hl = compute_url_highlight(row_cells, display_y, hovered_url, hovered_link_span);
    let spans = row_spans(
      row_cells,
      cursor_col,
      viewport_y,
      selection,
      font_size,
      hovered_url,
      &hl,
      row_suggestion,
    );
    grid_ui = grid_ui.push(rich_text(spans).size(font_size).font(theme::font::REGULAR));
    display_y += 1;
    viewport_y += 1;
  }

  let (term_bg, term_border) = {
    let rt = theme::color::runtime();
    (rt.background, rt.border)
  };

  container(grid_ui)
    .style(move |_| container::Style {
      background: Some(term_bg.into()),
      border: Border {
        color: term_border,
        radius: Radius {
          ..Default::default()
        },
        width: 0.5,
      },
      ..container::Style::default()
    })
    .padding(Padding {
      top: 12.0,
      right: 20.0,
      bottom: 8.0,
      left: 20.0,
    })
    .height(Length::Fill)
    .width(Length::Fill)
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
  drop(rt);

  let reverse = attrs.contains(crate::core::grid::CellAttrs::REVERSE);
  let bold = attrs.contains(crate::core::grid::CellAttrs::BOLD);
  let italic = attrs.contains(crate::core::grid::CellAttrs::ITALIC);
  let underline = attrs.contains(crate::core::grid::CellAttrs::UNDERLINE);
  let strikethrough = attrs.contains(crate::core::grid::CellAttrs::STRIKETHROUGH);
  let dim = attrs.contains(crate::core::grid::CellAttrs::DIM);

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
    let rev_bg = fg.unwrap_or(term_fg);
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
      span = span.background(Background::Color(bg_color));
    }
    span
  }
}
