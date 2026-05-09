use iced::{
  Border, Element, Length, Padding,
  border::Radius,
  widget::{button, column, container, mouse_area},
};

use crate::ui::{app_state::Message, theme, typography::Typography};

const PANEL_WIDTH: f32 = 180.0;

pub fn shell_picker(
  shells: &[String],
  anchor_x: f32,
  window_width: f32,
) -> Element<'static, Message> {
  let mut items = column![].spacing(2);

  for shell in shells {
    let shell_for_msg = shell.clone();
    let shell_label = shell.clone();
    items = items.push(
      button(
        Typography {
          color: theme::color::runtime().foreground,
          size: 12.into(),
          ..Default::default()
        }
        .as_text(shell_label),
      )
      .style(move |_t, status| button::Style {
        text_color: match status {
          button::Status::Hovered | button::Status::Pressed => theme::color::runtime().accent,
          _ => theme::color::runtime().foreground,
        },
        background: Some(
          match status {
            button::Status::Hovered | button::Status::Pressed => theme::color::BG_HIGH.as_color(),
            _ => theme::color::TRANSPARENT.as_color(),
          }
          .into(),
        ),
        border: Border {
          radius: Radius::new(4.0),
          ..Default::default()
        },
        ..Default::default()
      })
      .on_press(Message::NewTabWithShell(shell_for_msg))
      .padding(Padding::from([6, 12]))
      .width(Length::Fill),
    );
  }

  let panel = container(items)
    .style(move |_| container::Style {
      background: Some(theme::color::BG_DEEP.as_color().into()),
      border: Border {
        color: theme::color::runtime().border,
        radius: Radius::new(8.0),
        width: 1.0,
      },
      ..Default::default()
    })
    .padding(4)
    .width(180);

  let left = anchor_x.min(window_width - PANEL_WIDTH - 8.0).max(0.0);

  let positioned = container(panel)
    .padding(Padding {
      top: 76.0,
      left,
      ..Default::default()
    })
    .width(Length::Fill)
    .height(Length::Fill);

  mouse_area(positioned)
    .on_press(Message::CloseShellPicker)
    .into()
}
