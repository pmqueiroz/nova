use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::text::{self, Renderer as _};
use iced::advanced::widget::{Tree, Widget};
use iced::advanced::{Renderer as _, mouse};
use iced::alignment;
use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::{Color, Element, Font, Length, Padding, Pixels, Point, Rectangle, Size, Vector};
use smol_str::SmolStr;

use crate::core::config::CursorStyle;
use crate::core::glyph::block_glyph;
use crate::core::grid::{Cell, CellAttrs, Grid};
use crate::core::url::detect_urls;
use crate::ui::{app_state::Message, theme};

fn single_char(text: &str) -> Option<char> {
  let mut chars = text.chars();
  let c = chars.next()?;
  chars.next().is_none().then_some(c)
}

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
  row_cells: &[Cell],
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
      while i < row_cells.len() && !is_url_terminator(row_cells[i].c.chars().next().unwrap_or(' '))
      {
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

#[derive(Clone, Copy)]
struct Palette {
  fg: Color,
  bg: Color,
  accent: Color,
  cursor: Color,
}

impl Palette {
  fn current() -> Self {
    let rt = theme::color::runtime();
    Self {
      fg: rt.foreground,
      bg: rt.background,
      accent: rt.accent,
      cursor: rt.cursor,
    }
  }
}

#[derive(Clone, Copy)]
struct Style {
  color: Color,
  bg: Option<Color>,
  font: Font,
  underline: bool,
  strike: bool,
}

struct DrawCell {
  row: usize,
  col: usize,
  width: u8,
  text: SmolStr,
  style: Style,
  beam: Option<Color>,
}

impl DrawCell {
  fn is_invisible(&self) -> bool {
    self.text.trim().is_empty()
      && self.style.bg.is_none()
      && !self.style.underline
      && !self.style.strike
      && self.beam.is_none()
  }
}

fn cell_style(
  mut fg: Option<Color>,
  bg: Option<Color>,
  attrs: CellAttrs,
  palette: &Palette,
) -> Style {
  let original_fg = fg;
  let original_bg = bg;

  if attrs.contains(CellAttrs::DIM) {
    if let Some(c) = fg.as_mut() {
      c.a = 0.5;
    } else {
      fg = Some(Color {
        a: 0.5,
        ..palette.fg
      });
    }
  }

  let font = match (
    attrs.contains(CellAttrs::BOLD),
    attrs.contains(CellAttrs::ITALIC),
  ) {
    (true, true) => theme::font::bold_italic(),
    (true, false) => theme::font::bold(),
    (false, true) => theme::font::italic(),
    (false, false) => theme::font::regular(),
  };
  let underline = attrs.contains(CellAttrs::UNDERLINE);
  let strike = attrs.contains(CellAttrs::STRIKETHROUGH);

  if attrs.contains(CellAttrs::REVERSE) {
    let rev_bg = if original_fg.is_none() && original_bg.is_none() {
      palette.accent
    } else {
      fg.unwrap_or(palette.fg)
    };
    Style {
      color: bg.unwrap_or(palette.bg),
      bg: Some(rev_bg),
      font,
      underline,
      strike,
    }
  } else {
    let color = fg.unwrap_or(palette.fg);
    let bg = bg.map(|bg_color| {
      let is_selection_like = bg_color.r > 0.85
        && bg_color.g > 0.85
        && bg_color.b > 0.85
        && color.r < 0.3
        && color.g < 0.3
        && color.b < 0.3;
      if is_selection_like {
        palette.accent
      } else {
        bg_color
      }
    });
    Style {
      color,
      bg,
      font,
      underline,
      strike,
    }
  }
}

struct RowInput<'r> {
  cells: &'r [Cell],
  y: usize,
  cursor_col: Option<usize>,
  suggestion: Option<&'r str>,
  url_highlight: &'r [bool],
  search_hl: Option<(&'r [bool], &'r [bool])>,
}

struct Frame<'f> {
  palette: Palette,
  cursor_style: CursorStyle,
  cursor_visible: bool,
  selection: Option<(usize, usize, usize, usize)>,
  hovered_url: Option<&'f str>,
}

fn push_row(out: &mut Vec<DrawCell>, row: RowInput<'_>, frame: &Frame<'_>) {
  let mut col = 0usize;
  let mut push = |out: &mut Vec<DrawCell>, width: u8, text: SmolStr, style: Style, beam| {
    let cell = DrawCell {
      row: row.y,
      col,
      width,
      text,
      style,
      beam,
    };
    col += width as usize;
    if !cell.is_invisible() {
      out.push(cell);
    }
  };

  for (x, cell) in row.cells.iter().enumerate() {
    if cell.width == 0 {
      continue;
    }

    let is_cursor = row.cursor_col == Some(x);
    let is_selected = in_selection(x, row.y, frame.selection);
    let is_url_hovered = frame.hovered_url.is_some_and(|hover| {
      cell.uri.as_deref() == Some(hover) || row.url_highlight.get(x).copied().unwrap_or(false)
    });
    let is_search_current = row
      .search_hl
      .is_some_and(|(_, cur)| cur.get(x).copied().unwrap_or(false));
    let is_search_match = !is_search_current
      && row
        .search_hl
        .is_some_and(|(m, _)| m.get(x).copied().unwrap_or(false));

    let (mut eff_fg, mut eff_bg, mut eff_attrs) = if is_selected {
      (
        Some(frame.palette.bg),
        Some(frame.palette.accent),
        CellAttrs::empty(),
      )
    } else if is_search_current {
      (
        Some(SEARCH_CURRENT_FG),
        Some(SEARCH_CURRENT_BG),
        CellAttrs::empty(),
      )
    } else if is_search_match {
      (cell.fg, Some(SEARCH_MATCH_BG), cell.attrs)
    } else {
      (cell.fg, cell.bg, cell.attrs)
    };

    if is_url_hovered {
      eff_attrs.insert(CellAttrs::UNDERLINE);
    }

    if is_cursor && frame.cursor_visible {
      if let Some(sugg) = row.suggestion {
        let max_chars = row.cells.len().saturating_sub(x);
        let mut chars = sugg.chars().take(max_chars);
        if let Some(first) = chars.next() {
          let regular = theme::font::regular();
          let first_style = Style {
            color: frame.palette.cursor,
            bg: None,
            font: regular,
            underline: true,
            strike: false,
          };
          push(
            out,
            1,
            SmolStr::from(first.encode_utf8(&mut [0; 4]) as &str),
            first_style,
            None,
          );
          let dim = Style {
            color: Color {
              a: 0.35,
              ..frame.palette.fg
            },
            underline: false,
            ..first_style
          };
          for c in chars {
            push(
              out,
              1,
              SmolStr::from(c.encode_utf8(&mut [0; 4]) as &str),
              dim,
              None,
            );
          }
        }
        break;
      }
      match frame.cursor_style {
        CursorStyle::Block => eff_bg = Some(frame.palette.cursor),
        CursorStyle::Beam => {
          let style = cell_style(cell.fg, cell.bg, cell.attrs, &frame.palette);
          push(
            out,
            cell.width,
            cell.c.clone(),
            style,
            Some(frame.palette.cursor),
          );
          continue;
        }
        CursorStyle::Underline => {
          eff_fg = Some(frame.palette.cursor);
          eff_attrs.insert(CellAttrs::UNDERLINE);
        }
      }
    }
    let style = cell_style(eff_fg, eff_bg, eff_attrs, &frame.palette);
    push(out, cell.width, cell.c.clone(), style, None);
  }
}

pub const CLASSIC_PADDING: Padding = Padding {
  top: 12.0,
  right: 20.0,
  bottom: 8.0,
  left: 20.0,
};

struct TermView {
  cells: Vec<DrawCell>,
  font_size: f32,
  char_width: f32,
  line_height: f32,
  padding: Padding,
}

impl TermView {
  fn draw_cell(
    &self,
    renderer: &mut iced::Renderer,
    origin: Point,
    cell: &DrawCell,
    clip: Rectangle,
  ) {
    let width = self.char_width * cell.width as f32;
    let height = self.line_height;
    let pos = Point::new(
      origin.x + cell.col as f32 * self.char_width,
      origin.y + cell.row as f32 * height,
    );
    let style = &cell.style;
    if let Some(bg) = style.bg {
      renderer.fill_quad(
        Quad {
          bounds: Rectangle::new(pos, Size::new(width, height)),
          ..Quad::default()
        },
        bg,
      );
    }

    match single_char(&cell.text).and_then(block_glyph) {
      Some(glyph) => {
        let color = Color {
          a: style.color.a * glyph.alpha,
          ..style.color
        };
        for rect in &glyph.rects {
          let x0 = width * f32::from(rect.x0) / 8.0;
          let x1 = width * f32::from(rect.x1) / 8.0;
          let y0 = height * f32::from(rect.y0) / 8.0;
          let y1 = height * f32::from(rect.y1) / 8.0;
          renderer.fill_quad(
            Quad {
              bounds: Rectangle::new(pos + Vector::new(x0, y0), Size::new(x1 - x0, y1 - y0)),
              ..Quad::default()
            },
            color,
          );
        }
      }
      None if !cell.text.trim().is_empty() => {
        renderer.fill_text(
          text::Text {
            content: cell.text.to_string(),
            bounds: Size::new(f32::INFINITY, height),
            size: Pixels(self.font_size),
            line_height: LineHeight::default(),
            font: style.font,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: Shaping::Advanced,
            wrapping: Wrapping::None,
          },
          pos,
          style.color,
          clip,
        );
      }
      None => {}
    }

    if style.underline || style.strike {
      let size = self.font_size;
      let text_line = LineHeight::default().to_absolute(Pixels(size)).0;
      let baseline = pos.y + size + (text_line - size) / 2.0;
      let mut rule = |dy: f32| {
        renderer.fill_quad(
          Quad {
            bounds: Rectangle::new(Point::new(pos.x, baseline - dy), Size::new(width, 1.0)),
            ..Quad::default()
          },
          style.color,
        );
      };
      if style.underline {
        rule(size * 0.08);
      }
      if style.strike {
        rule(size / 2.0);
      }
    }

    if let Some(beam) = cell.beam {
      renderer.fill_quad(
        Quad {
          bounds: Rectangle::new(pos, Size::new(2.0, height)),
          ..Quad::default()
        },
        beam,
      );
    }
  }
}

impl Widget<Message, iced::Theme, iced::Renderer> for TermView {
  fn size(&self) -> Size<Length> {
    Size::new(Length::Fill, Length::Fill)
  }

  fn layout(
    &mut self,
    _tree: &mut Tree,
    _renderer: &iced::Renderer,
    limits: &layout::Limits,
  ) -> layout::Node {
    layout::Node::new(limits.resolve(Length::Fill, Length::Fill, Size::ZERO))
  }

  fn draw(
    &self,
    _tree: &Tree,
    renderer: &mut iced::Renderer,
    _theme: &iced::Theme,
    _style: &renderer::Style,
    layout: Layout<'_>,
    _cursor: mouse::Cursor,
    viewport: &Rectangle,
  ) {
    let bounds = layout.bounds();
    let Some(clip) = bounds.intersection(viewport) else {
      return;
    };
    let origin = Point::new(bounds.x + self.padding.left, bounds.y + self.padding.top);
    renderer.with_layer(clip, |renderer| {
      for cell in &self.cells {
        self.draw_cell(renderer, origin, cell, clip);
      }
    });
  }
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
  cursor_style: CursorStyle,
  cursor_blink_visible: bool,
  padding: Padding,
) -> Element<'a, Message> {
  let palette = Palette::current();
  let frame = Frame {
    palette,
    cursor_style,
    cursor_visible: cursor_blink_visible,
    selection,
    hovered_url,
  };

  let cursor_x = grid.cursor_x;
  let cursor_y = grid.cursor_y;
  let cursor_visible = grid.cursor_visible;
  let scrollback = &grid.scrollback;
  let sb_len = scrollback.len();
  let clamped_offset = scroll_offset.min(sb_len);

  let search_hl_for = |is_sb: bool, row: usize, cols: usize| -> Option<(Vec<bool>, Vec<bool>)> {
    let mut found = None::<(Vec<bool>, Vec<bool>)>;
    for (idx, m) in search_matches.iter().enumerate() {
      if m.0 != is_sb || m.1 != row {
        continue;
      }
      let (is_match, is_current) =
        found.get_or_insert_with(|| (vec![false; cols], vec![false; cols]));
      for col in m.2..m.3.min(cols) {
        is_match[col] = true;
        if search_current == Some(idx) {
          is_current[col] = true;
        }
      }
    }
    found
  };

  let mut cells = Vec::new();
  let mut viewport_y = 0usize;

  let sb_start = sb_len.saturating_sub(clamped_offset);
  for (sb_row_idx, (row_cells, _)) in scrollback.range(sb_start..).enumerate() {
    let hl = compute_url_highlight(row_cells, viewport_y, hovered_url, hovered_link_span);
    let sh = search_hl_for(true, sb_start + sb_row_idx, row_cells.len());
    push_row(
      &mut cells,
      RowInput {
        cells: row_cells,
        y: viewport_y,
        cursor_col: None,
        suggestion: None,
        url_highlight: &hl,
        search_hl: sh.as_ref().map(|(m, c)| (m.as_slice(), c.as_slice())),
      },
      &frame,
    );
    viewport_y += 1;
  }

  let live_count = grid.rows.saturating_sub(clamped_offset);
  for (y, row_cells) in grid.cells[..live_count * grid.cols]
    .chunks_exact(grid.cols)
    .enumerate()
  {
    let at_cursor_row = clamped_offset == 0 && y == cursor_y;
    let hl = compute_url_highlight(row_cells, viewport_y, hovered_url, hovered_link_span);
    let sh = search_hl_for(false, y, row_cells.len());
    push_row(
      &mut cells,
      RowInput {
        cells: row_cells,
        y: viewport_y,
        cursor_col: (cursor_visible && at_cursor_row).then_some(cursor_x),
        suggestion: if at_cursor_row {
          grid.suggestion.as_deref()
        } else {
          None
        },
        url_highlight: &hl,
        search_hl: sh.as_ref().map(|(m, c)| (m.as_slice(), c.as_slice())),
      },
      &frame,
    );
    viewport_y += 1;
  }

  Element::new(TermView {
    cells,
    font_size,
    char_width: font_size * 0.62,
    line_height: font_size * 1.29,
    padding,
  })
}
