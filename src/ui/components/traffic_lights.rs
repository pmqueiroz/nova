use iced::widget::{button, container, row};
use iced::{Alignment, Background, Border, Element, Length};

use crate::ui::app_state::Message;
use crate::ui::components::title_bar::light;
use crate::ui::icons::{Icon, icon};
use crate::ui::theme::tokens::{self, height, space};

pub fn traffic_lights() -> Element<'static, Message> {
  row![
    light(tokens::LIGHT_CLOSE, Message::CloseWindow),
    light(tokens::LIGHT_MIN, Message::MinimizeWindow),
    light(tokens::LIGHT_MAX, Message::MaximizeWindow),
  ]
  .spacing(space::S8)
  .align_y(Alignment::Center)
  .into()
}

pub const SYSTEM_BUTTON_WIDTH: f32 = 48.0;

pub fn system_button_style(close: bool, status: button::Status) -> button::Style {
  let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
  button::Style {
    background: hovered.then_some(Background::Color(if close {
      tokens::LIGHT_CLOSE
    } else {
      tokens::SURFACE_2
    })),
    text_color: if hovered { tokens::FG_1 } else { tokens::FG_2 },
    border: Border::default(),
    ..button::Style::default()
  }
}

pub fn system_controls() -> Element<'static, Message> {
  let win_btn = |i: Icon, msg: Message, close: bool| {
    button(
      container(icon(i, 16.0, tokens::FG_2))
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    )
    .on_press(msg)
    .width(SYSTEM_BUTTON_WIDTH)
    .height(height::H56)
    .padding(0)
    .style(move |_t, s| system_button_style(close, s))
  };

  row![
    win_btn(Icon::Minimize, Message::MinimizeWindow, false),
    win_btn(Icon::Maximize, Message::MaximizeWindow, false),
    win_btn(Icon::Close, Message::CloseWindow, true),
  ]
  .height(height::H56)
  .into()
}
