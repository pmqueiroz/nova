use iced::{
  Border, Color, Element, Length, Padding, Shadow,
  border::Radius,
  widget::{
    button, column, container, mouse_area,
    overlay::menu,
    pick_list, row, rule, scrollable,
    space::{horizontal, vertical},
    stack, text, text_input, toggler,
  },
};

use crate::core::config::{self, BellType};
use crate::ui::{
  app_state::{ColorField, Message, SettingsTab},
  theme,
};

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
        SettingsTab::General => general_tab(settings, shell_input),
        SettingsTab::Theme => theme_tab(settings),
        SettingsTab::Keybindings => keybindings_tab(settings, recording_index),
        SettingsTab::StatusBar => status_bar_tab(settings),
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
              a: 0.35
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
      right: 16.0
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
      button(
        text("Done")
          .size(12)
          .color(theme::color::BG_DEEP.as_color())
      )
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
    (SettingsTab::Raw, "Raw"),
  ];

  let mut nav = column![].spacing(2).padding(Padding::from([8, 6]));

  for (tab, label) in tabs {
    let is_active = active_tab == tab;
    let tab_clone = tab.clone();
    let label: &'static str = label;

    let label_color = {
      let rt = theme::color::runtime();
      if is_active {
        rt.accent
      } else {
        rt.foreground_muted
      }
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

fn general_tab<'a>(settings: &'a config::Config, shell_input: &'a str) -> Element<'a, Message> {
  let mut col = column![].spacing(20);

  col = col.push(setting_row(
    "Editor",
    "External editor for opening files",
    text_input("editor", &settings.general.editor)
      .on_input(Message::SettingsEditorChanged)
      .font(theme::font::REGULAR)
      .size(12)
      .style(input_style)
      .padding(Padding::from([6, 10]))
      .into(),
  ));

  let bell_list: Element<'a, Message> = pick_list(
    [BellType::None, BellType::Audio, BellType::Blink].as_slice(),
    Some(settings.general.bell.clone()),
    Message::SettingsBellChanged,
  )
  .font(theme::font::REGULAR)
  .text_size(12)
  .style(|_t, status| {
    let rt = theme::color::runtime();
    let (border_c, fg, fg_muted) = (rt.border, rt.foreground, rt.foreground_muted);
    let accent = rt.accent;
    drop(rt);
    let active_border = match status {
      pick_list::Status::Opened { .. } | pick_list::Status::Hovered => accent,
      _ => border_c,
    };
    pick_list::Style {
      text_color: fg,
      background: theme::color::BG_HIGH.as_color().into(),
      border: Border {
        color: active_border,
        radius: Radius::new(4.0),
        width: 1.0,
      },
      handle_color: fg_muted,
      placeholder_color: fg_muted,
    }
  })
  .menu_style(|_t| {
    let rt = theme::color::runtime();
    let (fg, accent, border_c) = (rt.foreground, rt.accent, rt.border);
    drop(rt);
    menu::Style {
      background: theme::color::BG_DEEP.as_color().into(),
      border: Border {
        color: border_c,
        radius: Radius::new(6.0),
        width: 1.0,
      },
      text_color: fg,
      selected_text_color: theme::color::BG_DEEP.as_color(),
      selected_background: accent.into(),
      shadow: Shadow::default(),
    }
  })
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
          text(label).font(theme::font::REGULAR).size(11).color(fg),
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
      .font(theme::font::REGULAR)
      .size(12)
      .style(input_style)
      .padding(Padding::from([6, 10])),
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

fn theme_tab<'a>(settings: &'a config::Config) -> Element<'a, Message> {
  let mut col = column![].spacing(24);

  let font_size = settings.theme.font.size;
  let fg = theme::color::runtime().foreground;

  let family_input: Element<'a, Message> = column![
    text_input("font family", &settings.theme.font.family)
      .on_input(Message::SettingsFontFamilyChanged)
      .font(theme::font::REGULAR)
      .size(12)
      .style(input_style)
      .padding(Padding::from([6, 10])),
    text("Requires restart to take effect")
      .font(theme::font::REGULAR)
      .size(10)
      .color(theme::color::runtime().foreground_muted),
  ]
  .spacing(4)
  .into();

  let size_control: Element<'a, Message> = row![
    button(text("−").size(14).color(fg))
      .style(btn_subtle_style)
      .on_press(Message::SettingsFontSizeChanged(font_size - 1.0))
      .padding(Padding::from([4, 10])),
    container(
      text(format!("{}", font_size as u32))
        .font(theme::font::REGULAR)
        .size(12)
        .color(fg),
    )
    .padding(Padding::from([4, 12]))
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
    }),
    button(text("+").size(14).color(fg))
      .style(btn_subtle_style)
      .on_press(Message::SettingsFontSizeChanged(font_size + 1.0))
      .padding(Padding::from([4, 10])),
  ]
  .spacing(4)
  .align_y(iced::alignment::Vertical::Center)
  .into();

  col = col.push(
    column![
      section_label("FONT"),
      column![
        setting_row("Family", "Monospace font family", family_input),
        setting_row("Size", "Font size in points", size_control),
      ]
      .spacing(16),
    ]
    .spacing(8),
  );

  let c = &settings.theme.colors;
  col = col.push(
    column![
      section_label("COLORS"),
      column![
        color_row("Background", &c.background, ColorField::Background),
        color_row("Foreground", &c.foreground, ColorField::Foreground),
        color_row("Accent", &c.accent, ColorField::Accent),
        color_row(
          "Foreground muted",
          &c.foreground_muted,
          ColorField::ForegroundMuted
        ),
        color_row("Border", &c.border, ColorField::Border),
        color_row("Cursor", &c.cursor, ColorField::Cursor),
      ]
      .spacing(12),
    ]
    .spacing(8),
  );

  col.into()
}

fn keybindings_tab<'a>(
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
    .font(theme::font::REGULAR)
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
            .font(theme::font::REGULAR)
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
      button(text(binding).font(theme::font::REGULAR).size(12).color(fg))
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
        container(text(label).font(theme::font::REGULAR).size(12).color(fg)).width(140),
        binding_widget,
        reset_btn,
      ]
      .spacing(12)
      .align_y(iced::alignment::Vertical::Center),
    );
  }

  rows.into()
}

fn status_bar_tab<'a>(settings: &'a config::Config) -> Element<'a, Message> {
  let (fg, fg_muted) = {
    let rt = theme::color::runtime();
    (rt.foreground, rt.foreground_muted)
  };
  let visible = settings.status_bar.visible;

  column![
    row![
      column![
        text("Visible").font(theme::font::BOLD).size(12).color(fg),
        text("Show or hide the status bar at the bottom")
          .font(theme::font::REGULAR)
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
        .font(theme::font::REGULAR)
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
        .font(theme::font::REGULAR)
        .size(12)
        .style(input_style)
        .padding(Padding::from([6, 10]))
        .into(),
    ),
  ]
  .spacing(16)
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

fn setting_row<'a>(
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
      text(label).font(theme::font::BOLD).size(12).color(fg),
      text(description)
        .font(theme::font::REGULAR)
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

fn color_row<'a>(label: &'static str, hex: &'a str, field: ColorField) -> Element<'a, Message> {
  let swatch_color = config::parse_hex_color(hex).unwrap_or(theme::color::RED.as_color());
  let fg = theme::color::runtime().foreground;
  let hex_owned = hex.to_string();

  row![
    container(iced::widget::Space::new().width(20).height(20)).style(move |_| container::Style {
      background: Some(swatch_color.into()),
      border: Border {
        color: theme::color::runtime().border,
        radius: Radius::new(4.0),
        width: 1.0
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

fn section_label<'a>(label: &'static str) -> Element<'a, Message> {
  let fg_muted = theme::color::runtime().foreground_muted;
  text(label)
    .font(theme::font::BOLD)
    .size(10)
    .color(fg_muted)
    .into()
}

fn input_style(_t: &iced::Theme, status: text_input::Status) -> text_input::Style {
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

fn btn_subtle_style(_t: &iced::Theme, status: button::Status) -> button::Style {
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
