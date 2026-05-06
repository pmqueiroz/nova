use chrono::Local;
use iced::{
  Border, Element, Length, Padding, alignment,
  border::Radius,
  widget::{button, column, container, row, scrollable, space::horizontal},
};

use crate::ui::{
  app_state::{Message, Tab},
  theme,
  typography::Typography,
};

pub fn app<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
  container(content)
    .style(move |_| container::Style {
      background: Some(theme::TRANSPARENT.as_color().into()),
      border: Border {
        color: theme::TRANSPARENT.as_color(),
        radius: Radius::new(12.0),
        width: 0.5,
      },
      ..container::Style::default()
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

pub fn tab_bar(tabs: &Vec<Tab>, active_index: usize) -> Element<'static, Message> {
  let mut tab_bar = row![];

  for (i, tab) in tabs.iter().enumerate() {
    let is_active = i == active_index;

    tab_bar = tab_bar.push(row![tab_item(format!("Terminal {}", tab.id), i, is_active)].spacing(2));
  }

  tab_bar = tab_bar.push(
    container(
      button(Typography::span("+"))
        .style(move |_t, _s| button::Style {
          background: Some(theme::TRANSPARENT.as_color().into()),
          ..Default::default()
        })
        .on_press(Message::NewTab)
        .padding(4),
    )
    .padding(Padding::from([0, 8])),
  );

  container(tab_bar)
    .padding(Padding::from([0, 8]))
    .width(Length::Fill)
    .style(move |_| container::Style {
      background: Some(theme::BG_DEEP.as_color().into()),
      border: Border {
        color: theme::BORDER.as_color(),
        radius: Radius {
          ..Default::default()
        },
        width: 0.5,
      },
      ..Default::default()
    })
    .height(36)
    .align_y(alignment::Vertical::Bottom)
    .into()
}

fn tab_item(title: String, index: usize, active: bool) -> Element<'static, Message> {
  button(
    container(
      row![
        button(
          Typography {
            ..Default::default()
          }
          .as_text(title),
        )
        .style(move |_t, _s| button::Style {
          background: Some(theme::TRANSPARENT.as_color().into()),
          ..Default::default()
        })
        .padding(0)
        .on_press(Message::SwitchTab(index)),
        button(
          Typography {
            ..Default::default()
          }
          .as_text("x")
        )
        .style(move |_t, _s| button::Style {
          background: Some(theme::TRANSPARENT.as_color().into()),
          ..Default::default()
        })
        .on_press(Message::CloseTab(index))
        .padding(0),
      ]
      .spacing(8),
    )
    .style(move |_t| container::Style {
      background: Some(
        theme::BG
          .with_alpha(if active { 1.0 } else { 0.0 })
          .as_color()
          .into(),
      ),
      border: Border {
        color: theme::BORDER
          .with_alpha(if active { theme::BORDER.a } else { 0.0 })
          .as_color(),
        radius: Radius {
          top_left: 8.0,
          top_right: 8.0,
          ..Default::default()
        },
        width: 1.0,
      },
      ..Default::default()
    })
    .center_y(30)
    .padding(Padding::from([0, 12])),
  )
  .padding(0)
  .style(move |_t, _s| button::Style {
    background: Some(theme::TRANSPARENT.as_color().into()),
    ..Default::default()
  })
  .on_press(Message::SwitchTab(index))
  .into()
}

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
    .spacing(16),
  )
  .style(move |_| container::Style {
    background: Some(theme::BG_DEEP.as_color().into()),
    border: Border {
      color: theme::BORDER.as_color(),
      radius: Radius {
        bottom_left: 12.0,
        bottom_right: 12.0,
        ..Default::default()
      },
      width: 0.5,
    },
    ..container::Style::default()
  })
  .center_y(22)
  .padding(Padding::from([0, 16]))
  .width(Length::Fill)
  .into()
}

pub fn status_bar_text(content: impl Into<String>) -> Element<'static, Message> {
  Typography {
    color: theme::FG_MUTED.as_color(),
    size: 14.into(),
  }
  .as_text(content)
  .align_y(alignment::Vertical::Center)
  .into()
}
