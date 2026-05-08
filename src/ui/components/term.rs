use iced::{
  Background, Border, Color, Element, Length, Padding,
  border::Radius,
  widget::{column, container, rich_text, text::Span},
};

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

pub fn term<'a>(
  active_tab: &Tab,
  selection: Option<(usize, usize, usize, usize)>,
  font_size: f32,
) -> Element<'a, Message> {
  let mut grid_ui = column![].spacing(0);

  let cursor_x = active_tab.grid.cursor_x;
  let cursor_y = active_tab.grid.cursor_y;

  for (y, row_cells) in active_tab.grid.cells.iter().enumerate() {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut seg_text = String::new();
    let mut seg_fg: Option<Color> = None;
    let mut seg_bg: Option<Color> = None;
    let mut seg_reverse = false;

    for (x, cell) in row_cells.iter().enumerate() {
      let is_cursor = x == cursor_x && y == cursor_y;
      let is_selected = in_selection(x, y, selection);

      let (eff_fg, eff_bg, eff_reverse) = if is_selected {
        let rt = theme::color::runtime();
        (Some(rt.background), Some(rt.accent), false)
      } else {
        (cell.fg, cell.bg, cell.reverse)
      };

      if is_cursor {
        if !seg_text.is_empty() {
          spans.push(cell_span(
            std::mem::take(&mut seg_text),
            seg_fg,
            seg_bg,
            seg_reverse,
            font_size,
          ));
        }
        let ch = if cell.c == ' ' { '_' } else { cell.c };
        let cursor_color = theme::color::runtime().cursor;
        spans.push(
          Span::new(ch.to_string())
            .color(cursor_color)
            .underline(true)
            .font(theme::font::REGULAR)
            .size(font_size),
        );
        seg_fg = eff_fg;
        seg_bg = eff_bg;
        seg_reverse = eff_reverse;
      } else {
        if eff_fg != seg_fg || eff_bg != seg_bg || eff_reverse != seg_reverse {
          if !seg_text.is_empty() {
            spans.push(cell_span(
              std::mem::take(&mut seg_text),
              seg_fg,
              seg_bg,
              seg_reverse,
              font_size,
            ));
          }
          seg_fg = eff_fg;
          seg_bg = eff_bg;
          seg_reverse = eff_reverse;
        }
        seg_text.push(cell.c);
      }
    }

    if !seg_text.is_empty() {
      spans.push(cell_span(seg_text, seg_fg, seg_bg, seg_reverse, font_size));
    }

    grid_ui = grid_ui.push(rich_text(spans).size(font_size).font(theme::font::REGULAR));
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
  fg: Option<Color>,
  bg: Option<Color>,
  reverse: bool,
  font_size: f32,
) -> Span<'static> {
  let rt = theme::color::runtime();
  let term_fg = rt.foreground;
  let term_bg = rt.background;
  drop(rt);
  if reverse {
    let rev_fg = bg.unwrap_or(term_bg);
    let rev_bg = fg.unwrap_or(term_fg);
    Span::new(text)
      .color(rev_fg)
      .background(Background::Color(rev_bg))
      .font(theme::font::REGULAR)
      .size(font_size)
  } else {
    let color = fg.unwrap_or(term_fg);
    let mut span = Span::new(text)
      .color(color)
      .font(theme::font::REGULAR)
      .size(font_size);
    if let Some(bg_color) = bg {
      span = span.background(Background::Color(bg_color));
    }
    span
  }
}
