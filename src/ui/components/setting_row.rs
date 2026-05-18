use iced::{
  Border, Element,
  border::Radius,
  widget::{button, column, container, row, space::horizontal, text, text_input},
};

use crate::ui::{app_state::Message, theme};

pub fn setting_row<'a>(
  label: &'static str,
  description: &'static str,
  control: Element<'a, Message>,
) -> Element<'a, Message> {
  let (fg, fg_muted) = {
    let rt = theme::color::runtime();
    (rt.foreground, rt.foreground_muted)
  };

  row![
    column![
      text(label).font(theme::font::bold()).size(12).color(fg),
      text(description)
        .font(theme::font::regular())
        .size(11)
        .color(fg_muted),
    ]
    .spacing(2)
    .width(180),
    horizontal(),
    container(control).width(220),
  ]
  .spacing(12)
  .align_y(iced::alignment::Vertical::Center)
  .into()
}

pub fn input_style(_t: &iced::Theme, status: text_input::Status) -> text_input::Style {
  let rt = theme::color::runtime();
  let accent = rt.accent;
  let fg_muted = rt.foreground_muted;
  let border_c = rt.border;
  let fg = rt.foreground;
  drop(rt);

  text_input::Style {
    background: theme::color::BG_HIGH.as_color().into(),
    border: Border {
      color: match status {
        text_input::Status::Focused { .. } => accent,
        text_input::Status::Hovered => fg_muted,
        _ => border_c,
      },
      radius: Radius::new(4.0),
      width: 1.0,
    },
    icon: fg_muted,
    placeholder: fg_muted,
    value: fg,
    selection: iced::Color { a: 0.3, ..accent },
  }
}

pub fn btn_subtle_style(_t: &iced::Theme, status: button::Status) -> button::Style {
  let rt = theme::color::runtime();
  let fg = rt.foreground;
  let fg_muted = rt.foreground_muted;
  let border_c = rt.border;
  drop(rt);

  button::Style {
    text_color: match status {
      button::Status::Hovered | button::Status::Pressed => fg,
      _ => fg_muted,
    },
    background: Some(match status {
      button::Status::Hovered | button::Status::Pressed => theme::color::BG_HIGH.as_color().into(),
      _ => theme::color::TRANSPARENT.as_color().into(),
    }),
    border: Border {
      color: border_c,
      radius: Radius::new(4.0),
      width: match status {
        button::Status::Hovered => 1.0,
        _ => 0.0,
      },
    },
    ..Default::default()
  }
}
