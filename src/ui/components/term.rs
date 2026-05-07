use iced::{
  Border, Color, Element, Length, Padding,
  border::Radius,
  widget::{column, container, rich_text, scrollable, text::Span},
};

use crate::core::config;
use crate::ui::{app_state::Message, tab::Tab, theme};

pub fn term<'a>(active_tab: &Tab) -> Element<'a, Message> {
  let mut grid_ui = column![].spacing(0);

  let cursor_x = active_tab.grid.cursor_x;
  let cursor_y = active_tab.grid.cursor_y;
  let font_size = config::get().theme.font.size;

  for (y, row_cells) in active_tab.grid.cells.iter().enumerate() {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut seg_text = String::new();
    let mut seg_color = Color::WHITE;

    for (x, cell) in row_cells.iter().enumerate() {
      let is_cursor = x == cursor_x && y == cursor_y;

      if is_cursor {
        if !seg_text.is_empty() {
          spans.push(cell_span(std::mem::take(&mut seg_text), seg_color, font_size));
        }
        let ch = if cell.c == ' ' { '_' } else { cell.c };
        spans.push(
          Span::new(ch.to_string())
            .color(theme::color::runtime().cursor)
            .underline(true)
            .font(theme::font::REGULAR)
            .size(font_size),
        );
        seg_color = cell.fg;
      } else {
        if cell.fg != seg_color {
          if !seg_text.is_empty() {
            spans.push(cell_span(std::mem::take(&mut seg_text), seg_color, font_size));
          }
          seg_color = cell.fg;
        }
        seg_text.push(cell.c);
      }
    }

    if !seg_text.is_empty() {
      spans.push(cell_span(seg_text, seg_color, font_size));
    }

    grid_ui = grid_ui.push(rich_text(spans).size(font_size).font(theme::font::REGULAR));
  }

  let rt = theme::color::runtime();

  container(scrollable(grid_ui).height(Length::Fill).width(Length::Fill))
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

fn cell_span(text: String, fg: Color, font_size: f32) -> Span<'static> {
  let color = if fg == Color::WHITE {
    theme::color::runtime().foreground
  } else {
    fg
  };
  Span::new(text)
    .color(color)
    .font(theme::font::REGULAR)
    .size(font_size)
}
