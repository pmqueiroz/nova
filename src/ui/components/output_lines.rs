use std::cell::RefCell;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::text::{self, Paragraph as _, Renderer as _};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Renderer as _, mouse};
use iced::alignment;
use iced::widget::text::{LineHeight, Shaping, Span, Wrapping};
use iced::{Background, Color, Element, Font, Length, Pixels, Point, Rectangle, Size, Vector};

use crate::core::block::Lines;
use crate::core::grid::Cell;
use crate::ui::components::block::{LineMarks, Run, run_font, runs};
use crate::ui::theme::font;
use crate::ui::theme::tokens::{self, block::LINE_HEIGHT};

type Paragraph = <iced::Renderer as text::Renderer>::Paragraph;

pub type MarksFn<'a> = Box<dyn Fn(usize, &[Cell]) -> LineMarks + 'a>;

pub struct OutputLines<'a> {
  lines: Lines<'a>,
  fg: Color,
  size: f32,
  alpha: f32,
  marks: MarksFn<'a>,
}

struct Shaped {
  line: usize,
  runs: Vec<Run>,
  alpha: f32,
  size: f32,
  paragraph: Paragraph,
}

#[derive(Default)]
struct State {
  cache: RefCell<Vec<Shaped>>,
}

pub fn output_lines<'a>(
  lines: Lines<'a>,
  fg: Color,
  size: f32,
  alpha: f32,
  marks: MarksFn<'a>,
) -> OutputLines<'a> {
  OutputLines {
    lines,
    fg,
    size,
    alpha,
    marks,
  }
}

pub fn visible_range(top: f32, bottom: f32, line_h: f32, count: usize) -> (usize, usize) {
  if line_h <= 0.0 || bottom <= top {
    return (0, 0);
  }
  let start = (top.max(0.0) / line_h).floor() as usize;
  let end = (bottom.max(0.0) / line_h).ceil() as usize;
  (start.min(count), end.min(count))
}

fn shape(line: usize, runs: Vec<Run>, alpha: f32, size: f32) -> Shaped {
  let spans: Vec<Span<'_, (), Font>> = runs
    .iter()
    .map(|(t, fg, _, flags)| {
      Span::new(t.as_str())
        .color(tokens::with_alpha(*fg, alpha))
        .font(run_font(*flags))
        .size(size)
    })
    .collect();
  let paragraph = Paragraph::with_spans(text::Text {
    content: spans.as_slice(),
    bounds: Size::new(f32::INFINITY, size * LINE_HEIGHT),
    size: Pixels(size),
    line_height: LineHeight::Relative(LINE_HEIGHT),
    font: font::regular(),
    align_x: text::Alignment::Default,
    align_y: alignment::Vertical::Top,
    shaping: Shaping::Advanced,
    wrapping: Wrapping::None,
  });
  Shaped {
    line,
    runs,
    alpha,
    size,
    paragraph,
  }
}

fn draw_decorations(renderer: &mut iced::Renderer, shaped: &Shaped, origin: Point) {
  let size = shaped.size;
  let line_h = size * LINE_HEIGHT;
  let translation = origin - Point::ORIGIN;
  let baseline = translation + Vector::new(0.0, size + (line_h - size) / 2.0);
  for (k, (_, fg, bg, flags)) in shaped.runs.iter().enumerate() {
    let underline = flags & 4 != 0;
    let strike = flags & 8 != 0;
    if bg.is_none() && !underline && !strike {
      continue;
    }
    let regions = shaped.paragraph.span_bounds(k);
    if let Some(bg) = bg {
      for r in &regions {
        renderer.fill_quad(
          Quad {
            bounds: *r + translation,
            ..Quad::default()
          },
          Background::Color(tokens::with_alpha(*bg, shaped.alpha)),
        );
      }
    }
    let color = tokens::with_alpha(*fg, shaped.alpha);
    let mut rule = |dy: f32| {
      for r in &regions {
        renderer.fill_quad(
          Quad {
            bounds: Rectangle::new(
              r.position() + baseline - Vector::new(0.0, dy),
              Size::new(r.width, 1.0),
            ),
            ..Quad::default()
          },
          color,
        );
      }
    };
    if underline {
      rule(size * 0.08);
    }
    if strike {
      rule(size / 2.0);
    }
  }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for OutputLines<'_> {
  fn tag(&self) -> tree::Tag {
    tree::Tag::of::<State>()
  }

  fn state(&self) -> tree::State {
    tree::State::new(State::default())
  }

  fn size(&self) -> Size<Length> {
    Size::new(Length::Fill, Length::Shrink)
  }

  fn layout(
    &mut self,
    _tree: &mut Tree,
    _renderer: &iced::Renderer,
    limits: &layout::Limits,
  ) -> layout::Node {
    let height = self.lines.len() as f32 * self.size * LINE_HEIGHT;
    layout::Node::new(limits.resolve(Length::Fill, Length::Fixed(height), Size::new(0.0, height)))
  }

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut iced::Renderer,
    _theme: &iced::Theme,
    _style: &renderer::Style,
    layout: Layout<'_>,
    _cursor: mouse::Cursor,
    viewport: &Rectangle,
  ) {
    let bounds = layout.bounds();
    let Some(clip) = bounds.intersection(viewport) else {
      return;
    };
    let line_h = self.size * LINE_HEIGHT;
    let (start, end) = visible_range(
      clip.y - bounds.y,
      clip.y + clip.height - bounds.y,
      line_h,
      self.lines.len(),
    );
    let state = tree.state.downcast_ref::<State>();
    let mut cache = state.cache.borrow_mut();
    let mut previous = std::mem::take(&mut *cache);
    for i in start..end {
      let Some(cells) = self.lines.get(i) else {
        continue;
      };
      let line_runs = runs(cells, self.fg, (self.marks)(i, cells));
      if line_runs.is_empty() {
        continue;
      }
      let hit = previous.iter().position(|s| {
        s.line == i && s.alpha == self.alpha && s.size == self.size && s.runs == line_runs
      });
      let shaped = match hit {
        Some(k) => previous.swap_remove(k),
        None => shape(i, line_runs, self.alpha, self.size),
      };
      let origin = Point::new(bounds.x, bounds.y + i as f32 * line_h);
      draw_decorations(renderer, &shaped, origin);
      renderer.fill_paragraph(&shaped.paragraph, origin, self.fg, clip);
      cache.push(shaped);
    }
  }
}

impl<'a, Message: 'a> From<OutputLines<'a>> for Element<'a, Message> {
  fn from(w: OutputLines<'a>) -> Self {
    Element::new(w)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn visible_range_covers_partial_lines() {
    assert_eq!(visible_range(15.0, 45.0, 10.0, 100), (1, 5));
  }

  #[test]
  fn visible_range_clamps_to_count() {
    assert_eq!(visible_range(0.0, 1000.0, 10.0, 7), (0, 7));
    assert_eq!(visible_range(500.0, 600.0, 10.0, 7), (7, 7));
  }

  #[test]
  fn visible_range_empty_when_collapsed() {
    assert_eq!(visible_range(10.0, 10.0, 10.0, 7), (0, 0));
  }
}
