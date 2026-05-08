use crate::core::grid::Cell;

pub fn detect_urls(cells: &[Cell]) -> Vec<(usize, usize, String)> {
  let mut results = Vec::new();
  let n = cells.len();
  let mut i = 0;

  while i < n {
    let prefix_len = if cells[i].c == 'h' {
      if i + 8 <= n && cells[i..i + 8].iter().map(|c| c.c).eq("https://".chars()) {
        8
      } else if i + 7 <= n && cells[i..i + 7].iter().map(|c| c.c).eq("http://".chars()) {
        7
      } else {
        0
      }
    } else {
      0
    };

    if prefix_len == 0 {
      i += 1;
      continue;
    }

    let mut end = i + prefix_len;
    while end < n {
      let c = cells[end].c;
      if c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | ')' | ']') {
        break;
      }
      end += 1;
    }

    if end > i + prefix_len {
      let url: String = cells[i..end].iter().map(|c| c.c).collect();
      results.push((i, end - 1, url));
      i = end;
    } else {
      i += 1;
    }
  }

  results
}
