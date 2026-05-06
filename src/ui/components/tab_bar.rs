use iced::{
  Border, Element, Length, Padding, alignment,
  border::Radius,
  widget::{button, container, row},
};

use crate::ui::{
  app_state::{Message, Tab},
  theme,
  typography::Typography,
};

pub fn tab_bar(tabs: &Vec<Tab>, active_index: usize) -> Element<'static, Message> {
  let mut tab_bar = row![];

  for (i, tab) in tabs.iter().enumerate() {
    let is_active = i == active_index;

    tab_bar = tab_bar.push(row![tab_item(format!("Terminal {}", tab.id), i, is_active)].spacing(2));
  }

  tab_bar = tab_bar.push(
    container(
      button("+")
        .style(move |_t, status| button::Style {
          text_color: match status {
            button::Status::Hovered => theme::color::ACCENT.as_color(),
            button::Status::Pressed => theme::color::ACCENT_DIM.as_color(),
            _ => theme::color::FG_MUTED.as_color(),
          },
          background: Some(theme::color::TRANSPARENT.as_color().into()),
          ..Default::default()
        })
        .on_press(Message::NewTab)
        .padding(4),
    )
    .padding(Padding::from([0, 8])),
  );

  container(tab_bar)
    .padding(Padding::from([0, 8]))
    .width(Length::Fill)
    .style(move |_| container::Style {
      background: Some(theme::color::BG_DEEP.as_color().into()),
      border: Border {
        color: theme::color::BORDER.as_color(),
        radius: Radius {
          ..Default::default()
        },
        width: 0.5,
      },
      ..Default::default()
    })
    .height(36)
    .align_y(alignment::Vertical::Bottom)
    .into()
}

fn tab_item(title: String, index: usize, active: bool) -> Element<'static, Message> {
  button(
    container(
      row![
        button(
          Typography {
            ..Default::default()
          }
          .as_text(title),
        )
        .style(move |_t, _s| button::Style {
          background: Some(theme::color::TRANSPARENT.as_color().into()),
          ..Default::default()
        })
        .padding(0)
        .on_press(Message::SwitchTab(index)),
        button("✕")
          .style(move |_t, status| button::Style {
            text_color: if status == button::Status::Hovered {
              theme::color::RED.as_color()
            } else {
              theme::color::FG_MUTED.as_color()
            },
            background: Some(theme::color::TRANSPARENT.as_color().into()),
            ..Default::default()
          })
          .on_press(Message::CloseTab(index))
          .padding(0),
      ]
      .spacing(8),
    )
    .style(move |_t| container::Style {
      background: Some(
        theme::color::BG
          .with_alpha(if active { 1.0 } else { 0.0 })
          .as_color()
          .into(),
      ),
      border: Border {
        color: theme::color::BORDER
          .with_alpha(if active { theme::color::BORDER.a } else { 0.0 })
          .as_color(),
        radius: Radius {
          top_left: 8.0,
          top_right: 8.0,
          ..Default::default()
        },
        width: 1.0,
      },
      ..Default::default()
    })
    .center_y(30)
    .padding(Padding::from([0, 12])),
  )
  .padding(0)
  .style(move |_t, _s| button::Style {
    background: Some(theme::color::TRANSPARENT.as_color().into()),
    ..Default::default()
  })
  .on_press(Message::SwitchTab(index))
  .into()
}
