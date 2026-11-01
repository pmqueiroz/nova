use std::f32::consts::PI;

use iced::gradient::Linear;
use iced::widget::{Column, Row, column, container, mouse_area, opaque, stack};
use iced::{
  Alignment, Background, Border, Color, Element, Gradient, Length, Padding, Shadow, Vector,
};

use super::transform::Transform;
use crate::ui::theme::motion::lerp;
use crate::ui::theme::tokens::{self, height, radius, space};

pub const WIDTH: f32 = 576.0;
pub const SETTINGS_WIDTH: f32 = 640.0;
pub const TOP_OFFSET_MAX: f32 = 128.0;
pub const TOP_OFFSET_VH: f32 = 0.16;

pub fn window_gradient() -> Background {
  Background::Gradient(Gradient::Linear(
    Linear::new(PI)
      .add_stop(0.0, tokens::WIN_TOP)
      .add_stop(1.0, tokens::WIN_BOT),
  ))
}

pub fn sheet_style(t: f32) -> container::Style {
  container::Style {
    background: Some(window_gradient()),
    border: Border {
      radius: radius::R16.into(),
      width: 1.0,
      color: tokens::LINE_2,
    },
    shadow: Shadow {
      color: Color::from_rgba(0.0, 0.0, 0.0, 0.48 * t),
      offset: Vector::new(0.0, space::S32),
      blur_radius: space::S64,
    },
    ..container::Style::default()
  }
}

pub fn top_offset(window_height: f32) -> f32 {
  (window_height * TOP_OFFSET_VH).min(TOP_OFFSET_MAX)
}

pub fn max_height(window_height: f32) -> f32 {
  window_height - 2.0 * top_offset(window_height)
}

pub fn pop_offset(t: f32) -> f32 {
  lerp(space::S8, 0.0, t).round()
}

pub fn head<'a, Message: 'a>(content: Row<'a, Message>) -> Element<'a, Message> {
  column![
    container(content.spacing(space::S12).align_y(Alignment::Center))
      .height(height::H64)
      .width(Length::Fill)
      .padding(Padding {
        top: 0.0,
        right: space::S16,
        bottom: 0.0,
        left: space::S24,
      })
      .center_y(height::H64),
    line(),
  ]
  .into()
}

pub fn foot<'a, Message: 'a>(content: Row<'a, Message>) -> Element<'a, Message> {
  column![
    line(),
    container(content.spacing(space::S16).align_y(Alignment::Center))
      .width(Length::Fill)
      .padding(Padding {
        top: space::S8,
        right: space::S16,
        bottom: space::S8,
        left: space::S24,
      })
      .center_y(height::H56),
  ]
  .into()
}

pub fn line<'a, Message: 'a>() -> Element<'a, Message> {
  container(iced::widget::Space::new())
    .width(Length::Fill)
    .height(1)
    .style(|_| container::Style {
      background: Some(Background::Color(tokens::LINE)),
      ..container::Style::default()
    })
    .into()
}

pub struct Sheet<'a, Message> {
  pub width: f32,
  pub head: Option<Element<'a, Message>>,
  pub body: Element<'a, Message>,
  pub foot: Option<Element<'a, Message>>,
}

pub fn sheet<'a, Message: Clone + 'a>(
  s: Sheet<'a, Message>,
  progress: f32,
  window_height: f32,
  on_close: Message,
) -> Element<'a, Message> {
  let t = progress.clamp(0.0, 1.2);
  let fade = progress.clamp(0.0, 1.0);
  let mut col = Column::new();
  if let Some(h) = s.head {
    col = col.push(h);
  }
  col = col.push(s.body);
  if let Some(f) = s.foot {
    col = col.push(f);
  }
  let panel = container(col)
    .width(Length::Fixed(s.width))
    .max_width(s.width)
    .clip(true)
    .style(move |_| sheet_style(fade));
  let popped = Transform::new(opaque(panel)).offset(0.0, pop_offset(t));
  let scrim = mouse_area(
    container(iced::widget::Space::new())
      .width(Length::Fill)
      .height(Length::Fill)
      .style(move |_| container::Style {
        background: Some(Background::Color(tokens::with_alpha(tokens::SCRIM, fade))),
        ..container::Style::default()
      }),
  )
  .on_press(on_close);
  let gap = top_offset(window_height);
  let placed = container(popped)
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .align_y(Alignment::Start)
    .padding(Padding {
      top: gap,
      right: space::S16,
      bottom: gap,
      left: space::S16,
    });
  stack![scrim, placed].into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn top_offset_caps_at_128() {
    assert_eq!(top_offset(400.0), 64.0);
    assert_eq!(top_offset(2000.0), 128.0);
  }

  #[test]
  fn max_height_leaves_top_gap_at_bottom() {
    assert_eq!(max_height(400.0), 272.0);
    assert_eq!(max_height(2000.0), 1744.0);
  }

  #[test]
  fn pop_offset_snaps_to_whole_pixels() {
    assert_eq!(pop_offset(0.0), 8.0);
    assert_eq!(pop_offset(1.0), 0.0);
    assert_eq!(pop_offset(0.53).fract(), 0.0);
  }

  #[test]
  fn sheet_shadow_fades_in() {
    assert_eq!(sheet_style(0.0).shadow.color.a, 0.0);
    assert!((sheet_style(1.0).shadow.color.a - 0.48).abs() < 1e-6);
  }
}
