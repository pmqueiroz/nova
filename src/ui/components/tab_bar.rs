use iced::{
  Border, Element, Length, Padding, alignment,
  border::Radius,
  widget::{button, container, mouse_area, row, space::horizontal, text},
};

use crate::ui::{
  app_state::Message,
  helpers::{basename, til_home, truncate},
  tab::Tab,
  theme,
  typography::Typography,
};

pub fn tab_bar(tabs: &Vec<Tab>, active_index: usize) -> Element<'static, Message> {
  let mut tab_bar = row![];

  for (i, tab) in tabs.iter().enumerate() {
    let is_active = i == active_index;

    tab_bar = tab_bar.push(
      row![
        mouse_area(tab_item(
          truncate(&basename(&til_home(&tab.pwd)), 12),
          i,
          is_active
        ))
        .on_middle_press(Message::CloseTab(i))
      ]
      .spacing(2),
    );
  }

  tab_bar = tab_bar.push(
    container(
      button("+")
        .style(move |_t, status| button::Style {
          text_color: match status {
            button::Status::Hovered => theme::color::runtime().accent,
            button::Status::Pressed => theme::color::ACCENT_DIM.as_color(),
            _ => theme::color::runtime().foreground_muted,
          },
          background: Some(theme::color::TRANSPARENT.as_color().into()),
          ..Default::default()
        })
        .on_press(Message::OpenShellPicker)
        .padding(4),
    )
    .padding(Padding::from([0, 8])),
  );

  container(tab_bar)
    .padding(Padding::from([4, 8]))
    .width(Length::Fill)
    .style(move |_| container::Style {
      background: Some(theme::color::BG_DEEP.as_color().into()),
      border: Border {
        color: theme::color::runtime().border,
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
            color: if active {
              theme::color::runtime().foreground
            } else {
              theme::color::runtime().foreground_muted
            },
            size: 11.into(),
            ..Default::default()
          }
          .as_text(title),
        )
        .style(move |_t, _s| button::Style {
          background: Some(theme::color::TRANSPARENT.as_color().into()),
          ..Default::default()
        })
        .padding(Padding::from([4, 0]))
        .on_press(Message::SwitchTab(index)),
        horizontal(),
        button(text("󰅖").size(11))
          .style(move |_t, status| button::Style {
            text_color: if status == button::Status::Hovered {
              theme::color::RED.as_color()
            } else {
              theme::color::runtime().foreground_muted
            },
            background: Some(theme::color::TRANSPARENT.as_color().into()),
            ..Default::default()
          })
          .on_press(Message::CloseTab(index))
          .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 8.0,
            right: 2.0,
          }),
      ]
      .spacing(0)
      .align_y(alignment::Vertical::Center),
    )
    .style(move |_t| container::Style {
      background: Some(
        if active {
          theme::color::BG_HIGH
        } else {
          theme::color::TRANSPARENT
        }
        .as_color()
        .into(),
      ),
      border: Border {
        color: {
          let b = theme::color::runtime().border;
          iced::Color {
            a: if active { 0.15 } else { 0.0 },
            ..b
          }
        },
        radius: Radius::new(4.0),
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
  .width(120)
  .on_press(Message::SwitchTab(index))
  .into()
}
