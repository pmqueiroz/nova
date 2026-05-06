use chrono::Local;
use iced::{
  Border, Element, Length, Padding, Shadow,
  border::{Radius, radius},
  widget::{Space, button, container, row, space::horizontal},
};

use crate::ui::{
  app_state::Message,
  theme,
  typography::{self, Typography},
};

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
    ..Default::default()
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
        weight: typography::Weight::Bold,
      }
      .as_text("connected")
    ]
    .padding(0)
    .spacing(4),
  )
  .center_y(12)
  .into()
}
