use std::sync::LazyLock;

use iced::{
  Border, Color, Element, Length, Padding,
  border::Radius,
  widget::{Id, column, container, mouse_area, row, stack, text, text_input},
};

use crate::ui::{app_state::Message, theme};

pub static PALETTE_INPUT_ID: LazyLock<Id> = LazyLock::new(Id::unique);

struct Cmd {
  id: &'static str,
  label: &'static str,
  description: &'static str,
}

const COMMANDS: &[Cmd] = &[
  Cmd {
    id: "ask_ai",
    label: "Ask AI",
    description: "Ask a question with terminal context",
  },
  Cmd {
    id: "explain_error",
    label: "Explain Error",
    description: "Explain errors in the last output",
  },
  Cmd {
    id: "new_tab",
    label: "New Tab",
    description: "Open a new terminal tab",
  },
  Cmd {
    id: "settings",
    label: "Settings",
    description: "Open settings",
  },
];

fn fuzzy(query: &str, target: &str) -> bool {
  if query.is_empty() {
    return true;
  }
  let mut chars = query.chars().peekable();
  for c in target.chars() {
    if chars.peek().map(|q| q.to_ascii_lowercase()) == Some(c.to_ascii_lowercase()) {
      chars.next();
    }
    if chars.peek().is_none() {
      return true;
    }
  }
  false
}

fn filtered(query: &str) -> Vec<(usize, &'static Cmd)> {
  COMMANDS
    .iter()
    .enumerate()
    .filter(|(_, cmd)| fuzzy(query, cmd.label))
    .collect()
}

pub fn palette_filtered_count(query: &str) -> usize {
  filtered(query).len()
}

pub fn palette_command_id_at(query: &str, selected: usize) -> Option<&'static str> {
  filtered(query).get(selected).map(|(_, cmd)| cmd.id)
}

pub fn command_palette<'a>(query: &'a str, selected: usize) -> Element<'a, Message> {
  let rt = theme::color::runtime();
  let fg = rt.foreground;
  let fg_muted = rt.foreground_muted;
  let accent = rt.accent;
  let bg = rt.background;
  let border_c = rt.border;
  drop(rt);

  let backdrop = mouse_area(
    container(iced::widget::Space::new())
      .width(Length::Fill)
      .height(Length::Fill)
      .style(|_| container::Style {
        background: Some(
          Color {
            a: 0.55,
            ..Color::BLACK
          }
          .into(),
        ),
        ..Default::default()
      }),
  )
  .on_press(Message::CloseCommandPalette);

  let input_row = container(
    text_input("Type a command…", query)
      .id(PALETTE_INPUT_ID.clone())
      .on_input(Message::PaletteQueryChanged)
      .size(13)
      .padding(Padding::from([0, 4]))
      .style(move |_t, _status| text_input::Style {
        background: iced::Background::Color(Color::TRANSPARENT),
        border: Border::default(),
        icon: fg_muted,
        placeholder: fg_muted,
        value: fg,
        selection: Color { a: 0.3, ..accent },
      }),
  )
  .padding(Padding {
    top: 12.0,
    bottom: 12.0,
    left: 16.0,
    right: 16.0,
  })
  .style(move |_| container::Style {
    border: Border {
      color: border_c,
      width: 0.0,
      radius: Radius::default(),
    },
    ..Default::default()
  })
  .width(Length::Fill);

  let results = filtered(query);

  let mut items = column![].spacing(0);
  for (i, (_, cmd)) in results.iter().enumerate() {
    let is_sel = i == selected;
    let item_bg = if is_sel {
      Color { a: 0.12, ..accent }
    } else {
      Color::TRANSPARENT
    };
    let label_color = if is_sel { accent } else { fg };
    let desc_color = fg_muted;
    let idx = i;

    let item = mouse_area(
      container(
        row![
          column![
            text(cmd.label)
              .size(13)
              .color(label_color)
              .font(theme::font::REGULAR),
            text(cmd.description)
              .size(11)
              .color(desc_color)
              .font(theme::font::REGULAR),
          ]
          .spacing(2),
        ]
        .spacing(8),
      )
      .padding(Padding {
        top: 10.0,
        bottom: 10.0,
        left: 16.0,
        right: 16.0,
      })
      .width(Length::Fill)
      .style(move |_| container::Style {
        background: Some(item_bg.into()),
        ..Default::default()
      }),
    )
    .on_press(Message::PaletteSelectAndConfirm(idx));

    items = items.push(item);
  }

  let footer = container(row![
    text("↑↓ navigate")
      .size(10)
      .color(fg_muted)
      .font(theme::font::REGULAR),
    text("  ↵ confirm")
      .size(10)
      .color(fg_muted)
      .font(theme::font::REGULAR),
    text("  esc close")
      .size(10)
      .color(fg_muted)
      .font(theme::font::REGULAR),
  ])
  .padding(Padding {
    top: 8.0,
    bottom: 10.0,
    left: 16.0,
    right: 16.0,
  })
  .style(move |_| container::Style {
    border: Border {
      color: border_c,
      width: 1.0,
      radius: Radius::default(),
    },
    ..Default::default()
  })
  .width(Length::Fill);

  let modal_content = column![
    input_row,
    container(iced::widget::rule::horizontal(1))
      .style(move |_| container::Style {
        border: Border {
          color: border_c,
          ..Default::default()
        },
        ..Default::default()
      })
      .width(Length::Fill),
    items,
    footer,
  ];

  let modal = mouse_area(
    container(
      container(modal_content)
        .style(move |_| container::Style {
          background: Some(bg.into()),
          border: Border {
            color: border_c,
            width: 1.0,
            radius: Radius::new(8.0),
          },
          ..Default::default()
        })
        .width(560),
    )
    .center_x(Length::Fill)
    .padding(Padding {
      top: 80.0,
      ..Default::default()
    })
    .width(Length::Fill)
    .height(Length::Fill),
  )
  .on_press(Message::NoOp);

  stack![backdrop, modal].into()
}
