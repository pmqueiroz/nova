use iced::widget::{Row, button, container, text};
use iced::{Background, Border, Color, Element, Length, Padding};

use super::transform::press;
use crate::ui::theme::tokens::{self, height, radius, space};

pub fn seg_button_style(on: bool, status: button::Status) -> button::Style {
  let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
  button::Style {
    background: Some(Background::Color(if on {
      tokens::SURFACE_3
    } else {
      Color::TRANSPARENT
    })),
    text_color: if on || hovered {
      tokens::FG_1
    } else {
      tokens::FG_3
    },
    border: Border {
      radius: radius::R8.into(),
      ..Border::default()
    },
    ..button::Style::default()
  }
}

pub fn track_style() -> container::Style {
  container::Style {
    background: Some(Background::Color(tokens::SURFACE_1)),
    border: Border {
      radius: radius::R12.into(),
      ..Border::default()
    },
    ..container::Style::default()
  }
}

pub fn seg<'a, T, Message>(
  value: &T,
  options: &[(T, &'a str)],
  on_change: impl Fn(T) -> Message,
) -> Element<'a, Message>
where
  T: PartialEq + Clone + 'a,
  Message: Clone + 'a,
{
  let row = options
    .iter()
    .fold(Row::new().spacing(space::S4), |r, (v, label)| {
      let on = v == value;
      r.push(press(
        button(container(text(*label).size(tokens::text::T14)).center_y(Length::Fill))
          .height(height::H32)
          .padding(Padding::from([0.0, space::S16]))
          .style(move |_t, s| seg_button_style(on, s))
          .on_press(on_change(v.clone())),
      ))
    });
  container(row)
    .padding(space::S4)
    .style(|_| track_style())
    .into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn selected_segment_is_raised() {
    let s = seg_button_style(true, button::Status::Active);
    assert_eq!(s.background, Some(Background::Color(tokens::SURFACE_3)));
    assert_eq!(s.text_color, tokens::FG_1);
  }

  #[test]
  fn idle_segment_is_dim_until_hover() {
    assert_eq!(
      seg_button_style(false, button::Status::Active).text_color,
      tokens::FG_3
    );
    assert_eq!(
      seg_button_style(false, button::Status::Hovered).text_color,
      tokens::FG_1
    );
  }
}
