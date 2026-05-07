use iced::{
  Border, Color, Element, Length, Padding,
  border::Radius,
  widget::{column, container, rich_text, row, scrollable, text, text::Span},
};

use crate::ui::{app_state::Message, tab::Tab, theme};

pub fn cursor<'a>() -> Element<'a, Message> {
  text("_")
    .color(theme::color::ACCENT.as_color())
    .font(theme::font::REGULAR)
    .size(16.0)
    .into()
}

pub fn term<'a>(active_tab: &Tab) -> Element<'a, Message> {
  let mut grid_ui = column![].spacing(0);

  let cursor_x = active_tab.grid.cursor_x;
  let cursor_y = active_tab.grid.cursor_y;

  for (y, row_cells) in active_tab.grid.cells.iter().enumerate() {
    if y == cursor_y {
      let mut before: Vec<Span<'static>> = Vec::new();
      let mut after: Vec<Span<'static>> = Vec::new();
      let mut seg_text = String::new();
      let mut seg_color = Color::WHITE;

      for (x, cell) in row_cells.iter().enumerate() {
        if x < cursor_x {
          if cell.fg != seg_color {
            if !seg_text.is_empty() {
              before.push(cell_span(std::mem::take(&mut seg_text), seg_color));
            }
            seg_color = cell.fg;
          }
          seg_text.push(cell.c);
        } else if x == cursor_x {
          if !seg_text.is_empty() {
            before.push(cell_span(std::mem::take(&mut seg_text), seg_color));
          }
          seg_color = cell.fg;
        } else {
          if cell.fg != seg_color {
            if !seg_text.is_empty() {
              after.push(cell_span(std::mem::take(&mut seg_text), seg_color));
            }
            seg_color = cell.fg;
          }
          seg_text.push(cell.c);
        }
      }
      if !seg_text.is_empty() {
        after.push(cell_span(seg_text, seg_color));
      }

      grid_ui = grid_ui.push(
        row![
          rich_text(before).size(16).font(theme::font::REGULAR),
          cursor(),
          rich_text(after).size(16).font(theme::font::REGULAR),
        ]
        .spacing(0),
      );
    } else {
      let mut spans: Vec<Span<'static>> = Vec::new();
      let mut seg_text = String::new();
      let mut seg_color = Color::WHITE;

      for cell in row_cells.iter() {
        if cell.fg != seg_color {
          if !seg_text.is_empty() {
            spans.push(cell_span(std::mem::take(&mut seg_text), seg_color));
          }
          seg_color = cell.fg;
        }
        seg_text.push(cell.c);
      }
      if !seg_text.is_empty() {
        spans.push(cell_span(seg_text, seg_color));
      }

      grid_ui = grid_ui.push(rich_text(spans).size(16).font(theme::font::REGULAR));
    }
  }

  container(scrollable(grid_ui).height(Length::Fill).width(Length::Fill))
    .style(move |_| container::Style {
      background: Some(theme::color::BG.as_color().into()),
      border: Border {
        color: theme::color::BORDER.as_color(),
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

fn cell_span(text: String, fg: Color) -> Span<'static> {
  let color = if fg == Color::WHITE {
    theme::color::FG.as_color()
  } else {
    fg
  };
  Span::new(text)
    .color(color)
    .font(theme::font::REGULAR)
    .size(16.0)
}
