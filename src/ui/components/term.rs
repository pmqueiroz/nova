use iced::{
  Border, Element, Length, Padding,
  border::Radius,
  widget::{column, container, row, scrollable},
};

use crate::ui::{
  app_state::{Message, Tab},
  theme,
  typography::Typography,
};

pub fn term<'a>(active_tab: &Tab) -> Element<'a, Message> {
  let mut grid_ui = column![].spacing(0);

  for (y, row_cells) in active_tab.grid.cells.iter().enumerate() {
    let mut ui_row = row![].spacing(0);
    let mut current_text = String::new();
    let mut current_color = theme::FG.as_color();

    for (x, cell) in row_cells.iter().enumerate() {
      let is_cursor = x == active_tab.grid.cursor_x && y == active_tab.grid.cursor_y;

      if is_cursor || cell.fg != current_color {
        if !current_text.is_empty() {
          ui_row = ui_row.push(Typography::default().as_text(current_text.clone()));
          current_text.clear();
        }
        current_color = cell.fg;
      }

      if is_cursor {
        ui_row = ui_row.push(cursor());
      } else {
        current_text.push(cell.c);
      }
    }

    if !current_text.is_empty() {
      ui_row = ui_row.push(
        Typography {
          color: current_color,
          ..Default::default()
        }
        .as_text(&current_text),
      );
    }

    grid_ui = grid_ui.push(ui_row);
  }

  container(scrollable(grid_ui).height(Length::Fill).width(Length::Fill))
    .style(move |_| container::Style {
      background: Some(theme::BG.as_color().into()),
      border: Border {
        color: theme::BORDER.as_color(),
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

pub fn cursor<'a>() -> Element<'a, Message> {
  Typography {
    color: theme::ACCENT.as_color(),
    ..Default::default()
  }
  .as_text("_")
  .into()
}
