use chrono::Local;
use iced::{
  Border, Element, Length, Padding,
  border::Radius,
  widget::{container, row, space::horizontal},
};
use std::time::{Duration, Instant};

use crate::ui::{
  app_state::Message,
  theme,
  typography::{self, Typography},
};

pub fn status_bar<'a>(
  shell: &str,
  git_branch: Option<&str>,
  command_start: Option<Instant>,
  last_command_elapsed: Option<Duration>,
  date_format: &str,
  time_format: &str,
  maximized: bool,
) -> Element<'a, Message> {
  let local_now = Local::now();

  let mut content = row![].spacing(16);

  if let Some(branch) = git_branch.filter(|b| !b.is_empty()) {
    content = content.push(status_bar_text(format!(" {}", branch), true));
  }

  content = content
    .push(status_bar_text(shell, false))
    .push(status_bar_text("utf-8", false));

  if let Some(start) = command_start {
    let elapsed = start.elapsed();
    content = content.push(status_bar_text(format_elapsed(elapsed), false));
  } else if let Some(elapsed) = last_command_elapsed {
    content = content.push(status_bar_text(format_elapsed(elapsed), false));
  }

  content = content
    .push(horizontal())
    .push(status_bar_text(
      local_now.format(date_format).to_string(),
      false,
    ))
    .push(status_bar_text(
      local_now.format(time_format).to_string(),
      false,
    ));

  let corner_radius = if maximized {
    Radius::default()
  } else {
    Radius {
      bottom_left: 12.0,
      bottom_right: 12.0,
      ..Default::default()
    }
  };

  container(content)
    .style(move |_| container::Style {
      background: Some(theme::color::BG_DEEP.as_color().into()),
      border: Border {
        color: theme::color::runtime().border,
        radius: corner_radius,
        width: 0.5,
      },
      ..container::Style::default()
    })
    .center_y(22)
    .padding(Padding {
      left: 8.0,
      right: 8.0,
      ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

fn format_elapsed(d: Duration) -> String {
  let secs = d.as_secs();
  let ms = d.subsec_millis();
  if secs == 0 {
    format!("{}ms", ms)
  } else if secs < 60 {
    format!("{}s", secs)
  } else if secs < 3600 {
    format!("{}m {}s", secs / 60, secs % 60)
  } else {
    format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
  }
}

pub fn status_bar_text(content: impl Into<String>, accent: bool) -> iced::widget::Text<'static> {
  Typography {
    weight: if accent {
      typography::Weight::Bold
    } else {
      typography::Weight::Normal
    },
    color: if accent {
      theme::color::runtime().accent
    } else {
      theme::color::runtime().foreground_muted
    },

    size: 10.into(),
  }
  .as_text(content)
}
