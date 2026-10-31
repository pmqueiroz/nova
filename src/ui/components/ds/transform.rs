use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, mouse, overlay};
use iced::animation::Animation;
use iced::time::Instant;
use iced::{Element, Event, Length, Rectangle, Size, Transformation, Vector, window};

use crate::ui::theme::motion;

pub const PRESS_SCALE: f32 = 0.96;

pub struct Transform<'a, Message, Theme, Renderer> {
  content: Element<'a, Message, Theme, Renderer>,
  scale: f32,
  offset: Vector,
  press: Option<f32>,
}

struct State {
  pressed: bool,
  anim: Animation<bool>,
  now: Instant,
}

impl Default for State {
  fn default() -> Self {
    Self {
      pressed: false,
      anim: motion::fast(false),
      now: Instant::now(),
    }
  }
}

impl<'a, Message, Theme, Renderer> Transform<'a, Message, Theme, Renderer> {
  pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
    Self {
      content: content.into(),
      scale: 1.0,
      offset: Vector::ZERO,
      press: None,
    }
  }

  pub fn scale(mut self, scale: f32) -> Self {
    self.scale = scale;
    self
  }

  pub fn offset(mut self, x: f32, y: f32) -> Self {
    self.offset = Vector::new(x, y);
    self
  }

  pub fn press(mut self) -> Self {
    self.press = Some(PRESS_SCALE);
    self
  }

  pub fn press_to(mut self, scale: f32) -> Self {
    self.press = Some(scale);
    self
  }
}

pub fn press<'a, Message, Theme, Renderer>(
  content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Transform<'a, Message, Theme, Renderer> {
  Transform::new(content).press()
}

fn transformation(bounds: Rectangle, scale: f32, offset: Vector) -> Transformation {
  let c = bounds.center();
  Transformation::translate(c.x + offset.x, c.y + offset.y)
    * Transformation::scale(scale)
    * Transformation::translate(-c.x, -c.y)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
  for Transform<'_, Message, Theme, Renderer>
where
  Renderer: renderer::Renderer,
{
  fn tag(&self) -> tree::Tag {
    tree::Tag::of::<State>()
  }

  fn state(&self) -> tree::State {
    tree::State::new(State::default())
  }

  fn children(&self) -> Vec<Tree> {
    vec![Tree::new(&self.content)]
  }

  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(std::slice::from_ref(&self.content));
  }

  fn size(&self) -> Size<Length> {
    self.content.as_widget().size()
  }

  fn size_hint(&self) -> Size<Length> {
    self.content.as_widget().size_hint()
  }

  fn layout(
    &mut self,
    tree: &mut Tree,
    renderer: &Renderer,
    limits: &layout::Limits,
  ) -> layout::Node {
    self
      .content
      .as_widget_mut()
      .layout(&mut tree.children[0], renderer, limits)
  }

  fn update(
    &mut self,
    tree: &mut Tree,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
  ) {
    if self.press.is_some() {
      let state = tree.state.downcast_mut::<State>();
      match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
          if cursor.is_over(layout.bounds()) =>
        {
          state.pressed = true;
          state.anim.go_mut(true, state.now);
          shell.request_redraw();
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.pressed => {
          state.pressed = false;
          state.anim.go_mut(false, state.now);
          shell.request_redraw();
        }
        Event::Window(window::Event::RedrawRequested(now)) => {
          state.now = *now;
          if state.anim.is_animating(*now) {
            shell.request_redraw();
          }
        }
        _ => {}
      }
    }

    self.content.as_widget_mut().update(
      &mut tree.children[0],
      event,
      layout,
      cursor,
      renderer,
      clipboard,
      shell,
      viewport,
    );
  }

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut Renderer,
    theme: &Theme,
    style: &renderer::Style,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
  ) {
    let mut scale = self.scale;
    if let Some(target) = self.press {
      let state = tree.state.downcast_ref::<State>();
      scale *= state.anim.interpolate(1.0, target, state.now);
    }

    if (scale - 1.0).abs() < f32::EPSILON && self.offset == Vector::ZERO {
      self.content.as_widget().draw(
        &tree.children[0],
        renderer,
        theme,
        style,
        layout,
        cursor,
        viewport,
      );
      return;
    }

    renderer.with_transformation(transformation(layout.bounds(), scale, self.offset), |r| {
      self
        .content
        .as_widget()
        .draw(&tree.children[0], r, theme, style, layout, cursor, viewport);
    });
  }

  fn mouse_interaction(
    &self,
    tree: &Tree,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
    renderer: &Renderer,
  ) -> mouse::Interaction {
    self.content.as_widget().mouse_interaction(
      &tree.children[0],
      layout,
      cursor,
      viewport,
      renderer,
    )
  }

  fn operate(
    &mut self,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    operation: &mut dyn widget::Operation,
  ) {
    self
      .content
      .as_widget_mut()
      .operate(&mut tree.children[0], layout, renderer, operation);
  }

  fn overlay<'b>(
    &'b mut self,
    tree: &'b mut Tree,
    layout: Layout<'b>,
    renderer: &Renderer,
    viewport: &Rectangle,
    translation: Vector,
  ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
    self.content.as_widget_mut().overlay(
      &mut tree.children[0],
      layout,
      renderer,
      viewport,
      translation,
    )
  }
}

impl<'a, Message, Theme, Renderer> From<Transform<'a, Message, Theme, Renderer>>
  for Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Theme: 'a,
  Renderer: renderer::Renderer + 'a,
{
  fn from(t: Transform<'a, Message, Theme, Renderer>) -> Self {
    Element::new(t)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use iced::Point;

  #[test]
  fn scaling_keeps_center_fixed() {
    let b = Rectangle::new(Point::new(10.0, 20.0), Size::new(100.0, 40.0));
    let t = transformation(b, 0.96, Vector::ZERO);
    let c = b.center() * t;
    assert!((c.x - b.center().x).abs() < 1e-3);
    assert!((c.y - b.center().y).abs() < 1e-3);
  }

  #[test]
  fn offset_translates() {
    let b = Rectangle::new(Point::ORIGIN, Size::new(10.0, 10.0));
    let p = Point::ORIGIN * transformation(b, 1.0, Vector::new(0.0, 8.0));
    assert!((p.y - 8.0).abs() < 1e-3);
  }
}
