use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, mouse};
use iced::animation::Animation;
use iced::time::Instant;
use iced::{
  Background, Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Vector, window,
};

use super::transform::press;
use crate::ui::theme::motion;
use crate::ui::theme::tokens::{self, duration};

pub const WIDTH: f32 = 48.0;
pub const HEIGHT: f32 = 28.0;
pub const KNOB: f32 = 24.0;
pub const INSET: f32 = 2.0;
pub const TRAVEL: f32 = 20.0;

pub struct Toggle<Message> {
  on: bool,
  on_toggle: Box<dyn Fn(bool) -> Message>,
}

struct State {
  knob: Animation<bool>,
  track: Animation<bool>,
  now: Instant,
  last: Option<bool>,
}

pub fn knob_x(t: f32) -> f32 {
  INSET + TRAVEL * t
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Toggle<Message> {
  fn tag(&self) -> tree::Tag {
    tree::Tag::of::<State>()
  }

  fn state(&self) -> tree::State {
    tree::State::new(State {
      knob: motion::anim(self.on, duration::D2, motion::SPRING),
      track: motion::anim(self.on, duration::D2, motion::EASE),
      now: Instant::now(),
      last: Some(self.on),
    })
  }

  fn size(&self) -> Size<Length> {
    Size::new(Length::Fixed(WIDTH), Length::Fixed(HEIGHT))
  }

  fn layout(
    &mut self,
    _tree: &mut Tree,
    _renderer: &iced::Renderer,
    _limits: &layout::Limits,
  ) -> layout::Node {
    layout::Node::new(Size::new(WIDTH, HEIGHT))
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
    if state.last != Some(self.on) {
      state.last = Some(self.on);
      state.knob.go_mut(self.on, state.now);
      state.track.go_mut(self.on, state.now);
      shell.request_redraw();
    }
    match event {
      Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        if cursor.is_over(layout.bounds()) =>
      {
        shell.publish((self.on_toggle)(!self.on));
        shell.capture_event();
      }
      Event::Window(window::Event::RedrawRequested(now)) => {
        state.now = *now;
        if state.knob.is_animating(*now) || state.track.is_animating(*now) {
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
    let accent = crate::ui::theme::color::runtime().accent;
    let tt = state.track.interpolate(0.0, 1.0, state.now);
    let kt = state.knob.interpolate(0.0, 1.0, state.now);
    renderer.fill_quad(
      Quad {
        bounds: b,
        border: Border {
          radius: (HEIGHT / 2.0).into(),
          ..Border::default()
        },
        ..Quad::default()
      },
      Background::Color(tokens::mix(tokens::SURFACE_3, accent, tt)),
    );
    let knob = Rectangle::new(
      Point::new(b.x + knob_x(kt), b.y + INSET),
      Size::new(KNOB, KNOB),
    );
    renderer.fill_quad(
      Quad {
        bounds: knob,
        border: Border {
          radius: (KNOB / 2.0).into(),
          ..Border::default()
        },
        shadow: Shadow {
          color: Color::from_rgba(0.0, 0.0, 0.0, 0.24),
          offset: Vector::new(0.0, 2.0),
          blur_radius: 4.0,
        },
        ..Quad::default()
      },
      Background::Color(tokens::mix(tokens::FG_1, tokens::ON_ACCENT, tt)),
    );
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

pub fn toggle<'a, Message: 'a>(
  on: bool,
  on_toggle: impl Fn(bool) -> Message + 'static,
) -> Element<'a, Message> {
  press(Element::new(Toggle {
    on,
    on_toggle: Box::new(on_toggle),
  }))
  .into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn knob_travels_twenty() {
    assert_eq!(knob_x(0.0), 2.0);
    assert_eq!(knob_x(1.0), 22.0);
    assert_eq!(knob_x(1.0) + KNOB + INSET, WIDTH);
  }
}
