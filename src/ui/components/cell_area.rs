use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, mouse, overlay};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

use crate::core::selection::{Cell, cell_at};

pub struct CellArea<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
  content: Element<'a, Message, Theme, Renderer>,
  origin: Vector,
  cell: Size,
  on_move: Box<dyn Fn(Point) -> Message + 'a>,
}

#[derive(Default)]
struct State {
  last: Option<Cell>,
  pressed: bool,
}

pub fn cell_area<'a, Message, Theme, Renderer>(
  content: impl Into<Element<'a, Message, Theme, Renderer>>,
  origin: Vector,
  cell: Size,
  on_move: impl Fn(Point) -> Message + 'a,
) -> CellArea<'a, Message, Theme, Renderer> {
  CellArea {
    content: content.into(),
    origin,
    cell,
    on_move: Box::new(on_move),
  }
}

pub fn should_publish(last: Option<Cell>, cell: Cell, pressed: bool) -> bool {
  pressed || last != Some(cell)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
  for CellArea<'_, Message, Theme, Renderer>
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
    let state = tree.state.downcast_mut::<State>();
    match event {
      Event::Mouse(mouse::Event::ButtonPressed(_)) if cursor.is_over(layout.bounds()) => {
        state.pressed = true;
      }
      Event::Mouse(mouse::Event::ButtonReleased(_)) => state.pressed = false,
      Event::Mouse(mouse::Event::CursorMoved { .. }) => match cursor.position_in(layout.bounds()) {
        Some(p) => {
          let cell = cell_at(
            p.x,
            p.y,
            self.origin.x,
            self.origin.y,
            self.cell.width,
            self.cell.height,
          );
          if should_publish(state.last, cell, state.pressed) {
            state.last = Some(cell);
            shell.publish((self.on_move)(p));
          }
        }
        None => state.last = None,
      },
      _ => {}
    }
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

impl<'a, Message, Theme, Renderer> From<CellArea<'a, Message, Theme, Renderer>>
  for Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Theme: 'a,
  Renderer: renderer::Renderer + 'a,
{
  fn from(w: CellArea<'a, Message, Theme, Renderer>) -> Self {
    Element::new(w)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use iced::advanced::clipboard;
  use iced::advanced::layout::{Limits, Node};
  use iced::widget::Space;

  #[derive(Debug, Clone, PartialEq)]
  struct Moved(Point);

  struct Rig {
    element: Element<'static, Moved, (), ()>,
    tree: Tree,
    node: Node,
  }

  impl Rig {
    fn new() -> Self {
      let mut element: Element<'static, Moved, (), ()> = cell_area(
        Space::new().width(100).height(100),
        Vector::new(10.0, 5.0),
        Size::new(8.0, 16.0),
        Moved,
      )
      .into();
      let mut tree = Tree::new(&element);
      let node = element.as_widget_mut().layout(
        &mut tree,
        &(),
        &Limits::new(Size::ZERO, Size::new(100.0, 100.0)),
      );
      Self {
        element,
        tree,
        node,
      }
    }

    fn send(&mut self, event: Event, at: Point) -> Vec<Moved> {
      let mut messages = Vec::new();
      let mut shell = Shell::new(&mut messages);
      self.element.as_widget_mut().update(
        &mut self.tree,
        &event,
        Layout::new(&self.node),
        mouse::Cursor::Available(at),
        &(),
        &mut clipboard::Null,
        &mut shell,
        &Rectangle::new(Point::ORIGIN, Size::new(100.0, 100.0)),
      );
      messages
    }

    fn move_to(&mut self, x: f32, y: f32) -> Vec<Moved> {
      let position = Point::new(x, y);
      self.send(
        Event::Mouse(mouse::Event::CursorMoved { position }),
        position,
      )
    }

    fn button(&mut self, pressed: bool, x: f32, y: f32) {
      let b = mouse::Button::Left;
      let event = if pressed {
        mouse::Event::ButtonPressed(b)
      } else {
        mouse::Event::ButtonReleased(b)
      };
      assert!(self.send(Event::Mouse(event), Point::new(x, y)).is_empty());
    }
  }

  #[test]
  fn publishes_only_on_new_cells_unless_pressed() {
    assert!(should_publish(None, (0, 0), false));
    assert!(!should_publish(Some((2, 3)), (2, 3), false));
    assert!(should_publish(Some((2, 3)), (2, 4), false));
    assert!(should_publish(Some((2, 3)), (2, 3), true));
  }

  #[test]
  fn widget_reports_moves_once_per_cell() {
    let mut r = Rig::new();
    assert_eq!(r.move_to(12.0, 6.0), [Moved(Point::new(12.0, 6.0))]);
    assert!(r.move_to(14.0, 10.0).is_empty());
    assert!(r.move_to(17.9, 20.9).is_empty());
    assert_eq!(r.move_to(19.0, 6.0), [Moved(Point::new(19.0, 6.0))]);
    assert_eq!(r.move_to(19.0, 22.0), [Moved(Point::new(19.0, 22.0))]);
  }

  #[test]
  fn widget_reports_every_move_while_dragging() {
    let mut r = Rig::new();
    assert_eq!(r.move_to(19.0, 22.0).len(), 1);
    r.button(true, 19.0, 22.0);
    assert_eq!(r.move_to(20.0, 22.0), [Moved(Point::new(20.0, 22.0))]);
    assert_eq!(r.move_to(21.0, 23.0), [Moved(Point::new(21.0, 23.0))]);
    r.button(false, 21.0, 23.0);
    assert!(r.move_to(22.0, 24.0).is_empty());
  }

  #[test]
  fn leaving_and_reentering_reports_again() {
    let mut r = Rig::new();
    assert_eq!(r.move_to(19.0, 22.0).len(), 1);
    assert!(r.move_to(150.0, 150.0).is_empty());
    assert_eq!(r.move_to(19.0, 22.0), [Moved(Point::new(19.0, 22.0))]);
  }
}
