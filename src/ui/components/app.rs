use iced::{Border, Element, Length, border::Radius, widget::container};

use crate::ui::{app_state::Message, theme};

pub fn app<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
  container(content)
    .style(move |_| container::Style {
      background: Some(theme::color::BG_DEEP.as_color().into()),
      border: Border {
        color: theme::color::TRANSPARENT.as_color(),
        radius: Radius::new(0.0),
        width: 0.0,
      },
      ..container::Style::default()
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}
