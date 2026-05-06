use iced::{
  Color, Element, Length,
  border::radius,
  widget::{Space, button, row},
};

use crate::ui::{app_state::Message, theme};

// TODO: windows icons
#[cfg(target_os = "windows")]
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
