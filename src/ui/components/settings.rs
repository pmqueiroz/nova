use iced::{
  Border, Color, Element, Length, Padding, Shadow,
  border::Radius,
  widget::{
    button, column, container, mouse_area,
    row, rule, scrollable,
    space::{horizontal, vertical},
    stack, text,
  },
};

use crate::core::config;
use crate::ui::{
  app_state::{Message, SettingsTab},
  theme,
};

use super::{settings_ai, settings_general, settings_keybindings, settings_status_bar, settings_theme};

const SIDEBAR_WIDTH: f32 = 160.0;

pub fn settings_modal<'a>(
  settings: &'a config::Config,
  active_tab: &'a SettingsTab,
  shell_input: &'a str,
  recording_index: Option<usize>,
  raw_content: &'a str,
  config_path: String,
) -> Element<'a, Message> {
  let backdrop = mouse_area(
    container(iced::widget::Space::new())
      .width(Length::Fill)
      .height(Length::Fill)
      .style(|_| container::Style {
        background: Some(
          Color {
            a: 0.6,
            ..Color::BLACK
          }
          .into(),
        ),
        ..Default::default()
      }),
  )
  .on_press(Message::CloseSettings);

  let modal = mouse_area(
    container(
      container(modal_inner(
        settings,
        active_tab,
        shell_input,
        recording_index,
        raw_content,
        config_path,
      ))
      .style(move |_| {
        let border_c = theme::color::runtime().border;
        container::Style {
          background: Some(theme::color::BG_DEEP.as_color().into()),
          border: Border {
            color: border_c,
            radius: Radius::new(10.0),
            width: 1.0,
          },
          ..Default::default()
        }
      })
      .width(760)
      .height(520),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill),
  )
  .on_press(Message::NoOp);

  stack![backdrop, modal].into()
}

fn modal_inner<'a>(
  settings: &'a config::Config,
  active_tab: &'a SettingsTab,
  shell_input: &'a str,
  recording_index: Option<usize>,
  raw_content: &'a str,
  config_path: String,
) -> Element<'a, Message> {
  let (fg, fg_muted) = {
    let rt = theme::color::runtime();
    (rt.foreground, rt.foreground_muted)
  };

  let header = container(
    row![
      text("Settings").font(theme::font::BOLD).size(14).color(fg),
      horizontal(),
      text(config_path)
        .font(theme::font::REGULAR)
        .size(11)
        .color(fg_muted),
      button(text("×").size(16).color(fg_muted))
        .style(|_t, status| {
          let text_color = match status {
            button::Status::Hovered | button::Status::Pressed => theme::color::RED.as_color(),
            _ => theme::color::runtime().foreground_muted,
          };
          button::Style {
            text_color,
            background: Some(theme::color::TRANSPARENT.as_color().into()),
            ..Default::default()
          }
        })
        .on_press(Message::CloseSettings)
        .padding(Padding::from([0, 8])),
    ]
    .align_y(iced::alignment::Vertical::Center)
    .spacing(8),
  )
  .padding(Padding {
    top: 12.0,
    bottom: 12.0,
    left: 16.0,
    right: 8.0,
  });

  let body = row![
    sidebar_nav(active_tab),
    rule::vertical(1),
    container(
      scrollable(match active_tab {
        SettingsTab::General => settings_general::general_tab(settings, shell_input),
        SettingsTab::Theme => settings_theme::theme_tab(settings),
        SettingsTab::Keybindings => settings_keybindings::keybindings_tab(settings, recording_index),
        SettingsTab::StatusBar => settings_status_bar::status_bar_tab(settings),
        SettingsTab::Ai => settings_ai::ai_tab(settings),
        SettingsTab::Raw => edit_raw_tab(raw_content),
      })
      .height(Length::Fill)
      .direction(scrollable::Direction::Vertical(
        scrollable::Scrollbar::new()
          .width(4)
          .margin(4)
          .scroller_width(4),
      ))
      .style(|_t, _status| scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
          background: None,
          border: Border::default(),
          scroller: scrollable::Scroller {
            background: Color {
              r: 0.5,
              g: 0.5,
              b: 0.5,
              a: 0.35,
            }
            .into(),
            border: Border {
              color: Color::TRANSPARENT,
              radius: Radius::new(4.0),
              width: 0.0,
            },
          },
        },
        horizontal_rail: scrollable::Rail {
          background: None,
          border: Border::default(),
          scroller: scrollable::Scroller {
            background: Color::TRANSPARENT.into(),
            border: Border::default(),
          },
        },
        gap: None,
        auto_scroll: scrollable::AutoScroll {
          background: Color::TRANSPARENT.into(),
          border: Border::default(),
          shadow: Shadow::default(),
          icon: Color::TRANSPARENT,
        },
      }),
    )
    .padding(Padding {
      top: 16.0,
      bottom: 16.0,
      left: 20.0,
      right: 16.0,
    })
    .width(Length::Fill)
    .height(Length::Fill),
  ]
  .height(Length::Fill);

  let footer = container(
    row![
      text("Changes saved automatically")
        .font(theme::font::REGULAR)
        .size(11)
        .color(fg_muted),
      horizontal(),
      button(text("Done").size(12).color(theme::color::BG_DEEP.as_color()))
        .style(|_t, status| {
          let bg = match status {
            button::Status::Hovered | button::Status::Pressed => iced::Color {
              a: 0.85,
              ..theme::color::runtime().accent
            },
            _ => theme::color::runtime().accent,
          };
          button::Style {
            text_color: theme::color::BG_DEEP.as_color(),
            background: Some(bg.into()),
            border: Border {
              radius: Radius::new(4.0),
              ..Default::default()
            },
            ..Default::default()
          }
        })
        .on_press(Message::CloseSettings)
        .padding(Padding::from([6, 16])),
    ]
    .align_y(iced::alignment::Vertical::Center),
  )
  .padding(Padding {
    top: 10.0,
    bottom: 10.0,
    left: 16.0,
    right: 16.0,
  });

  column![
    header,
    rule::horizontal(1),
    body,
    rule::horizontal(1),
    footer
  ]
  .into()
}

fn sidebar_nav<'a>(active_tab: &'a SettingsTab) -> Element<'a, Message> {
  let tabs: &[(SettingsTab, &str)] = &[
    (SettingsTab::General, "General"),
    (SettingsTab::Theme, "Theme"),
    (SettingsTab::Keybindings, "Keybindings"),
    (SettingsTab::StatusBar, "Status Bar"),
    (SettingsTab::Ai, "AI"),
    (SettingsTab::Raw, "Raw"),
  ];

  let mut nav = column![].spacing(2).padding(Padding::from([8, 6]));

  for (tab, label) in tabs {
    let is_active = active_tab == tab;
    let tab_clone = tab.clone();
    let label: &'static str = label;

    let label_color = {
      let rt = theme::color::runtime();
      if is_active { rt.accent } else { rt.foreground_muted }
    };

    nav = nav.push(
      button(
        text(label)
          .font(theme::font::REGULAR)
          .size(12)
          .color(label_color),
      )
      .style(move |_t, status| {
        let rt = theme::color::runtime();
        let text_color = if is_active {
          rt.accent
        } else {
          match status {
            button::Status::Hovered | button::Status::Pressed => rt.foreground,
            _ => rt.foreground_muted,
          }
        };
        let bg_color = if is_active {
          theme::color::BG_HIGH.as_color()
        } else {
          match status {
            button::Status::Hovered | button::Status::Pressed => theme::color::BG_HIGH.as_color(),
            _ => theme::color::TRANSPARENT.as_color(),
          }
        };
        drop(rt);
        button::Style {
          text_color,
          background: Some(bg_color.into()),
          border: Border {
            radius: Radius::new(4.0),
            ..Default::default()
          },
          ..Default::default()
        }
      })
      .on_press(Message::SettingsTabSelected(tab_clone))
      .padding(Padding::from([6, 12]))
      .width(Length::Fill),
    );
  }

  nav = nav.push(vertical());

  nav = nav.push(
    button(
      text("Reset to defaults")
        .font(theme::font::REGULAR)
        .size(11)
        .color(theme::color::RED.as_color()),
    )
    .style(|_t, status| button::Style {
      text_color: theme::color::RED.as_color(),
      background: Some(match status {
        button::Status::Hovered | button::Status::Pressed => iced::Color {
          a: 0.08,
          ..theme::color::RED.as_color()
        }
        .into(),
        _ => theme::color::TRANSPARENT.as_color().into(),
      }),
      border: Border {
        radius: Radius::new(4.0),
        ..Default::default()
      },
      ..Default::default()
    })
    .on_press(Message::SettingsResetAll)
    .padding(Padding::from([6, 12]))
    .width(Length::Fill),
  );

  container(nav)
    .width(SIDEBAR_WIDTH)
    .height(Length::Fill)
    .into()
}

fn edit_raw_tab<'a>(content: &'a str) -> Element<'a, Message> {
  let fg_muted = theme::color::runtime().foreground_muted;
  container(
    text(content)
      .font(theme::font::REGULAR)
      .size(12)
      .color(fg_muted),
  )
  .padding(4)
  .width(Length::Fill)
  .into()
}
