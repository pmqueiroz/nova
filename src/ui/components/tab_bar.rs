use iced::{
  Border, Element, Length, Padding, alignment,
  border::Radius,
  mouse,
  widget::{button, container, mouse_area, row, space::horizontal, text},
};

use crate::ui::{
  app_state::Message,
  helpers::{basename, til_home, truncate},
  tab::Tab,
  theme,
  typography::Typography,
};

pub fn tab_bar(
  tabs: &[Tab],
  active_index: usize,
  dragging_tab_index: Option<usize>,
) -> Element<'static, Message> {
  let mut tab_bar = row![];

  for (i, tab) in tabs.iter().enumerate() {
    let is_active = i == active_index;
    let is_dragging = dragging_tab_index == Some(i);

    let item = mouse_area(tab_item(
      truncate(&basename(&til_home(&tab.pwd)), 12),
      i,
      is_active,
      tab.command_done,
      is_dragging,
    ))
    .on_middle_press(Message::CloseTab(i))
    .interaction(if is_dragging {
      mouse::Interaction::Grabbing
    } else {
      mouse::Interaction::default()
    });

    tab_bar = tab_bar.push(row![item].spacing(2));
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

fn tab_item(
  title: String,
  index: usize,
  active: bool,
  command_done: bool,
  dragging: bool,
) -> Element<'static, Message> {
  let close_btn = button(text("󰅖").size(11))
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
    });

  let mut content = row![
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
    horizontal(),
  ]
  .spacing(0)
  .align_y(alignment::Vertical::Center);

  if command_done {
    content = content.push(
      container(
        Typography {
          color: theme::color::runtime().accent,
          size: 8.into(),
          ..Default::default()
        }
        .as_text("●".to_string()),
      )
      .padding(Padding::from([0, 4])),
    );
  }

  content = content.push(close_btn);

  container(content)
    .style(move |_| container::Style {
      background: Some(
        if dragging {
          theme::color::runtime().accent
        } else if active {
          theme::color::BG_HIGH.as_color()
        } else {
          theme::color::TRANSPARENT.as_color()
        }
        .into(),
      ),
      border: Border {
        color: {
          let b = theme::color::runtime().border;
          iced::Color {
            a: if active || dragging { 0.15 } else { 0.0 },
            ..b
          }
        },
        radius: Radius::new(4.0),
        width: 1.0,
      },
      ..Default::default()
    })
    .center_y(30)
    .padding(Padding::from([0, 12]))
    .width(120)
    .into()
}
