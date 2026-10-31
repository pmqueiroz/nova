use iced::{Border, Element, Length, Shadow, widget::container};

use crate::ui::app_state::Message;
use crate::ui::components::ds::sheet::window_gradient;
use crate::ui::components::ds::transform::Transform;
use crate::ui::theme::motion::lerp;
use crate::ui::theme::tokens::{self, radius, space};

pub fn window_radius(maximized: bool) -> f32 {
  if cfg!(target_os = "windows") || maximized {
    0.0
  } else {
    radius::R16
  }
}

pub fn window_style(maximized: bool) -> container::Style {
  let r = window_radius(maximized);
  container::Style {
    background: Some(window_gradient()),
    border: Border {
      color: if r > 0.0 {
        tokens::LINE
      } else {
        iced::Color::TRANSPARENT
      },
      radius: r.into(),
      width: if r > 0.0 { 1.0 } else { 0.0 },
    },
    shadow: Shadow::default(),
    ..container::Style::default()
  }
}

pub fn app<'a>(
  content: impl Into<Element<'a, Message>>,
  maximized: bool,
  pop: f32,
) -> Element<'a, Message> {
  let surface = container(content)
    .style(move |_| window_style(maximized))
    .width(Length::Fill)
    .height(Length::Fill)
    .clip(true);
  Transform::new(surface)
    .scale(lerp(0.96, 1.0, pop))
    .offset(0.0, lerp(space::S8, 0.0, pop))
    .into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn maximized_window_is_square() {
    assert_eq!(window_radius(true), 0.0);
  }

  #[cfg(not(target_os = "windows"))]
  #[test]
  fn floating_window_is_r16() {
    assert_eq!(window_radius(false), 16.0);
  }
}
