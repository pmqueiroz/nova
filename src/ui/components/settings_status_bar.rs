use iced::{
  Color, Element, Padding,
  widget::{column, row, space::horizontal, text, text_input, toggler},
};

use super::{input_style, setting_row};
use crate::core::config;
use crate::ui::{app_state::Message, theme};

pub fn status_bar_tab<'a>(settings: &'a config::Config) -> Element<'a, Message> {
  let (fg, fg_muted) = {
    let rt = theme::color::runtime();
    (rt.foreground, rt.foreground_muted)
  };
  let visible = settings.status_bar.visible;

  column![
    row![
      column![
        text("Visible").font(theme::font::bold()).size(12).color(fg),
        text("Show or hide the status bar at the bottom")
          .font(theme::font::regular())
          .size(11)
          .color(fg_muted),
      ]
      .spacing(2),
      horizontal(),
      toggler(visible)
        .on_toggle(Message::SettingsStatusBarToggled)
        .size(20)
        .style(|_t, status| {
          let is_toggled = match status {
            toggler::Status::Active { is_toggled }
            | toggler::Status::Hovered { is_toggled }
            | toggler::Status::Disabled { is_toggled } => is_toggled,
          };
          let rt = theme::color::runtime();
          let (accent, border_c) = (rt.accent, rt.border);
          drop(rt);
          toggler::Style {
            background: if is_toggled {
              accent.into()
            } else {
              theme::color::BG_HIGH.as_color().into()
            },
            background_border_width: 1.0,
            background_border_color: if is_toggled { accent } else { border_c },
            foreground: Color::WHITE.into(),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            text_color: None,
            border_radius: None,
            padding_ratio: 0.15,
          }
        }),
    ]
    .spacing(12)
    .align_y(iced::alignment::Vertical::Center),
    setting_row(
      "Date format",
      "chrono format string for the date",
      text_input("%b %d", &settings.status_bar.date_format)
        .on_input(Message::SettingsDateFormatChanged)
        .font(theme::font::regular())
        .size(12)
        .style(input_style)
        .padding(Padding::from([6, 10]))
        .into(),
    ),
    setting_row(
      "Time format",
      "chrono format string for the time",
      text_input("%H:%M:%S", &settings.status_bar.time_format)
        .on_input(Message::SettingsTimeFormatChanged)
        .font(theme::font::regular())
        .size(12)
        .style(input_style)
        .padding(Padding::from([6, 10]))
        .into(),
    ),
  ]
  .spacing(16)
  .into()
}
