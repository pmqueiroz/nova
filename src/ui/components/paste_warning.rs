use iced::{
  Border, Color, Element, Length, Padding,
  border::Radius,
  mouse,
  widget::{button, column, container, mouse_area, row, rule, scrollable, space, stack, text},
};

use crate::ui::{app_state::Message, theme};

pub fn paste_warning_overlay<'a>(content: &'a str) -> Element<'a, Message> {
  let rt = theme::color::runtime();
  let fg = rt.foreground;
  let fg_muted = rt.foreground_muted;
  let bg = rt.background;
  let border_c = rt.border;
  drop(rt);

  let red = Color {
    r: 0.9,
    g: 0.3,
    b: 0.3,
    a: 1.0,
  };
  let orange = Color {
    r: 0.95,
    g: 0.6,
    b: 0.1,
    a: 1.0,
  };

  let reason = if content.contains('\n') {
    "Contains multiple lines. Pasting may execute unintended commands."
  } else {
    "May contain dangerous commands."
  };

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
  .on_press(Message::PasteWarningCancel)
  .interaction(mouse::Interaction::Idle);

  let header = container(
    row![
      text("Paste Warning")
        .size(13)
        .color(orange)
        .font(theme::font::BOLD),
      space::horizontal(),
      button(text("×").size(14).color(fg_muted))
        .style(|_t, _s| button::Style {
          background: Some(Color::TRANSPARENT.into()),
          ..Default::default()
        })
        .on_press(Message::PasteWarningCancel)
        .padding(Padding::from([2, 6])),
    ]
    .align_y(iced::alignment::Vertical::Center),
  )
  .padding(Padding {
    top: 12.0,
    bottom: 12.0,
    left: 16.0,
    right: 8.0,
  })
  .width(Length::Fill);

  let reason_row = container(
    text(reason)
      .size(12)
      .color(fg_muted)
      .font(theme::font::REGULAR),
  )
  .padding(Padding {
    top: 10.0,
    bottom: 6.0,
    left: 16.0,
    right: 16.0,
  })
  .width(Length::Fill);

  let preview_box = container(
    container(
      scrollable(
        text(content)
          .size(11)
          .color(fg)
          .font(theme::font::REGULAR)
          .width(Length::Fill),
      )
      .height(120)
      .direction(scrollable::Direction::Vertical(
        scrollable::Scrollbar::new()
          .width(4)
          .margin(4)
          .scroller_width(4),
      )),
    )
    .padding(Padding::from([8, 10]))
    .style(move |_| container::Style {
      background: Some(Color { a: 0.06, ..fg }.into()),
      border: Border {
        color: border_c,
        width: 1.0,
        radius: Radius::new(4.0),
      },
      ..Default::default()
    })
    .width(Length::Fill),
  )
  .padding(Padding {
    top: 0.0,
    bottom: 12.0,
    left: 16.0,
    right: 16.0,
  })
  .width(Length::Fill);

  let actions = container(
    row![
      space::horizontal(),
      button(text("Cancel").size(12).color(fg_muted))
        .style(move |_t, status| button::Style {
          background: Some(match status {
            button::Status::Hovered => Color {
              a: 0.08,
              ..fg_muted
            }
            .into(),
            _ => Color::TRANSPARENT.into(),
          },),
          border: Border {
            color: border_c,
            width: 1.0,
            radius: Radius::new(4.0),
          },
          text_color: fg_muted,
          ..Default::default()
        })
        .on_press(Message::PasteWarningCancel)
        .padding(Padding {
          top: 7.0,
          bottom: 5.0,
          left: 12.0,
          right: 12.0
        }),
      button(text("Paste Anyway").size(12).color(red))
        .style(move |_t, status| button::Style {
          background: Some(match status {
            button::Status::Hovered => Color { a: 0.12, ..red }.into(),
            _ => Color::TRANSPARENT.into(),
          },),
          border: Border {
            color: red,
            width: 1.0,
            radius: Radius::new(4.0),
          },
          text_color: red,
          ..Default::default()
        })
        .on_press(Message::PasteWarningConfirm)
        .padding(Padding {
          top: 7.0,
          bottom: 5.0,
          left: 12.0,
          right: 12.0
        }),
    ]
    .spacing(8)
    .align_y(iced::alignment::Vertical::Center),
  )
  .padding(Padding {
    top: 0.0,
    bottom: 14.0,
    left: 16.0,
    right: 16.0,
  })
  .width(Length::Fill);

  let modal_col = column![
    header,
    rule::horizontal(1),
    reason_row,
    preview_box,
    actions
  ];

  let modal = mouse_area(
    container(
      container(modal_col)
        .style(move |_| container::Style {
          background: Some(bg.into()),
          border: Border {
            color: border_c,
            width: 1.0,
            radius: Radius::new(8.0),
          },
          ..Default::default()
        })
        .width(520),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill),
  )
  .on_press(Message::NoOp)
  .interaction(mouse::Interaction::Idle);

  stack![backdrop, modal].into()
}
