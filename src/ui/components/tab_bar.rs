use std::sync::LazyLock;

use iced::{
  Border, Element, Length, Padding, alignment,
  border::Radius,
  widget::{Id, button, container, mouse_area, row, space::horizontal, text, text_input},
};

use crate::ui::{
  app_state::Message,
  helpers::{basename, til_home, truncate},
  tab::Tab,
  theme,
  typography::Typography,
};

pub static TAB_TITLE_INPUT_ID: LazyLock<Id> = LazyLock::new(Id::unique);

pub fn tab_bar(
  tabs: &[Tab],
  active_index: usize,
  editing_tab_index: Option<usize>,
  editing_tab_title: &str,
) -> Element<'static, Message> {
  let mut tab_bar = row![];

  for (i, tab) in tabs.iter().enumerate() {
    let is_active = i == active_index;
    let label = tab
      .title_override
      .as_deref()
      .map(|t| truncate(t, 12))
      .unwrap_or_else(|| truncate(&basename(&til_home(&tab.pwd)), 12));

    tab_bar = tab_bar.push(
      row![
        mouse_area(tab_item(
          label,
          i,
          is_active,
          editing_tab_index == Some(i),
          editing_tab_title.to_string(),
        ))
        .on_press(Message::SwitchTab(i))
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

fn tab_inner_style(active: bool) -> container::Style {
  container::Style {
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
  }
}

fn tab_item(
  title: String,
  index: usize,
  active: bool,
  editing: bool,
  editing_value: String,
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

  if editing {
    container(
      row![
        text_input("", &editing_value)
          .id(TAB_TITLE_INPUT_ID.clone())
          .on_input(Message::TabTitleInput)
          .on_submit(Message::TabTitleCommit)
          .size(11)
          .style(move |_t, _s| text_input::Style {
            background: theme::color::TRANSPARENT.as_color().into(),
            border: Border::default(),
            icon: theme::color::runtime().foreground_muted,
            placeholder: theme::color::runtime().foreground_muted,
            value: theme::color::runtime().foreground,
            selection: theme::color::runtime().accent,
          })
          .padding(Padding::from([4, 0])),
        horizontal(),
        close_btn,
      ]
      .spacing(0)
      .align_y(alignment::Vertical::Center),
    )
    .style(move |_| tab_inner_style(active))
    .center_y(30)
    .padding(Padding::from([0, 12]))
    .width(120)
    .into()
  } else {
    let title_btn = button(
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
    .on_press(if active {
      Message::TabTitleEdit(index)
    } else {
      Message::SwitchTab(index)
    });

    container(
      row![title_btn, horizontal(), close_btn,]
        .spacing(0)
        .align_y(alignment::Vertical::Center),
    )
    .style(move |_| tab_inner_style(active))
    .center_y(30)
    .padding(Padding::from([0, 12]))
    .width(120)
    .into()
  }
}
