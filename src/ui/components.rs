use chrono::Local;
use iced::{
  Element, Length, Padding, alignment, color,
  widget::{container, row, space::horizontal},
};

use crate::ui::{app_state::Message, typography::Typography};

pub fn cursor<'a>() -> Element<'a, Message> {
  Typography {
    color: color!(62, 207, 142),
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
