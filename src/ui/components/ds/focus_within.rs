use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{self, Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, mouse, overlay};
use iced::animation::Animation;
use iced::time::Instant;
use iced::widget::container;
use iced::{Background, Border, Element, Event, Length, Rectangle, Shadow, Size, Vector, window};

use crate::ui::theme::motion;
use crate::ui::theme::tokens;

pub struct FocusWithin<'a, Message> {
  content: Element<'a, Message>,
  style: Box<dyn Fn(bool) -> iced::widget::container::Style + 'a>,
}

struct State {
  focused: bool,
  anim: Animation<bool>,
  now: Instant,
}

impl Default for State {
  fn default() -> Self {
    Self {
      focused: false,
      anim: motion::base(false),
      now: Instant::now(),
    }
  }
}

fn mix_bg(a: Option<Background>, b: Option<Background>, t: f32) -> Option<Background> {
  match (a, b) {
    (Some(Background::Color(x)), Some(Background::Color(y))) => {
      Some(Background::Color(tokens::mix(x, y, t)))
    }
    (x, y) => {
      if t < 0.5 {
        x
      } else {
        y
      }
    }
  }
}

pub fn blend(a: container::Style, b: container::Style, t: f32) -> container::Style {
  container::Style {
    background: mix_bg(a.background, b.background, t),
    border: Border {
      color: tokens::mix(a.border.color, b.border.color, t),
      width: a.border.width + (b.border.width - a.border.width) * t,
      radius: if t < 0.5 {
        a.border.radius
      } else {
        b.border.radius
      },
    },
    shadow: Shadow {
      color: tokens::mix(a.shadow.color, b.shadow.color, t),
      offset: a.shadow.offset + (b.shadow.offset - a.shadow.offset) * t,
      blur_radius: a.shadow.blur_radius + (b.shadow.blur_radius - a.shadow.blur_radius) * t,
    },
    ..b
  }
}

struct AnyFocused(bool);

impl Operation for AnyFocused {
  fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
    operate(self);
  }

  fn focusable(&mut self, _id: Option<&widget::Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
    self.0 |= state.is_focused();
  }
}

pub fn focus_within<'a, Message>(
  content: impl Into<Element<'a, Message>>,
  style: impl Fn(bool) -> iced::widget::container::Style + 'a,
) -> FocusWithin<'a, Message> {
  FocusWithin {
    content: content.into(),
    style: Box::new(style),
  }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for FocusWithin<'_, Message> {
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

  fn layout(
    &mut self,
    tree: &mut Tree,
    renderer: &iced::Renderer,
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
    renderer: &iced::Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
  ) {
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
    let mut op = AnyFocused(false);
    self
      .content
      .as_widget_mut()
      .operate(&mut tree.children[0], layout, renderer, &mut op);
    let state = tree.state.downcast_mut::<State>();
    if let Event::Window(window::Event::RedrawRequested(now)) = event {
      state.now = *now;
      if state.anim.is_animating(*now) {
        shell.request_redraw();
      }
    }
    if state.focused != op.0 {
      state.focused = op.0;
      state.anim.go_mut(op.0, state.now);
      shell.request_redraw();
    }
  }

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut iced::Renderer,
    theme: &iced::Theme,
    style: &renderer::Style,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
  ) {
    use iced::advanced::Renderer as _;
    let state = tree.state.downcast_ref::<State>();
    let t = state.anim.interpolate(0.0, 1.0, state.now);
    let s = blend((self.style)(false), (self.style)(true), t);
    if let Some(bg) = s.background {
      renderer.fill_quad(
        Quad {
          bounds: layout.bounds(),
          border: s.border,
          shadow: s.shadow,
          snap: true,
        },
        bg,
      );
    }
    self.content.as_widget().draw(
      &tree.children[0],
      renderer,
      theme,
      style,
      layout,
      cursor,
      viewport,
    );
  }

  fn mouse_interaction(
    &self,
    tree: &Tree,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
    renderer: &iced::Renderer,
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
    renderer: &iced::Renderer,
    operation: &mut dyn Operation,
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
    renderer: &iced::Renderer,
    viewport: &Rectangle,
    translation: Vector,
  ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
    self.content.as_widget_mut().overlay(
      &mut tree.children[0],
      layout,
      renderer,
      viewport,
      translation,
    )
  }
}

impl<'a, Message: 'a> From<FocusWithin<'a, Message>> for Element<'a, Message> {
  fn from(f: FocusWithin<'a, Message>) -> Self {
    Element::new(f)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use iced::Color;

  #[test]
  fn blend_interpolates_border_and_background() {
    let a = container::Style {
      background: Some(Background::Color(Color::BLACK)),
      border: Border {
        color: Color::BLACK,
        width: 1.0,
        radius: 4.0.into(),
      },
      ..container::Style::default()
    };
    let b = container::Style {
      background: Some(Background::Color(Color::WHITE)),
      border: Border {
        color: Color::WHITE,
        width: 1.0,
        radius: 4.0.into(),
      },
      ..container::Style::default()
    };
    let m = blend(a, b, 0.5);
    assert_eq!(m.border.color.r, 0.5);
    assert_eq!(
      m.background,
      Some(Background::Color(Color::from_rgb(0.5, 0.5, 0.5)))
    );
  }
}
