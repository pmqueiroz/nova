use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::{Element, Length, Point, Rectangle, Size, Vector};

use crate::ui::theme::motion;

pub struct Slide<'a, Message, Theme, Renderer> {
  front: Element<'a, Message, Theme, Renderer>,
  back: Element<'a, Message, Theme, Renderer>,
  progress: f32,
}

pub fn slide<'a, Message, Theme, Renderer>(
  front: impl Into<Element<'a, Message, Theme, Renderer>>,
  back: impl Into<Element<'a, Message, Theme, Renderer>>,
  progress: f32,
) -> Slide<'a, Message, Theme, Renderer> {
  Slide {
    front: front.into(),
    back: back.into(),
    progress: motion::ease(progress.clamp(0.0, 1.0)),
  }
}

pub fn offsets(progress: f32, height: f32) -> (f32, f32) {
  let shift = (progress * height).round();
  (-shift, height - shift)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
  for Slide<'_, Message, Theme, Renderer>
where
  Renderer: renderer::Renderer,
{
  fn children(&self) -> Vec<Tree> {
    vec![Tree::new(&self.front), Tree::new(&self.back)]
  }

  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(&[&self.front, &self.back]);
  }

  fn size(&self) -> Size<Length> {
    Size::new(Length::Shrink, Length::Shrink)
  }

  fn layout(
    &mut self,
    tree: &mut Tree,
    renderer: &Renderer,
    limits: &layout::Limits,
  ) -> layout::Node {
    let loose = limits.loose();
    let front = self
      .front
      .as_widget_mut()
      .layout(&mut tree.children[0], renderer, &loose);
    let back = self
      .back
      .as_widget_mut()
      .layout(&mut tree.children[1], renderer, &loose);
    let (fs, bs) = (front.size(), back.size());
    let height = fs.height.max(bs.height);
    let width = motion::lerp(fs.width, bs.width, self.progress).round();
    let center = |n: layout::Node, h: f32| n.move_to(Point::new(0.0, ((height - h) / 2.0).round()));
    layout::Node::with_children(
      limits.resolve(Length::Shrink, Length::Shrink, Size::new(width, height)),
      vec![center(front, fs.height), center(back, bs.height)],
    )
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
    let bounds = layout.bounds();
    let mut children = layout.children();
    let (Some(front_layout), Some(back_layout)) = (children.next(), children.next()) else {
      return;
    };
    let (front_y, back_y) = offsets(self.progress, bounds.height);
    renderer.with_layer(bounds, |r| {
      for (el, t, l, dy) in [
        (&self.front, &tree.children[0], front_layout, front_y),
        (&self.back, &tree.children[1], back_layout, back_y),
      ] {
        if dy.abs() < bounds.height {
          r.with_translation(Vector::new(0.0, dy), |r| {
            el.as_widget().draw(t, r, theme, style, l, cursor, viewport);
          });
        }
      }
    });
  }
}

impl<'a, Message, Theme, Renderer> From<Slide<'a, Message, Theme, Renderer>>
  for Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Theme: 'a,
  Renderer: renderer::Renderer + 'a,
{
  fn from(s: Slide<'a, Message, Theme, Renderer>) -> Self {
    Element::new(s)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn front_leaves_up_as_back_rises() {
    assert_eq!(offsets(0.0, 24.0), (0.0, 24.0));
    assert_eq!(offsets(0.5, 24.0), (-12.0, 12.0));
    assert_eq!(offsets(1.0, 24.0), (-24.0, 0.0));
  }
}
