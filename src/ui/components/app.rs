use iced::{Border, Element, Length, border::Radius, widget::container};

use crate::ui::{app_state::Message, theme};

pub fn app<'a>(content: impl Into<Element<'a, Message>>, maximized: bool) -> Element<'a, Message> {
  #[cfg(target_os = "macos")]
  let style = {
    let _ = maximized;
    container::Style {
      background: Some(theme::color::BG_DEEP.as_color().into()),
      border: Border {
        color: theme::color::BORDER.as_color(),
        radius: Radius::new(12.0),
        width: 0.5,
      },
      ..container::Style::default()
    }
  };

  #[cfg(target_os = "linux")]
  let style = container::Style {
    background: Some(theme::color::BG_DEEP.as_color().into()),
    border: Border {
      color: theme::color::BORDER.as_color(),
      radius: if maximized {
        Radius::new(0.0)
      } else {
        Radius::new(12.0)
      },
      width: if maximized { 0.0 } else { 0.5 },
    },
    ..container::Style::default()
  };

  #[cfg(not(any(target_os = "macos", target_os = "linux")))]
  let style = {
    let _ = maximized;
    container::Style {
      background: Some(theme::color::BG_DEEP.as_color().into()),
      border: Border {
        color: theme::color::TRANSPARENT.as_color(),
        radius: Radius::new(0.0),
        width: 0.0,
      },
      ..container::Style::default()
    }
  };

  container(content)
    .style(move |_| style)
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}
