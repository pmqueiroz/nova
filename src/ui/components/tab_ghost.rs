use iced::widget::{Column, Space, column, container, row, text};
use iced::{
  Alignment, Background, Border, Color, Element, Length, Padding, Point, Shadow, Size, Vector,
  border,
};

use crate::ui::app_state::Message;
use crate::ui::theme::tokens::{self, radius, space};

pub const GHOST_SIZE: Size = Size::new(176.0, 112.0);
const GHOST_HEADER: f32 = 24.0;
const GHOST_BORDER: f32 = 1.0;
const SKELETON_BAR: f32 = 6.0;
const SKELETON_ROWS: [f32; 5] = [0.55, 0.85, 0.4, 0.7, 0.3];

pub fn ghost_origin(cursor: Point, window: Size) -> Point {
  let x =
    (cursor.x - GHOST_SIZE.width / 2.0).clamp(0.0, (window.width - GHOST_SIZE.width).max(0.0));
  let y = (cursor.y - space::S12).clamp(0.0, (window.height - GHOST_SIZE.height).max(0.0));
  Point::new(x, y)
}

fn bar<'a>(width: f32, color: Color) -> Element<'a, Message> {
  container(Space::new())
    .width(width)
    .height(SKELETON_BAR)
    .style(move |_| container::Style {
      background: Some(Background::Color(color)),
      border: Border {
        radius: (SKELETON_BAR / 2.0).into(),
        ..Border::default()
      },
      ..container::Style::default()
    })
    .into()
}

pub fn tab_ghost<'a>(
  label: String,
  cursor: Point,
  window: Size,
  armed: bool,
) -> Element<'a, Message> {
  let accent = crate::ui::theme::color::runtime().accent;
  let edge = if armed { accent } else { tokens::LINE_2 };
  let inner_width = GHOST_SIZE.width - 2.0 * space::S12;

  let header = container(
    row![
      bar(SKELETON_BAR, edge),
      text(label)
        .size(tokens::text::T12)
        .color(tokens::FG_2)
        .wrapping(text::Wrapping::None),
    ]
    .spacing(space::S8)
    .align_y(Alignment::Center),
  )
  .height(GHOST_HEADER)
  .width(Length::Fill)
  .padding(Padding::from([0.0, space::S8]))
  .center_y(GHOST_HEADER)
  .clip(true)
  .style(|_| container::Style {
    background: Some(Background::Color(tokens::SURFACE_2)),
    border: Border {
      radius: border::top(radius::R8 - GHOST_BORDER),
      ..Border::default()
    },
    ..container::Style::default()
  });

  let lines = SKELETON_ROWS
    .iter()
    .fold(Column::new().spacing(space::S8), |c, f| {
      c.push(bar(inner_width * f, tokens::SURFACE_3))
    });

  let card = container(column![header, container(lines).padding(space::S12)])
    .padding(GHOST_BORDER)
    .width(GHOST_SIZE.width)
    .height(GHOST_SIZE.height)
    .clip(true)
    .style(move |_| container::Style {
      background: Some(Background::Color(tokens::WIN_BOT.scale_alpha(0.92))),
      border: Border {
        color: edge,
        width: GHOST_BORDER,
        radius: radius::R8.into(),
      },
      shadow: Shadow {
        color: Color::BLACK.scale_alpha(0.4),
        offset: Vector::new(0.0, space::S8),
        blur_radius: space::S24,
      },
      ..container::Style::default()
    });

  let origin = ghost_origin(cursor, window);
  container(card)
    .padding(Padding {
      top: origin.y,
      left: origin.x,
      right: 0.0,
      bottom: 0.0,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ghost_stays_inside_window() {
    let window = Size::new(800.0, 500.0);
    assert_eq!(ghost_origin(Point::new(0.0, 0.0), window), Point::ORIGIN);
    let corner = ghost_origin(Point::new(800.0, 500.0), window);
    assert_eq!(
      corner,
      Point::new(800.0 - GHOST_SIZE.width, 500.0 - GHOST_SIZE.height)
    );
  }
}
