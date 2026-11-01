use crate::core::block::{BlockId, Session};
use crate::core::grid::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockMatch {
  pub block_id: BlockId,
  pub line: usize,
  pub start: usize,
  pub end: usize,
}

pub fn line_matches(cells: &[Cell], query: &[char]) -> Vec<(usize, usize)> {
  let q = query.len();
  let n = cells.len();
  if q == 0 || n < q {
    return vec![];
  }
  let lower = |c: &Cell| c.c.chars().next().unwrap_or('\0').to_ascii_lowercase();
  (0..=n - q)
    .filter(|&s| (0..q).all(|i| lower(&cells[s + i]) == query[i]))
    .map(|s| (s, s + q))
    .collect()
}

pub fn search_session(session: &Session, query: &str) -> Vec<BlockMatch> {
  let q: Vec<char> = query.chars().map(|c| c.to_ascii_lowercase()).collect();
  let mut out = Vec::new();
  for b in &session.blocks {
    for line in 0..b.line_count() {
      if let Some(cells) = b.line(line) {
        for (start, end) in line_matches(cells, &q) {
          out.push(BlockMatch {
            block_id: b.id,
            line,
            start,
            end,
          });
        }
      }
    }
  }
  out
}

pub fn relative_position(session: &Session, m: &BlockMatch) -> f32 {
  let total: usize = session.blocks.iter().map(|b| b.line_count() + 2).sum();
  if total == 0 {
    return 1.0;
  }
  let mut before = 0usize;
  for b in &session.blocks {
    if b.id == m.block_id {
      before += 1 + m.line;
      break;
    }
    before += b.line_count() + 2;
  }
  (before as f32 / total as f32).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::sys::parser::AnsiExecutor;
  use vte::Parser;

  fn session_with(outputs: &[&str]) -> Session {
    let mut s = Session::new("~".into(), 30, 6);
    for (i, o) in outputs.iter().enumerate() {
      let id = s.submit(&format!("cmd{i}")).unwrap();
      let b = s.block_mut(id).unwrap();
      let mut p = Parser::new();
      let mut e = AnsiExecutor {
        grid: &mut b.output,
        bell_pending: false,
      };
      p.advance(&mut e, o.as_bytes());
      s.finish(0);
    }
    s
  }

  #[test]
  fn finds_case_insensitive_matches_across_blocks() {
    let s = session_with(&["Hello world\r\nhello", "nothing", "HELLO"]);
    let m = search_session(&s, "hello");
    assert_eq!(m.len(), 3);
    assert_eq!((m[0].line, m[0].start, m[0].end), (0, 0, 5));
    assert_eq!(m[1].line, 1);
    assert_eq!(m[2].block_id, s.blocks[2].id);
  }

  #[test]
  fn empty_query_matches_nothing() {
    let s = session_with(&["abc"]);
    assert!(search_session(&s, "").is_empty());
  }

  #[test]
  fn later_blocks_sit_lower() {
    let s = session_with(&["a\r\na\r\na", "b", "a"]);
    let m = search_session(&s, "a");
    let first = relative_position(&s, &m[0]);
    let last = relative_position(&s, m.last().unwrap());
    assert!(first < last);
  }
}
