use iced::{
  Border, Element, Padding,
  border::Radius,
  widget::{button, column, container, row, space::horizontal, text},
};

use super::btn_subtle_style;
use crate::core::config;
use crate::ui::{app_state::Message, theme};

pub fn keybindings_tab<'a>(
  settings: &'a config::Config,
  recording_index: Option<usize>,
) -> Element<'a, Message> {
  let kb = &settings.keybindings;
  let entries: &[(usize, &str, &str)] = &[
    (0, "New tab", kb.new_tab.as_str()),
    (1, "Close tab", kb.close_tab.as_str()),
    (2, "Next tab", kb.next_tab.as_str()),
    (3, "Previous tab", kb.prev_tab.as_str()),
    (4, "Paste", kb.paste.as_str()),
    (5, "Copy", kb.copy.as_str()),
  ];

  let fg_muted = theme::color::runtime().foreground_muted;

  let hint = text("Click a binding and press the desired keys. Esc to cancel.")
    .font(theme::font::regular())
    .size(11)
    .color(fg_muted);

  let mut rows: iced::widget::Column<'a, Message> = column![hint].spacing(12);

  for (idx, label, binding) in entries {
    let idx = *idx;
    let is_recording = recording_index == Some(idx);
    let label: &'static str = label;
    let binding = binding.to_string();

    let (fg, accent, fg_muted2) = {
      let rt = theme::color::runtime();
      (rt.foreground, rt.accent, rt.foreground_muted)
    };

    let binding_widget: Element<'a, Message> = if is_recording {
      container(
        row![
          text("Recording…")
            .font(theme::font::regular())
            .size(12)
            .color(accent),
          horizontal(),
          button(text("Cancel").size(11).color(fg_muted2))
            .style(btn_subtle_style)
            .on_press(Message::SettingsCancelRecordKb)
            .padding(Padding::from([2, 8])),
        ]
        .align_y(iced::alignment::Vertical::Center)
        .spacing(8),
      )
      .style(move |_| container::Style {
        background: Some(theme::color::BG_HIGH.as_color().into()),
        border: Border {
          color: accent,
          radius: Radius::new(4.0),
          width: 1.0,
        },
        ..Default::default()
      })
      .padding(Padding::from([6, 10]))
      .width(200)
      .into()
    } else {
      button(
        text(binding)
          .font(theme::font::regular())
          .size(12)
          .color(fg),
      )
      .style(btn_subtle_style)
      .on_press(Message::SettingsStartRecordKb(idx))
      .padding(Padding::from([5, 10]))
      .width(200)
      .into()
    };

    let reset_btn = button(text("Reset").size(11).color(fg_muted2))
      .style(btn_subtle_style)
      .on_press(Message::SettingsResetKb(idx))
      .padding(Padding::from([4, 8]));

    rows = rows.push(
      row![
        container(text(label).font(theme::font::regular()).size(12).color(fg)).width(140),
        binding_widget,
        reset_btn,
      ]
      .spacing(12)
      .align_y(iced::alignment::Vertical::Center),
    );
  }

  rows.into()
}
