use std::sync::LazyLock;

use iced::widget::{Id, container, row, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Shadow, Vector};

use crate::ui::app_state::Message;
use crate::ui::components::ds::button::icon_button;
use crate::ui::icons::{Icon, icon};
use crate::ui::theme::font;
use crate::ui::theme::tokens::{self, height, radius, space};

pub static SEARCH_INPUT_ID: LazyLock<Id> = LazyLock::new(Id::unique);

pub const SEARCH_WIDTH: f32 = 384.0;

pub fn count_label(query: &str, index: usize, count: usize) -> String {
  match (query.is_empty(), count) {
    (true, _) => String::new(),
    (false, 0) => "No results".to_string(),
    _ => format!("{} / {}", index + 1, count),
  }
}

pub fn search_bar(
  query: &str,
  match_index: usize,
  match_count: usize,
) -> Element<'static, Message> {
  let no_match = !query.is_empty() && match_count == 0;
  let accent = crate::ui::theme::color::runtime().accent;
  let input = text_input("Find", query)
    .id(SEARCH_INPUT_ID.clone())
    .on_input(Message::SearchQueryChanged)
    .on_submit(Message::SearchNext)
    .size(tokens::text::T14)
    .font(font::regular())
    .padding(0)
    .width(Length::Fill)
    .style(move |_t, _s| text_input::Style {
      background: Background::Color(Color::TRANSPARENT),
      border: Border::default(),
      icon: tokens::FG_3,
      placeholder: tokens::FG_3,
      value: if no_match { tokens::RED } else { tokens::FG_1 },
      selection: tokens::with_alpha(accent, 0.32),
    });

  let bar = container(
    row![
      icon(Icon::Search, 16.0, tokens::FG_3),
      input,
      text(count_label(query, match_index, match_count))
        .size(tokens::text::T12)
        .color(tokens::FG_3),
      icon_button(
        Icon::ChevronUp,
        16.0,
        height::H32,
        Some(Message::SearchPrev)
      ),
      icon_button(
        Icon::ChevronDown,
        16.0,
        height::H32,
        Some(Message::SearchNext)
      ),
      icon_button(Icon::Close, 16.0, height::H32, Some(Message::SearchClose)),
    ]
    .spacing(space::S8)
    .align_y(Alignment::Center),
  )
  .width(SEARCH_WIDTH)
  .height(height::H48)
  .padding(Padding {
    top: 0.0,
    right: space::S8,
    bottom: 0.0,
    left: space::S16,
  })
  .center_y(height::H48)
  .style(|_| container::Style {
    background: Some(crate::ui::components::ds::sheet::window_gradient()),
    border: Border {
      radius: radius::R12.into(),
      width: 1.0,
      color: tokens::LINE_2,
    },
    shadow: Shadow {
      color: Color::from_rgba(0.0, 0.0, 0.0, 0.48),
      offset: Vector::new(0.0, space::S8),
      blur_radius: space::S24,
    },
    ..container::Style::default()
  });

  container(bar)
    .align_x(Alignment::End)
    .align_y(Alignment::Start)
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(Padding {
      top: space::S16,
      right: space::S32,
      bottom: 0.0,
      left: 0.0,
    })
    .into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn counts_read_naturally() {
    assert_eq!(count_label("", 0, 0), "");
    assert_eq!(count_label("x", 0, 0), "No results");
    assert_eq!(count_label("x", 1, 4), "2 / 4");
  }
}
