use chrono::Local;
use iced::{
  Border, Color, Element, Length, Padding, Shadow, alignment,
  border::{Radius, radius},
  widget::{Space, button, column, container, mouse_area, row, scrollable, space::horizontal},
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

pub fn title_bar(window_focused: bool) -> Element<'static, Message> {
  #[cfg(target_os = "windows")]
  let controls = windows_controls(window_focused);
  #[cfg(not(target_os = "windows"))]
  let controls = traffic_lights(window_focused);

  #[cfg(target_os = "windows")]
  let title_row = row![
    horizontal(),
    Typography {
      color: theme::FG_MUTED.as_color(),
      size: 12.into(),
    }
    .as_text("nova"),
    horizontal(),
    controls,
  ]
  .spacing(8);

  #[cfg(not(target_os = "windows"))]
  let title_row = row![
    controls,
    horizontal(),
    Typography {
      color: theme::FG_MUTED.as_color(),
      size: 12.into(),
    }
    .as_text("nova"),
    horizontal(),
  ]
  .spacing(8);

  mouse_area(
    container(title_row)
      .style(move |_| container::Style {
        background: Some(theme::BG_DEEP.as_color().into()),
        border: Border {
          color: theme::BORDER.as_color(),
          radius: Radius {
            top_left: 12.0,
            top_right: 12.0,
            ..Default::default()
          },
          width: 0.5,
        },
        ..container::Style::default()
      })
      .padding(Padding::from([0, 16]))
      .width(Length::Fill)
      .center_y(40),
  )
  .on_press(Message::DragWindow)
  .into()
}

// TODO: windows icons
#[cfg(target_os = "windows")]
pub fn windows_controls(window_focused: bool) -> Element<'static, Message> {
  let circle_btn = |color: Color, msg: Message| {
    button(
      Space::new()
        .width(Length::Fixed(12.0))
        .height(Length::Fixed(12.0)),
    )
    .padding(0)
    .on_press(msg)
    .style(move |_t, _s| button::Style {
      background: Some(
        if window_focused {
          color
        } else {
          theme::TRAFFIC_LIGHT_INACTIVE.as_color()
        }
        .into(),
      ),
      border: iced::Border {
        radius: radius(120.0),
        ..Default::default()
      },
      ..Default::default()
    })
  };

  let red_light = circle_btn(theme::TRAFFIC_LIGHT_RED.as_color(), Message::CloseWindow);
  let yellow_light = circle_btn(
    theme::TRAFFIC_LIGHT_YELLOW.as_color(),
    Message::MinimizeWindow,
  );
  let green_light = circle_btn(
    theme::TRAFFIC_LIGHT_GREEN.as_color(),
    Message::MaximizeWindow,
  );

  row![red_light, yellow_light, green_light].spacing(8).into()
}

pub fn traffic_lights(window_focused: bool) -> Element<'static, Message> {
  let circle_btn = |color: Color, msg: Message| {
    button(
      Space::new()
        .width(Length::Fixed(12.0))
        .height(Length::Fixed(12.0)),
    )
    .padding(0)
    .on_press(msg)
    .style(move |_t, _s| button::Style {
      background: Some(
        if window_focused {
          color
        } else {
          theme::TRAFFIC_LIGHT_INACTIVE.as_color()
        }
        .into(),
      ),
      border: iced::Border {
        radius: radius(120.0),
        ..Default::default()
      },
      ..Default::default()
    })
  };

  let red_light = circle_btn(theme::TRAFFIC_LIGHT_RED.as_color(), Message::CloseWindow);
  let yellow_light = circle_btn(
    theme::TRAFFIC_LIGHT_YELLOW.as_color(),
    Message::MinimizeWindow,
  );
  let green_light = circle_btn(
    theme::TRAFFIC_LIGHT_GREEN.as_color(),
    Message::MaximizeWindow,
  );

  row![red_light, yellow_light, green_light].spacing(8).into()
}

pub fn tab_bar(tabs: &Vec<Tab>, active_index: usize) -> Element<'static, Message> {
  let mut tab_bar = row![];

  for (i, tab) in tabs.iter().enumerate() {
    let is_active = i == active_index;

    tab_bar = tab_bar.push(row![tab_item(format!("Terminal {}", tab.id), i, is_active)].spacing(2));
  }

  tab_bar = tab_bar.push(
    container(
      button("+")
        .style(move |_t, status| button::Style {
          text_color: match status {
            button::Status::Hovered => theme::ACCENT.as_color(),
            button::Status::Pressed => theme::ACCENT_DIM.as_color(),
            _ => theme::FG_MUTED.as_color(),
          },
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
        button("✕")
          .style(move |_t, status| button::Style {
            text_color: if status == button::Status::Hovered {
              theme::RED.as_color()
            } else {
              theme::FG_MUTED.as_color()
            },
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
      agent_status(),
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

pub fn status_bar_text(content: impl Into<String>) -> iced::widget::Text<'static> {
  Typography {
    color: theme::FG_MUTED.as_color(),
    size: 14.into(),
  }
  .as_text(content)
  .into()
}

pub fn agent_status() -> Element<'static, Message> {
  container(
    row![
      button(
        Space::new()
          .width(Length::Fixed(8.0))
          .height(Length::Fixed(8.0)),
      )
      .padding(0)
      .style(move |_t, _s| button::Style {
        background: Some(theme::ACCENT.as_color().into(),),
        border: iced::Border {
          radius: radius(120.0),
          ..Default::default()
        },
        shadow: Shadow {
          color: theme::ACCENT.with_alpha(0.8).as_color(),
          offset: iced::Vector::new(0.0, 0.0),
          blur_radius: 8.0,
        },
        ..Default::default()
      }),
      Typography {
        color: theme::ACCENT.as_color(),
        size: 14.into(),
      }
      .as_text("connected")
    ]
    .padding(0)
    .spacing(4),
  )
  .center_y(12)
  .into()
}
