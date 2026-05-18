use iced::{
  Border, Element, Padding,
  border::Radius,
  widget::{button, column, container, pick_list, row, text, text_input},
};

use super::{input_style, setting_row};
use crate::core::config::{self, BellType};
use crate::ui::{app_state::Message, theme};

pub fn general_tab<'a>(settings: &'a config::Config, shell_input: &'a str) -> Element<'a, Message> {
  let mut col = column![].spacing(20);

  col = col.push(setting_row(
    "Editor",
    "External editor for opening files",
    text_input("editor", &settings.general.editor)
      .on_input(Message::SettingsEditorChanged)
      .font(theme::font::regular())
      .size(12)
      .style(input_style)
      .padding(Padding {
        top: 7.0,
        bottom: 5.0,
        left: 10.0,
        right: 10.0,
      })
      .into(),
  ));

  let bell_list: Element<'a, Message> = pick_list(
    [BellType::None, BellType::Audio, BellType::Blink].as_slice(),
    Some(settings.general.bell.clone()),
    Message::SettingsBellChanged,
  )
  .font(theme::font::regular())
  .text_size(12)
  .into();

  col = col.push(setting_row("Bell", "Terminal bell behavior", bell_list));

  let shells = settings.general.shells.as_deref().unwrap_or(&[]);
  let mut chips = row![].spacing(6);
  for (i, shell) in shells.iter().enumerate() {
    let label: &'static str = Box::leak(shell.clone().into_boxed_str());
    let fg = theme::color::runtime().foreground;
    let fg_muted = theme::color::runtime().foreground_muted;
    chips = chips.push(
      container(
        row![
          text(label).font(theme::font::regular()).size(11).color(fg),
          button(text("×").size(10).color(fg_muted))
            .style(|_t, _s| button::Style {
              background: Some(theme::color::TRANSPARENT.as_color().into()),
              ..Default::default()
            })
            .on_press(Message::SettingsRemoveShell(i))
            .padding(Padding::from([0, 4])),
        ]
        .align_y(iced::alignment::Vertical::Center)
        .spacing(2),
      )
      .style(|_| {
        let border_c = theme::color::runtime().border;
        container::Style {
          background: Some(theme::color::BG_HIGH.as_color().into()),
          border: Border {
            color: border_c,
            radius: Radius::new(4.0),
            width: 1.0,
          },
          ..Default::default()
        }
      })
      .padding(Padding::from([3, 8])),
    );
  }

  let shells_widget: Element<'a, Message> = column![
    chips,
    text_input("add shell…", shell_input)
      .on_input(Message::SettingsShellInputChanged)
      .on_submit(Message::SettingsAddShell)
      .font(theme::font::regular())
      .size(12)
      .style(input_style)
      .padding(Padding {
        top: 7.0,
        bottom: 5.0,
        left: 10.0,
        right: 10.0
      }),
  ]
  .spacing(8)
  .into();

  col = col.push(setting_row(
    "Shells",
    "Available shells for new tabs",
    shells_widget,
  ));
  col.into()
}
