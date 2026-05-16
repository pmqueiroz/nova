use std::sync::LazyLock;

use iced::{
  Border, Color, Element, Length, Padding,
  alignment::{Horizontal, Vertical},
  border::Radius,
  widget::{Id, button, container, row, text, text_input},
};

use crate::ui::{app_state::Message, theme};

pub static SEARCH_INPUT_ID: LazyLock<Id> = LazyLock::new(Id::unique);

pub fn search_bar(
  query: &str,
  match_index: usize,
  match_count: usize,
) -> Element<'static, Message> {
  let rt = theme::color::runtime();
  let bg = rt.background;
  let fg_muted = rt.foreground_muted;
  let accent = rt.accent;
  drop(rt);

  let count_str = if match_count == 0 {
    if query.is_empty() {
      String::new()
    } else {
      "No results".to_string()
    }
  } else {
    format!("{} / {}", match_index + 1, match_count)
  };

  let no_match = !query.is_empty() && match_count == 0;
  let input_text_color = if no_match {
    theme::color::RED.as_color()
  } else {
    theme::color::runtime().foreground
  };

  let bar = container(
    row![
      text_input("Search...", query)
        .id(SEARCH_INPUT_ID.clone())
        .on_input(Message::SearchQueryChanged)
        .on_submit(Message::SearchNext)
        .size(12)
        .padding(Padding::from([3, 6]))
        .width(180)
        .style(move |_t, _status| text_input::Style {
          background: iced::Background::Color(Color::TRANSPARENT),
          border: Border {
            color: Color::TRANSPARENT,
            radius: Radius::new(0.0),
            width: 0.0,
          },
          icon: Color::TRANSPARENT,
          placeholder: Color { a: 0.3, ..fg_muted },
          value: input_text_color,
          selection: theme::color::runtime().accent,
        }),
      container(text(count_str).size(11).color(fg_muted))
        .width(60)
        .align_x(Horizontal::Center),
      button(text("↑").size(11).color(fg_muted))
        .on_press(Message::SearchPrev)
        .padding(Padding::from([2, 5]))
        .style(|_t, _s| button::Style {
          background: None,
          ..Default::default()
        }),
      button(text("↓").size(11).color(fg_muted))
        .on_press(Message::SearchNext)
        .padding(Padding::from([2, 5]))
        .style(|_t, _s| button::Style {
          background: None,
          ..Default::default()
        }),
      button(text("✕").size(10).color(fg_muted))
        .on_press(Message::SearchClose)
        .padding(Padding::from([2, 5]))
        .style(move |_t, status| button::Style {
          text_color: match status {
            button::Status::Hovered | button::Status::Pressed => theme::color::RED.as_color(),
            _ => fg_muted,
          },
          background: None,
          ..Default::default()
        }),
    ]
    .spacing(4)
    .align_y(Vertical::Center),
  )
  .style(move |_| container::Style {
    background: Some(bg.into()),
    border: Border {
      color: accent,
      radius: Radius::new(6.0),
      width: 1.0,
    },
    ..Default::default()
  })
  .padding(Padding::from([4, 6]));

  container(bar)
    .align_x(Horizontal::Right)
    .align_y(Vertical::Top)
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(Padding {
      top: 8.0,
      right: 16.0,
      bottom: 0.0,
      left: 0.0,
    })
    .into()
}
