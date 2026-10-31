use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, mouse};
use iced::animation::Animation;
use iced::time::Instant;
use iced::{
  Background, Border, Color, Element, Event, Length, Point, Rectangle, Size, Transformation, window,
};

use super::transform::press;
use crate::ui::theme::motion;
use crate::ui::theme::tokens::{self, duration, height, space};

pub const SIZE: f32 = height::H32;
pub const HOVER_SCALE: f32 = 1.08;

pub struct Swatch<Message> {
  color: Color,
  on: bool,
  on_press: Message,
}

struct State {
  hover: Animation<bool>,
  hovered: bool,
  now: Instant,
}

pub fn rings(on: bool) -> Vec<(f32, Color)> {
  if on {
    vec![
      (SIZE / 2.0 + space::S4, tokens::FG_1),
      (SIZE / 2.0 + space::S2, tokens::WIN_BOT),
    ]
  } else {
    vec![]
  }
}

fn circle(center: Point, r: f32) -> Rectangle {
  Rectangle::new(
    Point::new(center.x - r, center.y - r),
    Size::new(r * 2.0, r * 2.0),
  )
}

impl<Message: Clone> Widget<Message, iced::Theme, iced::Renderer> for Swatch<Message> {
  fn tag(&self) -> tree::Tag {
    tree::Tag::of::<State>()
  }

  fn state(&self) -> tree::State {
    tree::State::new(State {
      hover: motion::anim(false, duration::D1, motion::SPRING),
      hovered: false,
      now: Instant::now(),
    })
  }

  fn size(&self) -> Size<Length> {
    Size::new(Length::Fixed(SIZE), Length::Fixed(SIZE))
  }

  fn layout(
    &mut self,
    _tree: &mut Tree,
    _renderer: &iced::Renderer,
    _limits: &layout::Limits,
  ) -> layout::Node {
    layout::Node::new(Size::new(SIZE, SIZE))
  }

  fn update(
    &mut self,
    tree: &mut Tree,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    _renderer: &iced::Renderer,
    _clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    _viewport: &Rectangle,
  ) {
    let state = tree.state.downcast_mut::<State>();
    match event {
      Event::Mouse(mouse::Event::CursorMoved { .. }) => {
        let over = cursor.is_over(layout.bounds());
        if over != state.hovered {
          state.hovered = over;
          state.hover.go_mut(over, state.now);
          shell.request_redraw();
        }
      }
      Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        if cursor.is_over(layout.bounds()) =>
      {
        shell.publish(self.on_press.clone());
        shell.capture_event();
      }
      Event::Window(window::Event::RedrawRequested(now)) => {
        state.now = *now;
        if state.hover.is_animating(*now) {
          shell.request_redraw();
        }
      }
      _ => {}
    }
  }

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut iced::Renderer,
    _theme: &iced::Theme,
    _style: &renderer::Style,
    layout: Layout<'_>,
    _cursor: mouse::Cursor,
    _viewport: &Rectangle,
  ) {
    use iced::advanced::Renderer as _;
    let state = tree.state.downcast_ref::<State>();
    let b = layout.bounds();
    let c = b.center();
    let s = state.hover.interpolate(1.0, HOVER_SCALE, state.now);
    let t = Transformation::translate(c.x, c.y)
      * Transformation::scale(s)
      * Transformation::translate(-c.x, -c.y);
    let fill = |r: &mut iced::Renderer, radius: f32, color: Color| {
      r.fill_quad(
        Quad {
          bounds: circle(c, radius),
          border: Border {
            radius: radius.into(),
            ..Border::default()
          },
          ..Quad::default()
        },
        Background::Color(color),
      );
    };
    renderer.with_transformation(t, |r| {
      for (radius, color) in rings(self.on) {
        fill(r, radius, color);
      }
      fill(r, SIZE / 2.0, self.color);
      r.fill_quad(
        Quad {
          bounds: circle(c, SIZE / 2.0),
          border: Border {
            radius: (SIZE / 2.0).into(),
            width: space::S2,
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.16),
          },
          ..Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
      );
    });
  }

  fn mouse_interaction(
    &self,
    _tree: &Tree,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    _viewport: &Rectangle,
    _renderer: &iced::Renderer,
  ) -> mouse::Interaction {
    if cursor.is_over(layout.bounds()) {
      mouse::Interaction::Pointer
    } else {
      mouse::Interaction::default()
    }
  }
}

pub fn swatch<'a, Message: Clone + 'a>(
  color: Color,
  on: bool,
  on_press: Message,
) -> Element<'a, Message> {
  press(Element::new(Swatch {
    color,
    on,
    on_press,
  }))
  .into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn selected_rings_are_two_and_four_px_outside() {
    let r = rings(true);
    assert_eq!(r[0].0, SIZE / 2.0 + 4.0);
    assert_eq!(r[1].0, SIZE / 2.0 + 2.0);
    assert!(rings(false).is_empty());
  }
}
