use iced::{
  Border, Color, Element, Padding,
  border::Radius,
  widget::{button, column, container, pick_list, row, text, text_input, toggler},
};

use super::{input_style, section_label, setting_row};
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

  let initial_cmd_val = settings
    .general
    .initial_command
    .as_deref()
    .unwrap_or("")
    .to_string();
  col = col.push(
    column![
      section_label("STARTUP"),
      column![
        setting_row(
          "Initial command",
          "Command to run instead of shell (empty = default shell)",
          text_input("e.g. ssh server", &initial_cmd_val)
            .on_input(Message::SettingsInitialCommandChanged)
            .font(theme::font::REGULAR)
            .size(12)
            .style(input_style)
            .padding(Padding {
              top: 7.0,
              bottom: 5.0,
              left: 10.0,
              right: 10.0,
            })
            .into(),
        ),
        setting_row(
          "Wait after exit",
          "Keep terminal open when command exits",
          {
            let is_on = settings.general.wait_after_command;
            let wait_toggle: Element<'a, Message> = toggler(is_on)
              .on_toggle(Message::SettingsWaitAfterCommandToggled)
              .size(20)
              .style(move |_t, status| {
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
              })
              .into();
            wait_toggle
          },
        ),
      ]
      .spacing(16),
    ]
    .spacing(8),
  );

  col.into()
}
