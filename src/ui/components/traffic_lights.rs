use iced::{
  Color, Element, Length,
  widget::{button, row},
};

use crate::ui::app_state::Message;

pub fn traffic_lights(window_focused: bool) -> Element<'static, Message> {
  use crate::ui::theme;
  use iced::border::radius;
  use iced::widget::Space;

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
          theme::color::TRAFFIC_LIGHT_INACTIVE.as_color()
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

  row![
    circle_btn(theme::color::TRAFFIC_LIGHT_RED.as_color(), Message::CloseWindow),
    circle_btn(theme::color::TRAFFIC_LIGHT_YELLOW.as_color(), Message::MinimizeWindow),
    circle_btn(theme::color::TRAFFIC_LIGHT_GREEN.as_color(), Message::MaximizeWindow),
  ]
  .spacing(8)
  .into()
}

pub fn system_controls(_window_focused: bool) -> Element<'static, Message> {
  let win_btn = |icon_unicode: &'static str, msg: Message, is_close: bool| {
    use iced::widget::{container, text};

    let content = container(text(icon_unicode).size(16))
      .center_x(Length::Fill)
      .center_y(Length::Fill);

    button(content)
      .on_press(msg)
      .width(Length::Fixed(46.0))
      .height(Length::Fill)
      .padding(0)
      .style(move |_theme, status| {
        let is_hovered = status == iced::widget::button::Status::Hovered;

        iced::widget::button::Style {
          background: if is_hovered {
            if is_close {
              Some(Color::from_rgb8(232, 17, 35).into())
            } else {
              Some(Color::from_rgb(0.2, 0.2, 0.2).into())
            }
          } else {
            None
          },
          text_color: Color::WHITE,
          border: iced::Border::default(),
          ..Default::default()
        }
      })
  };

  row![
    win_btn("─", Message::MinimizeWindow, false),
    win_btn("□", Message::MaximizeWindow, false),
    win_btn("✕", Message::CloseWindow, true),
  ]
  .spacing(8)
  .height(Length::Fill)
  .into()
}
