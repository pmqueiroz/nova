use iced::{
  Background, Border, Color, Element, Length, Padding,
  border::Radius,
  widget::{column, container, rich_text, text::Span},
};

use crate::core::config;
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

pub fn term<'a>(active_tab: &Tab, selection: Option<(usize, usize, usize, usize)>) -> Element<'a, Message> {
  let mut grid_ui = column![].spacing(0);

  let cursor_x = active_tab.grid.cursor_x;
  let cursor_y = active_tab.grid.cursor_y;
  let font_size = config::get().theme.font.size;

  for (y, row_cells) in active_tab.grid.cells.iter().enumerate() {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut seg_text = String::new();
    let mut seg_fg = Color::WHITE;
    let mut seg_bg = Color::TRANSPARENT;
    let mut seg_reverse = false;

    for (x, cell) in row_cells.iter().enumerate() {
      let is_cursor = x == cursor_x && y == cursor_y;
      let is_selected = in_selection(x, y, selection);

      let (eff_fg, eff_bg, eff_reverse) = if is_selected {
        let rt = theme::color::runtime();
        (rt.background, rt.accent, false)
      } else {
        (cell.fg, cell.bg, cell.reverse)
      };

      if is_cursor {
        if !seg_text.is_empty() {
          spans.push(cell_span(std::mem::take(&mut seg_text), seg_fg, seg_bg, seg_reverse, font_size));
        }
        let ch = if cell.c == ' ' { '_' } else { cell.c };
        spans.push(
          Span::new(ch.to_string())
            .color(theme::color::runtime().cursor)
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
            spans.push(cell_span(std::mem::take(&mut seg_text), seg_fg, seg_bg, seg_reverse, font_size));
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

  let rt = theme::color::runtime();

  container(grid_ui)
    .style(move |_| container::Style {
      background: Some(rt.background.into()),
      border: Border {
        color: rt.border,
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

fn cell_span(text: String, fg: Color, bg: Color, reverse: bool, font_size: f32) -> Span<'static> {
  let rt = theme::color::runtime();
  if reverse {
    let rev_fg = if bg.a > 0.0 { bg } else { rt.background };
    let rev_bg = if fg == Color::WHITE { rt.foreground } else { fg };
    Span::new(text)
      .color(rev_fg)
      .background(Background::Color(rev_bg))
      .font(theme::font::REGULAR)
      .size(font_size)
  } else {
    let color = if fg == Color::WHITE { rt.foreground } else { fg };
    let mut span = Span::new(text)
      .color(color)
      .font(theme::font::REGULAR)
      .size(font_size);
    if bg.a > 0.0 {
      span = span.background(Background::Color(bg));
    }
    span
  }
}
