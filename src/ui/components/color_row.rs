use iced::{Border, Element, Padding, border::Radius, widget::{container, row, text, text_input}};

use crate::core::config;
use crate::ui::{app_state::{ColorField, Message}, theme};
use super::input_style;

pub fn color_row<'a>(label: &'static str, hex: &'a str, field: ColorField) -> Element<'a, Message> {
  let swatch_color = config::parse_hex_color(hex).unwrap_or(theme::color::RED.as_color());
  let fg = theme::color::runtime().foreground;
  let hex_owned = hex.to_string();

  row![
    container(iced::widget::Space::new().width(20).height(20)).style(move |_| container::Style {
      background: Some(swatch_color.into()),
      border: Border {
        color: theme::color::runtime().border,
        radius: Radius::new(4.0),
        width: 1.0,
      },
      ..Default::default()
    }),
    text(label)
      .font(theme::font::REGULAR)
      .size(12)
      .color(fg)
      .width(140),
    text_input("#hex", &hex_owned)
      .on_input(move |s| Message::SettingsColorChanged(field.clone(), s))
      .font(theme::font::REGULAR)
      .size(12)
      .style(input_style)
      .padding(Padding::from([5, 10]))
      .width(120),
  ]
  .spacing(10)
  .align_y(iced::alignment::Vertical::Center)
  .into()
}
