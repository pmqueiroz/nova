use iced::{
  Color, Element, Length,
  widget::{button, row},
};

use crate::ui::{app_state::Message};

// TODO: windows icons
#[cfg(target_os = "windows")]
pub fn traffic_lights(_: bool) -> Element<'static, Message> {
let win_btn = |icon_unicode: &'static str, msg: Message, is_close: bool| {
  use iced::widget::{container, text};

  let content = container(
        text(icon_unicode)
            .size(16)
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill);

  button(
      content
  )
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
let minimize_btn = win_btn("─", Message::MinimizeWindow, false);
let maximize_btn = win_btn("□", Message::MaximizeWindow, false); // Or use \u{EAC2} if already maximized
let close_btn = win_btn("✕", Message::CloseWindow, true);

  row![minimize_btn, maximize_btn, close_btn].spacing(8).height(Length::Fill).into()
}

#[cfg(not(target_os = "windows"))]
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

  let red_light = circle_btn(
    theme::color::TRAFFIC_LIGHT_RED.as_color(),
    Message::CloseWindow,
  );
  let yellow_light = circle_btn(
    theme::color::TRAFFIC_LIGHT_YELLOW.as_color(),
    Message::MinimizeWindow,
  );
  let green_light = circle_btn(
    theme::color::TRAFFIC_LIGHT_GREEN.as_color(),
    Message::MaximizeWindow,
  );

  row![red_light, yellow_light, green_light].spacing(8).into()
}
