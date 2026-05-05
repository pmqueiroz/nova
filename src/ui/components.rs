use chrono::Local;
use iced::{
  Element, Length, Padding, alignment, color,
  widget::{column, container, row, scrollable, space::horizontal},
};

use crate::ui::{
  app_state::{Message, Tab},
  typography::Typography,
};

pub fn app<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
  container(content)
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

pub fn cursor<'a>() -> Element<'a, Message> {
  Typography {
    color: color!(62, 207, 142),
    ..Default::default()
  }
  .as_text("_")
  .into()
}

pub fn term<'a>(active_tab: &Tab) -> Element<'a, Message> {
  let mut grid_ui = column![].spacing(0);

  for (y, row_cells) in active_tab.grid.cells.iter().enumerate() {
    let mut ui_row = row![].spacing(0);
    let mut current_text = String::new();
    let mut current_color = color!(0xE5E5E5);

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
      background: Some(color!(0x0D0D0D).into()),
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

pub fn status_bar<'a>() -> Element<'a, Message> {
  let local_now = Local::now();

  container(
    row![
      status_bar_text("bash"),
      status_bar_text("utf-8"),
      horizontal(),
      status_bar_text(&local_now.format("%b %d").to_string()),
      status_bar_text(&local_now.format("%H:%M:%S").to_string()),
    ]
    .spacing(16)
    .align_y(alignment::Vertical::Center),
  )
  .style(move |_| container::Style {
    background: Some(color!(0x080808).into()),
    ..container::Style::default()
  })
  .padding(Padding::from([0, 16]))
  .width(Length::Fill)
  .height(22)
  .into()
}

pub fn status_bar_text(content: impl Into<String>) -> Element<'static, Message> {
  Typography {
    color: color!(0x444444),
    size: 14.into(),
  }
  .as_text(content)
  .align_y(alignment::Vertical::Center)
  .into()
}
