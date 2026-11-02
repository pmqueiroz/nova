use crate::core::block::{Block, BlockId};
use crate::core::url::detect_urls;

pub type Cell = (usize, usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockSelection {
  pub pane_id: usize,
  pub block_id: BlockId,
  pub anchor: Cell,
  pub head: Cell,
  pub dragging: bool,
}

impl BlockSelection {
  pub fn ordered(&self) -> (Cell, Cell) {
    if self.anchor <= self.head {
      (self.anchor, self.head)
    } else {
      (self.head, self.anchor)
    }
  }

  pub fn is_empty(&self) -> bool {
    self.anchor == self.head
  }

  pub fn line_range(&self, line: usize, line_len: usize) -> Option<(usize, usize)> {
    let ((sl, sc), (el, ec)) = self.ordered();
    if line < sl || line > el || self.is_empty() {
      return None;
    }
    let start = if line == sl { sc } else { 0 };
    let end = if line == el { ec } else { line_len.max(start) };
    (end > start).then_some((start, end))
  }

  pub fn text(&self, block: &Block) -> String {
    let ((sl, sc), (el, ec)) = self.ordered();
    let mut out = Vec::new();
    for l in sl..=el {
      let chars: Vec<char> = block.line_text(l).chars().collect();
      let start = if l == sl { sc.min(chars.len()) } else { 0 };
      let end = if l == el {
        ec.min(chars.len())
      } else {
        chars.len()
      };
      out.push(chars[start..end.max(start)].iter().collect::<String>());
    }
    out.join("\n")
  }
}

pub fn cell_at(x: f32, y: f32, pad_x: f32, pad_y: f32, char_w: f32, line_h: f32) -> Cell {
  let col = ((x - pad_x).max(0.0) / char_w).floor() as usize;
  let line = ((y - pad_y).max(0.0) / line_h).floor() as usize;
  (line, col)
}

pub fn url_at(block: &Block, (line, col): Cell) -> Option<(usize, usize, String)> {
  let cells = block.line(line)?;
  detect_urls(cells)
    .into_iter()
    .find(|(s, e, _)| col >= *s && col <= *e)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::block::Session;
  use crate::sys::parser::AnsiExecutor;
  use vte::Parser;

  fn block(text: &str) -> Block {
    let mut s = Session::new("~".into(), 40, 6);
    let id = s.submit("x").unwrap();
    let mut b = s
      .blocks
      .remove(s.blocks.iter().position(|b| b.id == id).unwrap());
    let mut p = Parser::new();
    let mut e = AnsiExecutor::new(&mut b.output);
    p.advance(&mut e, text.as_bytes());
    b
  }

  fn sel(anchor: Cell, head: Cell) -> BlockSelection {
    BlockSelection {
      pane_id: 0,
      block_id: 1,
      anchor,
      head,
      dragging: false,
    }
  }

  #[test]
  fn backwards_drag_is_ordered() {
    let s = sel((2, 3), (0, 1));
    assert_eq!(s.ordered(), ((0, 1), (2, 3)));
  }

  #[test]
  fn text_spans_lines() {
    let b = block("hello world\r\nsecond line\r\nthird");
    assert_eq!(sel((0, 6), (1, 6)).text(&b), "world\nsecond");
  }

  #[test]
  fn middle_lines_select_fully() {
    let s = sel((0, 2), (2, 1));
    assert_eq!(s.line_range(1, 10), Some((0, 10)));
    assert_eq!(s.line_range(0, 10), Some((2, 10)));
    assert_eq!(s.line_range(2, 10), Some((0, 1)));
    assert_eq!(s.line_range(3, 10), None);
  }

  #[test]
  fn click_without_drag_is_empty() {
    assert!(sel((1, 1), (1, 1)).is_empty());
    assert_eq!(sel((1, 1), (1, 1)).line_range(1, 5), None);
  }

  #[test]
  fn cell_mapping_accounts_for_padding() {
    assert_eq!(cell_at(24.0, 16.0, 24.0, 16.0, 8.0, 21.0), (0, 0));
    assert_eq!(
      cell_at(24.0 + 17.0, 16.0 + 43.0, 24.0, 16.0, 8.0, 21.0),
      (2, 2)
    );
  }

  #[test]
  fn finds_url_under_cell() {
    let b = block("see https://example.com now");
    let (s, _, u) = url_at(&b, (0, 10)).unwrap();
    assert_eq!(s, 4);
    assert_eq!(u, "https://example.com");
    assert!(url_at(&b, (0, 1)).is_none());
  }
}
