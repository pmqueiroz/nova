use chrono::Local;
use iced::{
  Border, Element, Length, Padding, Shadow,
  border::{Radius, radius},
  widget::{Space, button, container, row, space::horizontal},
};

use crate::ui::{
  app_state::Message,
  tab::Tab,
  theme,
  typography::{self, Typography},
};

pub fn status_bar<'a>(active_tab: &Tab) -> Element<'a, Message> {
  let local_now = Local::now();

  let resolved_branch = if let Some(b) = &active_tab.git_branch {
    b.clone()
  } else {
    "".to_string()
  };

  let mut content = row![].spacing(16);

  if !resolved_branch.is_empty() {
    content = content.push(status_bar_text(format!(" {}", resolved_branch), true));
  }

  content = content
    .push(status_bar_text(format!(" {}", active_tab.shell), false))
    .push(status_bar_text("utf-8", false))
    .push(horizontal())
    .push(status_bar_text(
      local_now.format("%b %d").to_string(),
      false,
    ))
    .push(status_bar_text(
      local_now.format("%H:%M:%S").to_string(),
      false,
    ));

  container(content)
    .style(move |_| container::Style {
      background: Some(theme::color::BG_DEEP.as_color().into()),
      border: Border {
        color: theme::color::BORDER.as_color(),
        radius: Radius {
          bottom_left: 12.0,
          bottom_right: 12.0,
          ..Default::default()
        },
        width: 0.5,
      },
      ..container::Style::default()
    })
    .center_y(22)
    .padding(Padding {
      top: 4.0,
      left: 8.0,
      right: 8.0,
      ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

pub fn status_bar_text(content: impl Into<String>, accent: bool) -> iced::widget::Text<'static> {
  Typography {
    weight: if accent {
      typography::Weight::Bold
    } else {
      typography::Weight::Normal
    },
    color: if accent {
      theme::color::ACCENT.as_color()
    } else {
      theme::color::FG_MUTED.as_color()
    },

    size: 10.into(),
    ..Default::default()
  }
  .as_text(content)
  .into()
}
