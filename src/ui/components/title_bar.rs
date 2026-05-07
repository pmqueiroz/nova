use iced::{
  Border, Element, Length, Padding,
  border::Radius,
  widget::{container, mouse_area, row, space::horizontal},
};

use crate::ui::{
  app_state::Message, components::traffic_lights, helpers::til_home, theme, typography::Typography,
};

pub fn title_bar(window_focused: bool, pwd: &String) -> Element<'static, Message> {
  let controls = traffic_lights(window_focused);
  let title = Typography {
    color: theme::color::FG_MUTED.as_color(),
    size: 12.into(),
    ..Default::default()
  }
  .as_text(format!("nova - {}", til_home(pwd)));

  #[cfg(target_os = "windows")]
  let title_row = row![horizontal(), title, horizontal(), controls,]
    .spacing(8)
    .align_y(iced::Alignment::Center);
  #[cfg(not(target_os = "windows"))]
  let title_row = row![controls, horizontal(), title, horizontal(),].spacing(8);

  mouse_area(
    container(title_row)
      .style(move |_| container::Style {
        background: Some(theme::color::BG_DEEP.as_color().into()),
        border: Border {
          color: theme::color::BORDER.as_color(),
          radius: Radius {
            top_left: 12.0,
            top_right: 12.0,
            ..Default::default()
          },
          width: 0.5,
        },
        ..container::Style::default()
      })
      .padding(Padding {
        #[cfg(target_os = "windows")]
        left: 46.0 * 3.0,
        #[cfg(not(target_os = "windows"))]
        left: 16.0,
        ..Default::default()
      })
      .width(Length::Fill)
      .center_y(40),
  )
  .on_press(Message::DragWindow)
  .into()
}
