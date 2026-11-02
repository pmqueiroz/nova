use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, mouse, overlay};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

use crate::core::selection::{Cell, cell_at};

pub struct CellArea<'a, Message> {
  content: Element<'a, Message>,
  origin: Vector,
  cell: Size,
  on_move: Box<dyn Fn(Point) -> Message + 'a>,
}

#[derive(Default)]
struct State {
  last: Option<Cell>,
  pressed: bool,
}

pub fn cell_area<'a, Message>(
  content: impl Into<Element<'a, Message>>,
  origin: Vector,
  cell: Size,
  on_move: impl Fn(Point) -> Message + 'a,
) -> CellArea<'a, Message> {
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

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for CellArea<'_, Message> {
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
    renderer: &mut iced::Renderer,
    theme: &iced::Theme,
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

impl<'a, Message: 'a> From<CellArea<'a, Message>> for Element<'a, Message> {
  fn from(w: CellArea<'a, Message>) -> Self {
    Element::new(w)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn publishes_only_on_new_cells_unless_pressed() {
    assert!(should_publish(None, (0, 0), false));
    assert!(!should_publish(Some((2, 3)), (2, 3), false));
    assert!(should_publish(Some((2, 3)), (2, 4), false));
    assert!(should_publish(Some((2, 3)), (2, 3), true));
  }
}
